import { type ChildProcessWithoutNullStreams, spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import { TextDecoder } from "node:util";
import { z } from "zod/v4";
import type { BridgeConfig } from "./config";
import type { HeadlessCallOptions } from "./headless";
import type { HeadlessRequest } from "./headless-contract";
import { BridgeError, eventSchema } from "./headless-events";

export const WORKER_LINE_BYTES = 16 * 1024 * 1024;
export const WORKER_STARTUP_MS = 5000;
export const workerReadySchema = z
  .object({ protocolVersion: z.literal(1), type: z.literal("ready") })
  .strict();
export const workerEventSchema = z
  .object({ event: eventSchema, requestId: z.string() })
  .strict();
const RENDER_OPERATIONS = new Set([
  "render_preview",
  "render_preview_range",
  "render_draft_preview",
  "export_video",
]);

interface Pending {
  dispatched: boolean;
  event: (event: z.infer<typeof eventSchema>) => void;
  id: string;
  reject: (error: BridgeError) => void;
  terminal: boolean;
}

/** One reserved render at a time; the client dispatches overlap through one-shot execution. */
export class RenderWorker {
  busy = false;
  retired = false;
  terminated = false;
  readonly #config: BridgeConfig;
  readonly #cleanup: (request: HeadlessRequest, id: string) => Promise<void>;
  #child: ChildProcessWithoutNullStreams | undefined;
  #pending: Pending | undefined;
  #ready = false;
  #readyResolve: (() => void) | undefined;
  #readyReject: ((error: BridgeError) => void) | undefined;
  #closed: Promise<void> | undefined;
  #chunks: Buffer[] = [];
  #bufferBytes = 0;
  #stopping: Promise<void> | undefined;

  constructor(
    config: BridgeConfig,
    cleanup: (request: HeadlessRequest, id: string) => Promise<void>
  ) {
    this.#config = config;
    this.#cleanup = cleanup;
  }

  #isReady(): boolean {
    return this.#ready;
  }
  #isRetired(): boolean {
    return this.retired;
  }
  #isTerminated(): boolean {
    return this.terminated;
  }
  #isBusy(): boolean {
    return this.busy;
  }

  static accepts(request: HeadlessRequest): boolean {
    return RENDER_OPERATIONS.has(request.operation);
  }

  #fail(error: BridgeError) {
    this.retired = true;
    this.#readyReject?.(error);
    this.#pending?.reject(error);
    this.#stop().catch(() => undefined);
  }

  #start(): Promise<void> {
    if (this.#isReady()) {
      return Promise.resolve();
    }
    return new Promise((resolve, reject) => {
      this.#readyResolve = resolve;
      this.#readyReject = reject;
      const child = spawn(
        this.#config.headlessPath,
        [...this.#config.headlessArguments, "--render-worker"],
        {
          detached: process.platform !== "win32",
          env: { ...this.#config.environment },
          stdio: ["pipe", "pipe", "pipe"],
          windowsHide: true,
        }
      );
      this.#child = child;
      this.#closed = new Promise<void>((closed) => {
        child.once("close", () => {
          this.terminated = true;
          this.retired = true;
          closed();
          const error = new BridgeError(
            this.#isReady() ? "INTERNAL_ERROR" : "DEPENDENCY_UNAVAILABLE",
            "OpenCut render worker exited"
          );
          this.#readyReject?.(error);
          this.#pending?.reject(error);
        });
      });
      child.on("error", () =>
        this.#fail(
          new BridgeError(
            "DEPENDENCY_UNAVAILABLE",
            "Cannot start OpenCut render worker"
          )
        )
      );
      child.stdin.on("error", () =>
        this.#fail(
          new BridgeError(
            "INTERNAL_ERROR",
            "Cannot write render worker request"
          )
        )
      );
      child.stderr.resume();
      child.stdout.on("data", (chunk: Buffer) => this.#data(chunk));
    });
  }

  #data(chunk: Buffer) {
    if (this.#isRetired()) {
      return;
    }
    let offset = 0;
    try {
      while (offset < chunk.length) {
        const newline = chunk.indexOf(10, offset);
        const end = newline === -1 ? chunk.length : newline;
        if (this.#bufferBytes + end - offset > WORKER_LINE_BYTES) {
          throw new Error("oversized line");
        }
        this.#chunks.push(chunk.subarray(offset, end));
        this.#bufferBytes += end - offset;
        offset = end + 1;
        if (newline === -1) {
          break;
        }
        const value: unknown = JSON.parse(
          new TextDecoder("utf-8", { fatal: true }).decode(
            Buffer.concat(this.#chunks, this.#bufferBytes)
          )
        );
        this.#chunks = [];
        this.#bufferBytes = 0;
        if (!this.#isReady()) {
          workerReadySchema.parse(value);
          this.#ready = true;
          this.#readyResolve?.();
          this.#readyResolve = undefined;
          this.#readyReject = undefined;
          continue;
        }
        const envelope = workerEventSchema.parse(value);
        const pending = this.#pending;
        if (
          !pending?.dispatched ||
          pending.id !== envelope.requestId ||
          pending.terminal
        ) {
          throw new Error("unexpected event");
        }
        pending.terminal = envelope.event.type !== "progress";
        pending.event(envelope.event);
      }
    } catch {
      this.#fail(
        new BridgeError(
          this.#isReady() ? "INTERNAL_ERROR" : "DEPENDENCY_UNAVAILABLE",
          "OpenCut returned malformed worker output"
        )
      );
    }
  }

  async call<Output>(
    request: HeadlessRequest,
    schema: z.ZodType<Output>,
    options: HeadlessCallOptions
  ): Promise<Output> {
    if (this.#isBusy() || this.#isRetired()) {
      throw new BridgeError("INTERNAL_ERROR", "Render worker is unavailable");
    }
    this.busy = true;
    const id = options.requestId ?? randomUUID();
    let timer: ReturnType<typeof setTimeout> | undefined;
    let startup: ReturnType<typeof setTimeout> | undefined;
    const abort = () =>
      this.#fail(
        new BridgeError(
          "JOB_CANCELLED",
          "OpenCut operation was cancelled",
          true
        )
      );
    try {
      const line = Buffer.from(
        `${JSON.stringify({ request, requestId: id })}\n`
      );
      if (line.length - 1 > WORKER_LINE_BYTES) {
        throw new BridgeError(
          "INVALID_ARGUMENT",
          "Render worker request exceeds limit"
        );
      }
      const outcome = new Promise<Output>((resolve, reject) => {
        this.#pending = {
          dispatched: false,
          event: (event) => {
            if (event.type === "progress") {
              options.onProgress?.(event.progress);
            } else if (event.type === "error") {
              reject(
                new BridgeError(
                  event.error.code,
                  event.error.message,
                  event.error.retryable,
                  undefined,
                  event.error
                )
              );
            } else {
              const parsed = schema.parse(event.result);
              // Defer success until every event in this chunk has been checked.
              queueMicrotask(() => {
                if (!this.#isRetired()) {
                  resolve(parsed);
                }
              });
            }
          },
          id,
          reject,
          terminal: false,
        };
      });
      // Handle rejection immediately, including cancellation during worker startup.
      const startupFailure = outcome.then(() => undefined);
      startupFailure.catch(() => undefined);
      timer = setTimeout(
        () =>
          this.#fail(
            new BridgeError(
              "HEADLESS_TIMEOUT",
              "OpenCut headless request timed out",
              true
            )
          ),
        options.timeoutMs ?? this.#config.headlessRequestTimeoutMs
      );
      options.signal?.addEventListener("abort", abort, { once: true });
      if (options.signal?.aborted) {
        abort();
      }
      if (!this.#isRetired()) {
        if (!this.#isReady()) {
          startup = setTimeout(
            () =>
              this.#fail(
                new BridgeError(
                  "DEPENDENCY_UNAVAILABLE",
                  "Render worker readiness timed out"
                )
              ),
            WORKER_STARTUP_MS
          );
        }
        await Promise.race([this.#start(), startupFailure]);
        clearTimeout(startup);
        if (!this.#isRetired()) {
          if (this.#pending) {
            this.#pending.dispatched = true;
          }
          this.#child?.stdin.write(line);
        }
      }
      return await outcome;
    } finally {
      clearTimeout(timer);
      clearTimeout(startup);
      options.signal?.removeEventListener("abort", abort);
      await this.#finish(request, id);
    }
  }

  async #finish(request: HeadlessRequest, id: string) {
    if (this.#isRetired()) {
      await this.#stop();
      if (this.#isTerminated()) {
        await this.#cleanup(request, id);
      } else {
        this.#closed
          ?.then(() => this.#cleanup(request, id))
          .finally(() => {
            this.busy = false;
          })
          .catch(() => undefined);
      }
    }
    this.#pending = undefined;
    if (!this.#isRetired() || this.#isTerminated()) {
      this.busy = false;
    }
  }

  #stop(): Promise<void> {
    if (this.#stopping) {
      return this.#stopping;
    }
    this.retired = true;
    const child = this.#child;
    if (!child || (this.#isTerminated() && process.platform === "win32")) {
      this.terminated = true;
      return Promise.resolve();
    }
    const termination = (async () => {
      if (child.pid && process.platform === "win32") {
        await new Promise<void>((resolve) => {
          const killer = spawn(
            "taskkill",
            ["/pid", String(child.pid), "/t", "/f"],
            { stdio: "ignore", windowsHide: true }
          );
          killer.once("error", () => resolve());
          killer.once("close", () => resolve());
        });
      } else if (child.pid) {
        try {
          process.kill(-child.pid, "SIGKILL");
        } catch {
          child.kill("SIGKILL");
        }
      }
      await this.#closed;
    })();
    // Failed OS termination must not hold the caller forever. The worker stays
    // retired and its files stay untouched until exit has actually been observed.
    this.#stopping = new Promise<void>((resolve) => {
      const deadline = setTimeout(resolve, WORKER_STARTUP_MS);
      termination.then(
        () => {
          clearTimeout(deadline);
          resolve();
        },
        () => {
          clearTimeout(deadline);
          resolve();
        }
      );
    });
    return this.#stopping;
  }

  close(): Promise<void> {
    this.#fail(
      new BridgeError("JOB_CANCELLED", "OpenCut operation was cancelled", true)
    );
    return this.#stop();
  }
}
