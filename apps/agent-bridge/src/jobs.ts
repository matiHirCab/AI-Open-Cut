import { randomUUID } from "node:crypto";

import { z } from "zod/v4";

import { BridgeError, errorBody, type HeadlessClient } from "./headless";
import type { HeadlessRequest } from "./headless-contract";
import { type Logger, NOOP_LOGGER } from "./logger";
import {
  artifactSchema,
  type Job,
  jobSchema,
  type ttsResultSchema,
} from "./schemas";

interface JobCompletion {
  artifact?: ReturnType<typeof artifactSchema.parse>;
  result?: ReturnType<typeof ttsResultSchema.parse>;
  speechPreview?: Job["speechPreview"];
  transcriptionPreview?: Job["transcriptionPreview"];
}

export interface JobTaskContext {
  jobId?: string;
  markNonCancellable: () => void;
  onProgress: (progress: number) => void;
  signal: AbortSignal;
}

interface JobEntry {
  cancellable: boolean;
  controller: AbortController;
  job: Job;
  promise?: Promise<void>;
}

interface JobRegistryOptions {
  headless?: HeadlessClient;
  logger?: Logger;
  maxCount?: number;
  now?: () => number;
  ttlMs?: number;
}

interface ArtifactConflictError {
  generatedArtifact: { expiresAtMs: number; token: string };
}

const isTerminal = (job: Job) =>
  job.status === "completed" ||
  job.status === "failed" ||
  job.status === "cancelled";

export class JobRegistry {
  readonly #headless: HeadlessClient | undefined;
  readonly #jobs = new Map<string, JobEntry>();
  readonly #maxCount: number;
  readonly #logger: Logger;
  readonly #now: () => number;
  readonly #ttlMs: number;
  readonly #lifecycle = { closed: false };

  constructor(options: JobRegistryOptions = {}) {
    this.#headless = options.headless;
    this.#maxCount = options.maxCount ?? 1000;
    this.#logger = options.logger ?? NOOP_LOGGER;
    this.#now = options.now ?? Date.now;
    this.#ttlMs = options.ttlMs ?? 3_600_000;
  }

  start(
    kind: Job["kind"],
    projectId: string,
    revision: number,
    request: Extract<
      HeadlessRequest,
      {
        operation:
          | "render_preview"
          | "render_preview_range"
          | "render_draft_preview"
          | "export_video";
      }
    >
  ) {
    if (!this.#headless) {
      throw new BridgeError("INTERNAL_ERROR", "Headless client is unavailable");
    }
    const headless = this.#headless;
    return this.startTask(kind, projectId, revision, async (context) => ({
      artifact: await headless.call(request, artifactSchema, {
        onProgress: context.onProgress,
        signal: context.signal,
      }),
    }));
  }

