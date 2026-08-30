import { type ChildProcessWithoutNullStreams, spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import { rm } from "node:fs/promises";
import { dirname, join } from "node:path";

import { z } from "zod/v4";

import type { BridgeConfig } from "./config";
import { retryableFor } from "./errors";
import type { HeadlessRequest } from "./headless-contract";
import { type Logger, NOOP_LOGGER } from "./logger";

const NEWLINE_PATTERN = /\r?\n/;

const errorSchema = z
  .object({
    code: z.string(),
    failedStage: z.string().nullable().default(null),
    ffmpegExitCode: z.number().int().nullable().default(null),
    ffmpegStderrExcerpt: z.string().nullable().default(null),
    message: z.string(),
    retryable: z.boolean(),
  })
  .strict();

const eventSchema = z.discriminatedUnion("type", [
  z.object({ progress: z.number(), type: z.literal("progress") }).strict(),
  z.object({ result: z.unknown(), type: z.literal("result") }).strict(),
  z.object({ error: errorSchema, type: z.literal("error") }).strict(),
]);

export class BridgeError extends Error {
  readonly code: string;
  readonly retryable: boolean;
  readonly failedStage: string | null;
  readonly ffmpegExitCode: number | null;
  readonly ffmpegStderrExcerpt: string | null;

  constructor(
    code: string,
    message: string,
    _retryable = retryableFor(code),
    options?: ErrorOptions,
    details: {
      failedStage?: string | null;
      ffmpegExitCode?: number | null;
      ffmpegStderrExcerpt?: string | null;
    } = {}
  ) {
    super(message, options);
    this.name = "BridgeError";
    this.code = code;
    this.retryable = _retryable;
    this.failedStage = details.failedStage ?? null;
    this.ffmpegExitCode = details.ffmpegExitCode ?? null;
    this.ffmpegStderrExcerpt = details.ffmpegStderrExcerpt ?? null;
  }
}

interface HeadlessCallOptions {
  onProgress?: (progress: number) => void;
  requestId?: string;
  signal?: AbortSignal;
  timeoutMs?: number;
}

export class HeadlessClient {
  readonly #active = new Set<AbortController>();
  readonly #config: BridgeConfig;
  readonly #logger: Logger;
  #closed = false;

  constructor(config: BridgeConfig, logger: Logger = NOOP_LOGGER) {
    this.#config = config;
    this.#logger = logger;
  }

  async call<Output>(
    request: HeadlessRequest,
    schema: z.ZodType<Output>,
    options: HeadlessCallOptions = {}
  ) {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: lifecycle state changes across calls.
    if (this.#isClosed()) {
      throw new BridgeError(
        "BRIDGE_SHUTTING_DOWN",
        "OpenCut bridge is shutting down",
        true
      );
    }
    const controller = new AbortController();
    const requestId = options.requestId ?? randomUUID();
    const startedAt = performance.now();
    this.#logger.info("headless.request.started", {
      operation: request.operation,
      requestId,
    });
    const forwardAbort = () => controller.abort();
    options.signal?.addEventListener("abort", forwardAbort, { once: true });
    this.#active.add(controller);
    try {
      const result = await callHeadless(this.#config, request, schema, {
        ...options,
        requestId,
        signal: controller.signal,
      });
      this.#logger.info("headless.request.completed", {
        durationMs: Math.round(performance.now() - startedAt),
        operation: request.operation,
        requestId,
        status: "completed",
      });
      return result;
    } catch (error) {
      this.#logger.error("headless.request.failed", {
        code: error instanceof BridgeError ? error.code : "INTERNAL_ERROR",
        durationMs: Math.round(performance.now() - startedAt),
        operation: request.operation,
        requestId,
        status: "failed",
      });
      throw error;
    } finally {
      options.signal?.removeEventListener("abort", forwardAbort);
      this.#active.delete(controller);
    }
  }

  close() {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: close is intentionally idempotent.
    if (this.#isClosed()) {
      return;
    }
    this.#closed = true;
    for (const controller of this.#active) {
      controller.abort();
    }
    this.#active.clear();
  }

  #isClosed() {
    return this.#closed;
  }
}

