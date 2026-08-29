import { type ChildProcessWithoutNullStreams, spawn } from "node:child_process";
import { randomUUID } from "node:crypto";

import { z } from "zod/v4";

import type { BridgeConfig } from "./config";
import { BridgeError, type HeadlessClient } from "./headless";
import type { JobTaskContext } from "./jobs";
import type { Logger } from "./logger";
import {
  resolvedAssetInputSchema,
  type schemas,
  transcriptionEstimateSchema,
  transcriptionPreviewResultSchema,
  transcriptionSegmentSchema,
  transcriptionStatusSchema,
  writeResultSchema,
} from "./schemas";

type Status = z.infer<typeof transcriptionStatusSchema>;
type Segment = z.infer<typeof transcriptionSegmentSchema>;

export interface TranscriptionResult {
  durationMs: number;
  language: string;
  segments: Segment[];
}

export interface Transcriber {
  close: () => Promise<void>;
  queueStatus: () => Status["queue"];
  status: () => Promise<Status>;
  transcribe: (
    path: string,
    language: string | undefined,
    durationMs: number,
    signal: AbortSignal
  ) => Promise<TranscriptionResult>;
}

interface RetainedPreview
  extends z.infer<typeof transcriptionPreviewResultSchema> {
  timer: ReturnType<typeof setTimeout>;
}

interface PendingWorkerRequest {
  abort: () => void;
  reject: (error: unknown) => void;
  resolve: (value: unknown) => void;
  signal: AbortSignal | undefined;
  timer: ReturnType<typeof setTimeout>;
}

const DEFAULT_STYLE = {
  backgroundColor: "#000000",
  bottomMarginPx: 64,
  color: "#FFFFFF",
  fontSize: 48,
};

export class FasterWhisperTranscriber implements Transcriber {
  #active = 0;
  #buffer = "";
  #child: ChildProcessWithoutNullStreams | undefined;
  #closed = false;
  #queued = 0;
  readonly #pending = new Map<string, PendingWorkerRequest>();
  #tail: Promise<void> = Promise.resolve();
  readonly #config: BridgeConfig;
  readonly #logger: Logger;

  constructor(config: BridgeConfig, logger: Logger) {
    this.#config = config;
    this.#logger = logger;
  }

  queueStatus() {
    return {
      active: this.#active,
      concurrency: 1 as const,
      fairness: "fifo" as const,
      maxQueued: this.#config.transcriptionMaxQueued,
      queued: this.#queued,
    };
  }