  startTask(
    kind: Job["kind"],
    projectId: string,
    revision: number,
    task: (context: JobTaskContext) => Promise<JobCompletion>
  ) {
    this.#ensureAdmission();
    const jobId = randomUUID();
    const now = this.#now();
    const entry: JobEntry = {
      cancellable: true,
      controller: new AbortController(),
      job: {
        createdAtMs: now,
        expiresAtMs: null,
        jobId,
        kind,
        persistence: "process",
        progress: 0,
        projectId,
        revision,
        status: "queued",
        updatedAtMs: now,
      },
    };
    this.#jobs.set(jobId, entry);
    this.#logger.info("job.admitted", {
      jobId,
      operation: kind,
      status: "queued",
    });
    entry.promise = this.#run(entry, task);
    return jobSchema.parse(entry.job);
  }

  get(jobId: string) {
    this.#cleanup();
    const entry = this.#jobs.get(jobId);
    if (!entry) {
      throw new BridgeError("JOB_NOT_FOUND", "Job was not found");
    }
    return jobSchema.parse(entry.job);
  }

  cancel(jobId: string) {
    this.#cleanup();
    const entry = this.#jobs.get(jobId);
    if (!entry) {
      throw new BridgeError("JOB_NOT_FOUND", "Job was not found");
    }
    if (entry.job.status === "cancelled") {
      return jobSchema.parse(entry.job);
    }
    if (isTerminal(entry.job) || !entry.cancellable) {
      throw new BridgeError(
        "JOB_NOT_CANCELLABLE",
        "Job can no longer be cancelled"
      );
    }
    entry.controller.abort();
    this.#finishCancelled(entry);
    return jobSchema.parse(entry.job);
  }

  async close() {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: lifecycle state mutates across calls.
    if (this.#isClosed()) {
      return;
    }
    this.#lifecycle.closed = true;
    for (const entry of this.#jobs.values()) {
      if (!isTerminal(entry.job) && entry.cancellable) {
        entry.controller.abort();
        this.#finishCancelled(entry);
      }
    }
    await Promise.allSettled(
      [...this.#jobs.values()].flatMap((entry) =>
        entry.promise ? [entry.promise] : []
      )
    );
  }

  async #run(
    entry: JobEntry,
    task: (context: JobTaskContext) => Promise<JobCompletion>
  ) {
    if (entry.controller.signal.aborted) {
      return;
    }
    this.#update(entry, { status: "running" });
    const startedAt = this.#now();
    this.#logger.info("job.started", {
      jobId: entry.job.jobId,
      operation: entry.job.kind,
      queueWaitMs: startedAt - entry.job.createdAtMs,
      status: "running",
    });
    try {
      const completion = await task({
        jobId: entry.job.jobId,
        markNonCancellable: () => {
          entry.cancellable = false;
        },
        onProgress: (progress) => this.#progress(entry, progress),
        signal: entry.controller.signal,
      });
      if (entry.job.status !== "cancelled") {
        this.#finish(entry, {
          ...completion,
          progress: 1,
          status: "completed",
        });
        this.#logger.info("job.completed", {
          durationMs: this.#now() - startedAt,
          jobId: entry.job.jobId,
          operation: entry.job.kind,
          status: "completed",
        });
      }
    } catch (error) {
      if (entry.job.status === "cancelled" || entry.controller.signal.aborted) {
        this.#finishCancelled(entry);
        return;
      }
      const conflict = error as Partial<ArtifactConflictError>;
      this.#finish(entry, {
        error: errorBody(error),
        generatedArtifact: conflict.generatedArtifact,
        status: "failed",
      });
      this.#logger.error("job.failed", {
        code: errorBody(error).code,
        durationMs: this.#now() - startedAt,
        jobId: entry.job.jobId,
        operation: entry.job.kind,
        status: "failed",
      });
    }
  }

  #progress(entry: JobEntry, value: number) {
    if (!Number.isFinite(value) || isTerminal(entry.job)) {
      return;
    }
    const progress = Math.max(
      entry.job.progress,
      Math.min(1, Math.max(0, value))
    );
    this.#update(entry, { progress });
  }

  #finishCancelled(entry: JobEntry) {
    if (entry.job.status === "cancelled") {
      return;
    }
    this.#finish(entry, {
      error: {
        code: "JOB_CANCELLED",
        failedStage: null,
        ffmpegExitCode: null,
        ffmpegStderrExcerpt: null,
        message: "Job was cancelled",
        retryable: true,
      },
      status: "cancelled",
    });
  }

  #finish(entry: JobEntry, changes: Partial<Job>) {
    const now = this.#now();
    entry.job = {
      ...entry.job,
      ...changes,
      expiresAtMs: now + this.#ttlMs,
      updatedAtMs: now,
    };
  }

  #update(entry: JobEntry, changes: Partial<Job>) {
    entry.job = { ...entry.job, ...changes, updatedAtMs: this.#now() };
  }

  #ensureAdmission() {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: lifecycle state mutates across calls.
    if (this.#isClosed()) {
      throw new BridgeError(
        "BRIDGE_SHUTTING_DOWN",
        "OpenCut bridge is shutting down",
        true
      );
    }
    this.#cleanup();
    while (this.#jobs.size >= this.#maxCount) {
      const [oldest] = [...this.#jobs.values()]
        .filter((entry) => isTerminal(entry.job))
        .sort((left, right) => left.job.updatedAtMs - right.job.updatedAtMs);
      if (!oldest) {
        throw new BridgeError(
          "JOB_REGISTRY_FULL",
          "OpenCut job registry is full",
          true
        );
      }
      this.#jobs.delete(oldest.job.jobId);
    }
  }

  #cleanup() {
    const now = this.#now();
    for (const [jobId, entry] of this.#jobs) {
      if (entry.job.expiresAtMs !== null && entry.job.expiresAtMs <= now) {
        this.#jobs.delete(jobId);
      }
    }
  }

  #isClosed() {
    return this.#lifecycle.closed;
  }
}

export const jobResultSchema = z.object({ job: jobSchema }).strict();