export const callHeadless = async <Output>(
  config: BridgeConfig,
  request: HeadlessRequest,
  schema: z.ZodType<Output>,
  options: HeadlessCallOptions = {}
) => {
  const requestId = options.requestId ?? randomUUID();
  const ownedPaths = ownedTemporaryPaths(config, request, requestId);
  return await new Promise<Output>((resolvePromise, rejectPromise) => {
    const child = spawn(config.headlessPath, config.headlessArguments, {
      detached: process.platform !== "win32",
      env: { ...config.environment, OPENCUT_REQUEST_ID: requestId },
      stdio: ["pipe", "pipe", "pipe"],
      windowsHide: true,
    });
    let stdout = "";
    let stderr = "";
    let result: Output | undefined;
    let reportedError: BridgeError | undefined;
    let settled = false;

    const cleanup = async () => {
      await Promise.all(
        ownedPaths.map(
          async (path) => await rm(path, { force: true, recursive: true })
        )
      );
    };
    const finishReject = (error: BridgeError) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(timer);
      options.signal?.removeEventListener("abort", onAbort);
      cleanup().finally(() => rejectPromise(error));
    };
    const terminate = (error: BridgeError) => {
      terminateProcessTree(child);
      finishReject(error);
    };
    const onAbort = () =>
      terminate(
        new BridgeError(
          "JOB_CANCELLED",
          "OpenCut operation was cancelled",
          true
        )
      );
    const timer = setTimeout(
      () =>
        terminate(
          new BridgeError(
            "HEADLESS_TIMEOUT",
            "OpenCut headless request timed out",
            true
          )
        ),
      options.timeoutMs ?? config.headlessRequestTimeoutMs
    );

    options.signal?.addEventListener("abort", onAbort, { once: true });
    if (options.signal?.aborted) {
      onAbort();
      return;
    }

    child.stdout.setEncoding("utf8");
    const handleEvent = (event: z.infer<typeof eventSchema>) => {
      if (event.type === "progress") {
        options.onProgress?.(event.progress);
      } else if (event.type === "error") {
        reportedError = new BridgeError(
          event.error.code,
          event.error.message,
          event.error.retryable,
          undefined,
          event.error
        );
      } else {
        result = schema.parse(event.result);
      }
    };
    child.stdout.on("data", (chunk: string) => {
      if (settled) {
        return;
      }
      stdout += chunk;
      const lines = stdout.split(NEWLINE_PATTERN);
      stdout = lines.pop() ?? "";
      try {
        for (const line of lines) {
          if (!line.trim()) {
            continue;
          }
          handleEvent(eventSchema.parse(JSON.parse(line)));
        }
      } catch {
        terminate(
          new BridgeError(
            "INTERNAL_ERROR",
            "OpenCut returned malformed protocol output"
          )
        );
      }
    });
    child.stderr.setEncoding("utf8");
    child.stderr.on("data", (chunk: string) => {
      if (stderr.length < 4096) {
        stderr += chunk;
      }
    });
    child.on("error", (error) => {
      finishReject(
        new BridgeError(
          "DEPENDENCY_UNAVAILABLE",
          `Cannot start OpenCut headless: ${error.message}`
        )
      );
    });
    child.on("close", (code) => {
      if (settled) {
        return;
      }
      clearTimeout(timer);
      options.signal?.removeEventListener("abort", onAbort);
      if (reportedError) {
        finishReject(reportedError);
      } else if (code !== 0 || result === undefined) {
        const detail = stderr.trim().split(NEWLINE_PATTERN).at(-1);
        finishReject(
          new BridgeError(
            "INTERNAL_ERROR",
            detail
              ? `OpenCut headless failed: ${detail}`
              : "OpenCut headless returned no result"
          )
        );
      } else {
        settled = true;
        resolvePromise(result);
      }
    });
    child.stdin.end(JSON.stringify(request));
  });
};

const ownedTemporaryPaths = (
  config: BridgeConfig,
  request: HeadlessRequest,
  requestId: string
) => {
  if (
    (request.operation === "render_preview" ||
      request.operation === "render_preview_range") &&
    typeof request.projectId === "string" &&
    config.projectsDirectory
  ) {
    return [
      join(
        config.projectsDirectory,
        request.projectId,
        `.opencut-work-${requestId}`
      ),
      join(
        config.projectsDirectory,
        request.projectId,
        "previews",
        `.opencut-${requestId}.${request.operation === "render_preview_range" ? "mp4" : "png"}`
      ),
    ];
  }
  if (
    request.operation === "export_video" &&
    typeof request.relativePath === "string" &&
    config.exportsDirectory
  ) {
    return [
      ...(config.projectsDirectory && typeof request.projectId === "string"
        ? [
            join(
              config.projectsDirectory,
              request.projectId,
              `.opencut-work-${requestId}`
            ),
          ]
        : []),
      join(
        config.exportsDirectory,
        dirname(request.relativePath),
        `.opencut-${requestId}.mp4`
      ),
    ];
  }
  return [];
};

const terminateProcessTree = (child: ChildProcessWithoutNullStreams) => {
  if (!child.pid || child.killed) {
    return;
  }
  if (process.platform === "win32") {
    const killer = spawn("taskkill", ["/pid", String(child.pid), "/t", "/f"], {
      stdio: "ignore",
      windowsHide: true,
    });
    killer.unref();
  } else {
    try {
      process.kill(-child.pid, "SIGTERM");
    } catch {
      child.kill();
    }
  }
};

export const errorBody = (error: unknown) => {
  if (error instanceof BridgeError) {
    return {
      code: error.code,
      failedStage: error.failedStage,
      ffmpegExitCode: error.ffmpegExitCode,
      ffmpegStderrExcerpt: error.ffmpegStderrExcerpt
        ? redactPaths(error.ffmpegStderrExcerpt, 4096, true)
        : null,
      message: redactPaths(error.message),
      retryable: error.retryable,
    };
  }
  if (error instanceof z.ZodError) {
    return {
      code: "INTERNAL_ERROR",
      failedStage: null,
      ffmpegExitCode: null,
      ffmpegStderrExcerpt: null,
      message: "OpenCut returned data that failed contract validation",
      retryable: false,
    };
  }
  return {
    code: "INTERNAL_ERROR",
    failedStage: null,
    ffmpegExitCode: null,
    ffmpegStderrExcerpt: null,
    message: "OpenCut bridge encountered an internal error",
    retryable: false,
  };
};

const redactPaths = (message: string, limit = 500, tail = false) => {
  const sanitized = message
    .replace(/[A-Za-z]:[\\/][^\r\n]*?(?=: |$)/gm, "[path]")
    .replace(/(^|[\s='"([])\/[^\r\n]*?(?=: |$)/gm, "$1[path]");
  const characters = [...sanitized];
  return (tail ? characters.slice(-limit) : characters.slice(0, limit)).join(
    ""
  );
};