  async status() {
    const { maxDurationMs, ...result } = z
      .object({
        computeType: z.literal("int8"),
        device: z.literal("cpu"),
        maxDurationMs: z.int().positive(),
        modelCached: z.boolean(),
        modelId: z.string().min(1),
        modelLoaded: z.boolean(),
        modelVersion: z.string().min(1).nullable(),
        providerId: z.string().min(1),
        ready: z.boolean(),
        version: z.string().min(1),
      })
      .strict()
      .parse(
        await this.#request(
          { operation: "status" },
          this.#config.transcriptionControlTimeoutMs
        )
      );
    return transcriptionStatusSchema.parse({
      ...result,
      limits: { maxDurationMs },
      queue: this.queueStatus(),
    });
  }

  async transcribe(
    path: string,
    language: string | undefined,
    durationMs: number,
    signal: AbortSignal
  ) {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: lifecycle state mutates across calls.
    if (this.#closed) {
      throw new BridgeError(
        "TRANSCRIPTION_UNAVAILABLE",
        "Transcription provider is closed"
      );
    }
    if (this.#queued >= this.#config.transcriptionMaxQueued) {
      throw new BridgeError(
        "TRANSCRIPTION_QUEUE_FULL",
        "Transcription queue is full",
        true
      );
    }
    this.#queued += 1;
    let release!: () => void;
    const predecessor = this.#tail;
    this.#tail = new Promise<void>((resolve) => {
      release = resolve;
    });
    await predecessor;
    this.#queued -= 1;
    this.#active = 1;
    try {
      const result = await this.#request(
        {
          durationMs,
          language,
          operation: "transcribe",
          path,
          vadFilter: true,
          wordTimestamps: true,
        },
        this.#config.transcriptionTimeoutMs,
        signal
      );
      const parsed = z
        .object({
          durationMs: z.int().positive(),
          language: z.string().min(1),
          segments: z.array(transcriptionSegmentSchema),
        })
        .strict()
        .parse(result);
      return parsed;
    } finally {
      this.#active = 0;
      release();
    }
  }

  async close() {
    this.#closed = true;
    await this.#tail;
    this.#rejectAll(
      new BridgeError(
        "TRANSCRIPTION_UNAVAILABLE",
        "Transcription provider is closed"
      )
    );
    this.#child?.kill();
    this.#child = undefined;
  }

  #request(
    request: Record<string, unknown>,
    timeoutMs: number,
    signal?: AbortSignal
  ): Promise<unknown> {
    return new Promise((resolve, reject) => {
      // biome-ignore lint/suspicious/noUnnecessaryConditions: lifecycle state mutates across calls.
      if (this.#closed) {
        reject(
          new BridgeError(
            "TRANSCRIPTION_UNAVAILABLE",
            "Transcription provider is closed"
          )
        );
        return;
      }
      const id = randomUUID();
      const child = this.#worker();
      const abort = () => {
        this.#settle(
          id,
          new BridgeError("JOB_CANCELLED", "Transcription was cancelled", true)
        );
        child.kill();
      };
      const timer = setTimeout(() => {
        this.#settle(
          id,
          new BridgeError(
            "TRANSCRIPTION_TIMEOUT",
            "Transcription provider timed out",
            true
          )
        );
        child.kill();
      }, timeoutMs);
      signal?.addEventListener("abort", abort, { once: true });
      this.#pending.set(id, { abort, reject, resolve, signal, timer });
      child.stdin.write(`${JSON.stringify({ ...request, id })}\n`);
      this.#logger.debug("transcription.worker.request", {
        operation: String(request.operation),
      });
    });
  }

  #worker() {
    if (this.#child) {
      return this.#child;
    }
    const child = spawn(
      this.#config.transcriptionPythonPath,
      [
        this.#config.transcriptionWorkerPath,
        "--model",
        this.#config.transcriptionModelId,
        "--model-dir",
        this.#config.transcriptionModelDirectory,
      ],
      { stdio: ["pipe", "pipe", "pipe"], windowsHide: true }
    );
    this.#child = child;
    child.stderr.resume();
    child.stdout.setEncoding("utf8");
    child.stdout.on("data", (chunk: string) => this.#consume(chunk));
    child.once("error", (error) =>
      this.#rejectAll(
        new BridgeError(
          "TRANSCRIPTION_UNAVAILABLE",
          "Transcription worker could not start",
          false,
          { cause: error }
        )
      )
    );
    child.once("exit", () => {
      if (this.#child === child) {
        this.#child = undefined;
      }
      this.#buffer = "";
      this.#rejectAll(
        new BridgeError(
          "TRANSCRIPTION_WORKER_TERMINATED",
          "Transcription worker terminated",
          true
        )
      );
    });
    return child;
  }

  #consume(chunk: string) {
    this.#buffer += chunk;
    for (
      let newline = this.#buffer.indexOf("\n");
      newline >= 0;
      newline = this.#buffer.indexOf("\n")
    ) {
      const line = this.#buffer.slice(0, newline);
      this.#buffer = this.#buffer.slice(newline + 1);
      try {
        const response = z
          .object({
            error: z
              .object({
                code: z.string().optional(),
                message: z.string().optional(),
              })
              .optional(),
            id: z.string(),
            ok: z.boolean(),
            result: z.unknown().optional(),
          })
          .parse(JSON.parse(line));
        this.#settle(
          response.id,
          response.ok
            ? undefined
            : new BridgeError(
                response.error?.code ?? "TRANSCRIPTION_PROVIDER_FAILED",
                response.error?.message ?? "Transcription failed",
                true
              ),
          response.result
        );
      } catch (error) {
        this.#rejectAll(
          new BridgeError(
            "TRANSCRIPTION_INVALID_OUTPUT",
            "Transcription worker returned invalid JSON",
            false,
            { cause: error }
          )
        );
        this.#child?.kill();
      }
    }
  }

  #settle(id: string, error?: unknown, value?: unknown) {
    const pending = this.#pending.get(id);
    if (!pending) {
      return;
    }
    this.#pending.delete(id);
    clearTimeout(pending.timer);
    pending.signal?.removeEventListener("abort", pending.abort);
    if (error) {
      pending.reject(error);
    } else {
      pending.resolve(value);
    }
  }

  #rejectAll(error: unknown) {
    for (const id of [...this.#pending.keys()]) {
      this.#settle(id, error);
    }
  }
}

export class TranscriptionApplicationService {
  readonly #previews = new Map<string, RetainedPreview>();
  #closed = false;
  readonly #headless: HeadlessClient;
  readonly #now: () => number;
  readonly #provider: Transcriber;
  readonly #ttlMs: number;

  constructor(
    provider: Transcriber,
    headless: HeadlessClient,
    ttlMs = 600_000,
    now: () => number = Date.now
  ) {
    this.#provider = provider;
    this.#headless = headless;
    this.#ttlMs = ttlMs;
    this.#now = now;
  }

  status() {
    return this.#provider.status();
  }

