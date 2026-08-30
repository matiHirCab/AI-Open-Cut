import { type ChildProcessWithoutNullStreams, spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import { constants } from "node:fs";
import { access, mkdir, readdir, rm, stat } from "node:fs/promises";
import { dirname, join } from "node:path";

import { z } from "zod/v4";

import type { BridgeConfig } from "./config";
import { resolveExecutablePath, runtimePathDiagnostics } from "./diagnostics";
import { normalizeProviderErrorCode, publicDescriptionFor } from "./errors";
import { BridgeError } from "./headless";
import { type Logger, NOOP_LOGGER } from "./logger";
import {
  speechTextOptionsSchema,
  speechVoiceListSchema,
  synthesizedSpeechMetadataSchema,
  ttsStatusSchema,
} from "./schemas";
import type {
  SpeechSynthesisRequest,
  SpeechSynthesizer,
  SynthesizedSpeech,
} from "./speech";
import { prepareSpeechSegments } from "./speech";

const NEWLINE_PATTERN = /\r?\n/;
const PATH_SEPARATOR_PATTERN = /[\\/]/;

const workerResultSchema = z.discriminatedUnion("type", [
  z
    .object({ id: z.string(), result: z.unknown(), type: z.literal("result") })
    .strict(),
  z
    .object({
      error: z
        .object({
          code: z.string(),
          message: z.string(),
          retryable: z.boolean(),
        })
        .strict(),
      id: z.string(),
      type: z.literal("error"),
    })
    .strict(),
]);

interface Pending {
  reject: (error: unknown) => void;
  resolve: (result: unknown) => void;
  timer: ReturnType<typeof setTimeout>;
}

interface QueuedGeneration {
  enqueuedAt: number;
  reject: (error: unknown) => void;
  request: SpeechSynthesisRequest;
  requestId: string;
  resolve: (result: SynthesizedSpeech) => void;
  signal: AbortSignal | undefined;
}

export class KokoroSpeechSynthesizer implements SpeechSynthesizer {
  #active: QueuedGeneration | undefined;
  #cachedStatus: z.infer<typeof ttsStatusSchema> | undefined;
  #cachedVoices: z.infer<typeof speechVoiceListSchema> | undefined;
  #child: ChildProcessWithoutNullStreams | undefined;
  #closed = false;
  readonly #config: BridgeConfig;
  readonly #logger: Logger;
  readonly #ownedPaths = new Set<string>();
  readonly #pending = new Map<string, Pending>();
  readonly #queue: QueuedGeneration[] = [];
  #stderr = "";
  #stdout = "";

  constructor(config: BridgeConfig, logger: Logger = NOOP_LOGGER) {
    this.#config = config;
    this.#logger = logger;
  }

  async status() {
    if (this.#active && this.#cachedStatus) {
      return { ...this.#cachedStatus, queue: this.queueStatus() };
    }
    let workerStatus: Record<string, unknown> | undefined;
    let startupError: BridgeError | undefined;
    try {
      workerStatus = (await this.#request(
        { operation: "status" },
        this.#config.ttsControlTimeoutMs
      )) as Record<string, unknown>;
    } catch (error) {
      startupError =
        error instanceof BridgeError
          ? error
          : new BridgeError("TTS_UNAVAILABLE", "Kokoro path validation failed");
    }
    const paths = (await runtimePathDiagnostics(this.#config)).kokoro;
    let status: z.infer<typeof ttsStatusSchema>;
    if (workerStatus) {
      status = ttsStatusSchema.parse({
        ...workerStatus,
        paths,
        startupError: null,
      });
    } else {
      const detail =
        startupError ??
        new BridgeError("TTS_UNAVAILABLE", "Kokoro unavailable");
      status = ttsStatusSchema.parse({
        defaultLanguage: "en-US",
        defaultSpeed: 1,
        defaultVoiceId: "af_heart",
        device: "cpu",
        devices: ["cpu"],
        languages: ["en-US", "en-GB"],
        limits: { maxSpeed: 2, maxTextCharacters: 5000, minSpeed: 0.5 },
        modelCached: false,
        modelId: "hexgrad/Kokoro-82M",
        modelLoaded: false,
        models: [
          { id: "hexgrad/Kokoro-82M", sampleRateHz: 24_000, version: null },
        ],
        modelVersion: null,
        paths,
        providerId: "kokoro",
        ready: false,
        resources: {
          execution: "local",
          minimumLogicalCpus: 2,
          minimumRamBytes: 2_147_483_648,
          recommendedLogicalCpus: 4,
          recommendedRamBytes: 4_294_967_296,
        },
        sampleRateHz: 24_000,
        startupError: {
          code: detail.code,
          failedStage: detail.failedStage,
          ffmpegExitCode: null,
          ffmpegStderrExcerpt: null,
          message: detail.message,
          retryable: detail.retryable,
        },
        version: "unavailable",
        voices: ["af_heart"],
      });
    }
    this.#cachedStatus = status;
    return { ...status, queue: this.queueStatus() };
  }

  async #preflight() {
    const pythonPath = resolveExecutablePath(
      this.#config.ttsPythonPath,
      this.#config.environment
    );
    try {
      const metadata = await stat(pythonPath);
      if (!metadata.isFile()) {
        throw new Error("not a file");
      }
      await access(
        pythonPath,
        process.platform === "win32" ? constants.R_OK : constants.X_OK
      );
    } catch (error) {
      // biome-ignore lint/style/useErrorCause: BridgeError receives ErrorOptions as its fourth argument.
      throw new BridgeError(
        "TTS_UNAVAILABLE",
        "Configured Kokoro Python executable is missing",
        undefined,
        { cause: error }
      );
    }
    try {
      const metadata = await stat(this.#config.ttsWorkerPath);
      if (!metadata.isFile()) {
        throw new Error("not a file");
      }
      await access(this.#config.ttsWorkerPath, constants.R_OK);
    } catch (error) {
      // biome-ignore lint/style/useErrorCause: BridgeError receives ErrorOptions as its fourth argument.
      throw new BridgeError(
        "TTS_UNAVAILABLE",
        "Configured Kokoro worker script is missing",
        undefined,
        { cause: error }
      );
    }
    try {
      const metadata = await stat(this.#config.ttsModelDirectory);
      if (!metadata.isDirectory()) {
        throw new Error("not a directory");
      }
      await access(this.#config.ttsModelDirectory, constants.R_OK);
    } catch (error) {
      // biome-ignore lint/style/useErrorCause: BridgeError receives ErrorOptions as its fourth argument.
      throw new BridgeError(
        "TTS_UNAVAILABLE",
        "Configured Kokoro model directory is missing",
        undefined,
        { cause: error }
      );
    }
    await mkdir(this.#config.ttsWorkDirectory, { recursive: true });
    await access(this.#config.ttsWorkDirectory, constants.R_OK);
    await access(this.#config.ttsWorkDirectory, constants.W_OK);
    return pythonPath;
  }

  async listVoices() {
    if (this.#active && this.#cachedVoices) {
      return this.#cachedVoices;
    }
    const voices = speechVoiceListSchema.parse(
      await this.#request(
        { operation: "list_voices" },
        this.#config.ttsControlTimeoutMs
      )
    );
    this.#cachedVoices = voices;
    return voices;
  }

  estimate(request: SpeechSynthesisRequest) {
    const segments = prepareSpeechSegments(request);
    const textOptions = speechTextOptionsSchema.parse(
      request.textOptions ?? {}
    );
    const characters = segments.reduce(
      (total, segment) => total + Array.from(segment).length,
      0
    );
    const pauseMs =
      Math.max(0, segments.length - 1) * textOptions.sentencePauseMs;
    const expectedDurationMs = Math.max(
      1,
      Math.round((characters / (15 * request.speed)) * 1000 + pauseMs)
    );
    return Promise.resolve({
      chunks: segments.length,
      expectedDurationMs,
      maximumDurationMs: Math.max(
        expectedDurationMs,
        Math.round(expectedDurationMs * 1.35)
      ),
      minimumDurationMs: Math.max(1, Math.round(expectedDurationMs * 0.75)),
    });
  }

  synthesize(request: SpeechSynthesisRequest, signal?: AbortSignal) {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: lifecycle state mutates across calls.
    if (this.#isClosed()) {
      return Promise.reject(
        new BridgeError(
          "BRIDGE_SHUTTING_DOWN",
          "Speech provider is shutting down",
          true
        )
      );
    }
    if (signal?.aborted) {
      return Promise.reject(
        new BridgeError("JOB_CANCELLED", "Speech synthesis was cancelled", true)
      );
    }
    if (this.#active && this.#queue.length >= this.#config.ttsMaxQueued) {
      return Promise.reject(
        new BridgeError(
          "TTS_QUEUE_FULL",
          "Speech synthesis queue is full",
          true
        )
      );
    }
    return new Promise<SynthesizedSpeech>((resolvePromise, rejectPromise) => {
      const queued: QueuedGeneration = {
        enqueuedAt: performance.now(),
        reject: rejectPromise,
        request,
        requestId: randomUUID(),
        resolve: resolvePromise,
        signal,
      };
      const onAbort = () => {
        const index = this.#queue.indexOf(queued);
        if (index >= 0) {
          this.#queue.splice(index, 1);
          rejectPromise(
            new BridgeError(
              "JOB_CANCELLED",
              "Speech synthesis was cancelled",
              true
            )
          );
        } else if (this.#active === queued) {
          this.#terminate(
            new BridgeError(
              "JOB_CANCELLED",
              "Speech synthesis was cancelled",
              true
            )
          );
        }
      };
      signal?.addEventListener("abort", onAbort, { once: true });
      const { reject, resolve } = queued;
      queued.resolve = (value) => {
        signal?.removeEventListener("abort", onAbort);
        resolve(value);
      };
      queued.reject = (error) => {
        signal?.removeEventListener("abort", onAbort);
        reject(error);
      };
      if (this.#active) {
        this.#queue.push(queued);
      } else {
        this.#start(queued);
      }
    });
  }

  queueStatus() {
    return {
      active: this.#active ? 1 : 0,
      concurrency: 1 as const,
      fairness: "fifo" as const,
      maxQueued: this.#config.ttsMaxQueued,
      queued: this.#queue.length,
    };
  }

  async cleanup(path: string) {
    await this.#removeOwnedOutput(path);
    this.#ownedPaths.delete(path);
  }

  cancel() {
    const error = new BridgeError(
      "JOB_CANCELLED",
      "Speech synthesis was cancelled",
      true
    );
    for (const queued of this.#queue.splice(0)) {
      queued.reject(error);
    }
    this.#terminate(error);
  }

  async close() {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: lifecycle state mutates across calls.
    if (this.#isClosed()) {
      return;
    }
    this.#closed = true;
    const error = new BridgeError(
      "BRIDGE_SHUTTING_DOWN",
      "Speech provider is shutting down",
      true
    );
    for (const queued of this.#queue.splice(0)) {
      queued.reject(error);
    }
    this.#terminate(error);
    await Promise.allSettled(
      [...this.#ownedPaths].map(async (path) => await this.cleanup(path))
    );
  }

  #start(queued: QueuedGeneration) {
    this.#active = queued;
    const providerId =
      this.#cachedStatus?.providerId ?? this.#config.speechProviderId;
    this.#logger.info("provider.generation.started", {
      providerId,
      queueWaitMs: Math.round(performance.now() - queued.enqueuedAt),
      requestId: queued.requestId,
    });
    const startedAt = performance.now();
    this.#run(queued)
      .then(
        (result) => {
          this.#logger.info("provider.generation.completed", {
            durationMs: Math.round(performance.now() - startedAt),
            providerId,
            requestId: queued.requestId,
            status: "completed",
          });
          queued.resolve(result);
        },
        (error: unknown) => {
          this.#logger.error("provider.generation.failed", {
            code:
              error instanceof BridgeError ? error.code : "TTS_PROVIDER_FAILED",
            durationMs: Math.round(performance.now() - startedAt),
            providerId,
            requestId: queued.requestId,
            status: "failed",
          });
          queued.reject(error);
        }
      )
      .finally(() => {
        if (this.#active === queued) {
          this.#active = undefined;
        }
        const next = this.#queue.shift();
        if (next && !this.#isClosed()) {
          this.#start(next);
        }
      });
  }

  async #run(queued: QueuedGeneration) {
    await mkdir(this.#config.ttsWorkDirectory, { recursive: true });
    if (queued.signal?.aborted) {
      throw new BridgeError(
        "JOB_CANCELLED",
        "Speech synthesis was cancelled",
        true
      );
    }
    const outputPath = join(
      this.#config.ttsWorkDirectory,
      `${randomUUID()}.wav`
    );
    this.#ownedPaths.add(outputPath);
    try {
      const segments = prepareSpeechSegments(queued.request);
      const textOptions = speechTextOptionsSchema.parse(
        queued.request.textOptions ?? {}
      );
      const result = synthesizedSpeechMetadataSchema.parse(
        await this.#request(
          {
            language: queued.request.language,
            operation: "generate",
            outputPath,
            segments,
            sentencePauseMs: textOptions.sentencePauseMs,
            speed: queued.request.speed,
            text: queued.request.text,
            voice: queued.request.voiceId,
          },
          this.#config.ttsSynthesisTimeoutMs,
          "TTS_TIMEOUT"
        )
      );
      if (
        result.language !== queued.request.language ||
        result.voiceId !== queued.request.voiceId
      ) {
        throw new BridgeError(
          "TTS_INVALID_OUTPUT",
          "Kokoro returned speech for a different language or voice"
        );
      }
      return { ...result, outputPath, request: queued.request };
    } catch (error) {
      await this.cleanup(outputPath);
      throw error;
    }
  }

  async #removeOwnedOutput(path: string) {
    await rm(path, { force: true });
    const prefix = `.${path.split(PATH_SEPARATOR_PATTERN).at(-1)}.`;
    const entries = await readdir(this.#config.ttsWorkDirectory, {
      withFileTypes: true,
    }).catch(() => []);
    await Promise.all(
      entries
        .filter(
          (entry) =>
            entry.isFile() &&
            entry.name.startsWith(prefix) &&
            entry.name.endsWith(".tmp")
        )
        .map(
          async (entry) =>
            await rm(join(this.#config.ttsWorkDirectory, entry.name), {
              force: true,
            })
        )
    );
  }

  async #request(
    request: Record<string, unknown>,
    timeoutMs: number,
    timeoutCode = "TTS_TIMEOUT"
  ) {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: lifecycle state mutates across requests.
    if (this.#isClosed()) {
      throw new BridgeError(
        "BRIDGE_SHUTTING_DOWN",
        "Speech provider is shutting down",
        true
      );
    }
    const pythonPath = await this.#preflight();
    const child = this.#ensureChild(pythonPath);
    const id = randomUUID();
    return await new Promise<unknown>((resolvePromise, rejectPromise) => {
      const timer = setTimeout(() => {
        const error = new BridgeError(
          timeoutCode,
          "Speech provider request timed out",
          true
        );
        this.#terminate(error);
      }, timeoutMs);
      this.#pending.set(id, {
        reject: rejectPromise,
        resolve: resolvePromise,
        timer,
      });
      child.stdin.write(`${JSON.stringify({ ...request, id })}\n`, (error) => {
        if (error) {
          clearTimeout(timer);
          this.#pending.delete(id);
          rejectPromise(
            new BridgeError(
              "TTS_WORKER_TERMINATED",
              "Kokoro worker terminated",
              true
            )
          );
        }
      });
    });
  }

  #ensureChild(pythonPath: string) {
    if (this.#child && !this.#child.killed) {
      return this.#child;
    }
    this.#stdout = "";
    this.#stderr = "";
    const child = spawn(pythonPath, [this.#config.ttsWorkerPath], {
      cwd: dirname(this.#config.ttsWorkerPath),
      env: {
        ...this.#config.environment,
        CUDA_VISIBLE_DEVICES: "",
        HF_HOME: this.#config.ttsModelDirectory,
        HF_HUB_OFFLINE: "1",
        OPENCUT_KOKORO_MODEL_DIR: this.#config.ttsModelDirectory,
        OPENCUT_TTS_WORK_DIR: this.#config.ttsWorkDirectory,
      },
      stdio: ["pipe", "pipe", "pipe"],
      windowsHide: true,
    });
    this.#child = child;
    child.stdout.setEncoding("utf8");
    child.stdout.on("data", (chunk: string) => this.#onStdout(chunk));
    child.stderr.setEncoding("utf8");
    child.stderr.on("data", (chunk: string) => {
      if (this.#stderr.length < 4096) {
        this.#stderr += chunk;
      }
    });
    child.on("error", () => this.#terminate());
    child.on("close", () => this.#terminate());
    return child;
  }

  #onStdout(chunk: string) {
    this.#stdout += chunk;
    const lines = this.#stdout.split(NEWLINE_PATTERN);
    this.#stdout = lines.pop() ?? "";
    for (const line of lines) {
      if (!line.trim()) {
        continue;
      }
      try {
        const event = workerResultSchema.parse(JSON.parse(line));
        const pending = this.#pending.get(event.id);
        if (!pending) {
          continue;
        }
        clearTimeout(pending.timer);
        this.#pending.delete(event.id);
        if (event.type === "error") {
          const code = normalizeProviderErrorCode(event.error.code);
          pending.reject(new BridgeError(code, publicDescriptionFor(code)));
        } else {
          pending.resolve(event.result);
        }
      } catch {
        this.#terminate();
      }
    }
  }

  #terminate(terminationError?: BridgeError) {
    if (this.#child) {
      this.#child.removeAllListeners();
      this.#child.kill();
      this.#child = undefined;
    }
    const error =
      terminationError ??
      new BridgeError(
        "TTS_WORKER_TERMINATED",
        publicDescriptionFor("TTS_WORKER_TERMINATED"),
        true
      );
    for (const pending of this.#pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(error);
    }
    this.#pending.clear();
  }

  #isClosed() {
    return this.#closed;
  }
}