  async doctorTranscribe(path: string, durationMs: number) {
    return await this.#provider.transcribe(
      path,
      undefined,
      durationMs,
      new AbortController().signal
    );
  }

  async estimate(input: z.infer<(typeof schemas)["transcriptionEstimate"]>) {
    const [status, asset] = await Promise.all([
      this.status(),
      this.#headless.call(
        {
          assetId: input.assetId,
          operation: "resolve_asset_input",
          projectId: input.projectId,
        },
        resolvedAssetInputSchema
      ),
    ]);
    if (!(asset.probe?.hasAudio && asset.probe.durationMs)) {
      throw new BridgeError(
        "VALIDATION_FAILED",
        "Asset has no probed audio duration"
      );
    }
    return transcriptionEstimateSchema.parse({
      cost: { amount: 0, billing: "local", currency: null },
      durationMs: asset.probe.durationMs,
      language: input.language ?? null,
      modelCached: status.modelCached,
      modelId: status.modelId,
      providerId: status.providerId,
      queue: this.#provider.queueStatus(),
    });
  }

  async preview(
    input: z.infer<(typeof schemas)["transcriptionPreview"]>,
    context: JobTaskContext
  ) {
    this.#assertOpen();
    const [status, asset] = await Promise.all([
      this.status(),
      this.#headless.call(
        {
          assetId: input.assetId,
          operation: "resolve_asset_input",
          projectId: input.projectId,
        },
        resolvedAssetInputSchema
      ),
    ]);
    if (!status.ready) {
      throw new BridgeError(
        "TRANSCRIPTION_UNAVAILABLE",
        "Transcription model is not prepared"
      );
    }
    if (!(asset.probe?.hasAudio && asset.probe.durationMs)) {
      throw new BridgeError(
        "VALIDATION_FAILED",
        "Asset has no probed audio duration"
      );
    }
    if (asset.probe.durationMs > status.limits.maxDurationMs) {
      throw new BridgeError(
        "VALIDATION_FAILED",
        "Asset exceeds transcription duration limit"
      );
    }
    context.onProgress(0.05);
    const result = await this.#provider.transcribe(
      asset.path,
      input.language,
      asset.probe.durationMs,
      context.signal
    );
    const token = randomUUID();
    const expiresAtMs = this.#now() + this.#ttlMs;
    const preview = transcriptionPreviewResultSchema.parse({
      ...result,
      assetId: input.assetId,
      baseRevision: asset.revision,
      expiresAtMs,
      modelId: status.modelId,
      modelVersion: status.modelVersion,
      projectId: input.projectId,
      providerId: status.providerId,
      token,
    });
    const timer = setTimeout(() => this.#previews.delete(token), this.#ttlMs);
    timer.unref?.();
    this.#previews.set(token, { ...preview, timer });
    context.onProgress(1);
    return preview;
  }

  async commitPreview(
    input: z.infer<(typeof schemas)["transcriptionCommitPreview"]>
  ) {
    const preview = this.#preview(input.token);
    if (preview.projectId !== input.projectId) {
      throw new BridgeError(
        "VALIDATION_FAILED",
        "Preview belongs to another project"
      );
    }
    const result = await this.#headless.call(
      {
        assetId: preview.assetId,
        captionTrackId: input.captionTrackId,
        expectedRevision: input.expectedRevision,
        generatedAtMs: this.#now(),
        language: preview.language,
        modelId: preview.modelId,
        modelVersion: preview.modelVersion ?? undefined,
        operation: "commit_transcription",
        projectId: input.projectId,
        providerId: preview.providerId,
        segments: preview.segments,
        style: input.style ?? DEFAULT_STYLE,
      },
      writeResultSchema
    );
    clearTimeout(preview.timer);
    this.#previews.delete(input.token);
    return result;
  }

  discardPreview(token: string) {
    const preview = this.#preview(token);
    clearTimeout(preview.timer);
    this.#previews.delete(token);
    return { discarded: true as const, token };
  }
  async close() {
    this.#closed = true;
    for (const preview of this.#previews.values()) {
      clearTimeout(preview.timer);
    }
    this.#previews.clear();
    await this.#provider.close();
  }
  #preview(token: string) {
    const preview = this.#previews.get(token);
    if (!preview || preview.expiresAtMs <= this.#now()) {
      this.#previews.delete(token);
      throw new BridgeError(
        "TRANSCRIPTION_PREVIEW_NOT_FOUND",
        "Transcription preview was not found"
      );
    }
    return preview;
  }
  #assertOpen() {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: lifecycle state mutates across calls.
    if (this.#closed) {
      throw new BridgeError(
        "TRANSCRIPTION_UNAVAILABLE",
        "Transcription service is closed"
      );
    }
  }
}
