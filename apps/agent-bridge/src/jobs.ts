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
  audioAnalysis?: Job["audioAnalysis"];
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

/** Internal task evidence; public job status alone cannot prove native effects. */
export interface JobSettlement {
  completion: JobCompletion | undefined;
  entered: boolean;
  error: unknown;
  job: Job;
}
export type JobSettlementObserver = (value: JobSettlement) => Promise<void>;

interface JobEntry {
  cancellable: boolean;
  controller: AbortController;
  disposalDebt?: boolean;
  job: Job;
  ownedPreview?: JobCompletion["artifact"];
  previewReserved?: boolean;
  promise?: Promise<void>;
  removing?: boolean;
  settled: boolean;
}

interface JobRegistryOptions {
  beforeStart?: () => void;
  disposePreview?: (job: Job) => Promise<void>;
  headless?: HeadlessClient;
  logger?: Logger;
  maxCount?: number;
  maxPreviewBytes?: number;
  maxPreviewCount?: number;
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

const isPreview = (job: Job) =>
  job.kind === "preview" || job.kind === "preview_range";

export class JobRegistry {
  readonly #beforeStart: (() => void) | undefined;
  readonly #disposePreview: ((job: Job) => Promise<void>) | undefined;
  readonly #maxPreviewBytes: number;
  readonly #maxPreviewCount: number;
  #retention = Promise.resolve();
  #closing: Promise<void> | undefined;
  readonly #headless: HeadlessClient | undefined;
  readonly #jobs = new Map<string, JobEntry>();
  readonly #maxCount: number;
  readonly #logger: Logger;
  readonly #now: () => number;
  readonly #ttlMs: number;
  readonly #lifecycle = { closed: false };

  constructor(options: JobRegistryOptions = {}) {
    this.#beforeStart = options.beforeStart;
    this.#disposePreview = options.disposePreview;
    this.#maxPreviewBytes = options.maxPreviewBytes ?? 67_108_864;
    this.#maxPreviewCount = options.maxPreviewCount ?? 32;
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
          | "render_review_range"
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
    task: (context: JobTaskContext) => Promise<JobCompletion>,
    observe?: JobSettlementObserver
  ) {
    this.#ensureAdmission();
    this.#beforeStart?.();
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
      settled: false,
    };
    // Observed producers must never throw a response validation error after
    // admission. Existing callers retain their original synchronous snapshot.
    const observedSnapshot = observe
      ? jobSchema.parse({ ...entry.job, status: "running" })
      : undefined;
    this.#jobs.set(jobId, entry);
    this.#logger.info("job.admitted", {
      jobId,
      operation: kind,
      status: "queued",
    });
    entry.promise = this.#run(entry, task, observe);
    if (observedSnapshot) {
      return observedSnapshot;
    }
    return jobSchema.parse(entry.job);
  }

  get(jobId: string) {
    this.#cleanup();
    const entry = this.#jobs.get(jobId);
    if (!entry || this.#expired(entry)) {
      throw new BridgeError("JOB_NOT_FOUND", "Job was not found");
    }
    return jobSchema.parse(entry.job);
  }

  cancel(jobId: string) {
    this.#cleanup();
    const entry = this.#jobs.get(jobId);
    if (!entry || this.#expired(entry)) {
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

  close() {
    this.#closing ??= this.#close().catch((error: unknown) => {
      this.#closing = undefined;
      throw error;
    });
    return this.#closing;
  }

  async #close() {
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
    for (const entry of this.#jobs.values()) {
      if (entry.ownedPreview) {
        this.#queueRemoval(entry);
      }
    }
    await this.#retention;
    if ([...this.#jobs.values()].some((entry) => entry.ownedPreview)) {
      throw new BridgeError(
        "VALIDATION_FAILED",
        "Preview artifact cleanup failed"
      );
    }
  }

  async #run(
    entry: JobEntry,
    task: (context: JobTaskContext) => Promise<JobCompletion>,
    observe?: JobSettlementObserver
  ) {
    let completion: JobCompletion | undefined;
    let entered = false;
    let failure: unknown;
    let startedAt = entry.job.createdAtMs;
    try {
      this.#update(entry, { status: "running" });
      startedAt = this.#now();
      this.#logger.info("job.started", {
        jobId: entry.job.jobId,
        operation: entry.job.kind,
        queueWaitMs: startedAt - entry.job.createdAtMs,
        status: "running",
      });
      if (isPreview(entry.job) && this.#disposePreview) {
        const admission = this.#reservePreview(entry);
        if (admission) {
          await admission;
        }
      }
      if (entry.controller.signal.aborted) {
        return;
      }
      entered = true;
      completion = await task({
        jobId: entry.job.jobId,
        markNonCancellable: () => {
          entry.cancellable = false;
        },
        onProgress: (progress) => this.#progress(entry, progress),
        signal: entry.controller.signal,
      });
      const { artifact } = completion;
      if (artifact && isPreview(entry.job) && this.#disposePreview) {
        await this.#serialize(async () => {
          await this.#retainPreview(entry, artifact);
        });
      }
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
      failure = error;
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
    } finally {
      if (observe) {
        try {
          await observe({
            completion,
            entered,
            error: failure,
            job: { ...entry.job },
          });
        } catch {
          try {
            this.#logger.error("job.settlement.failed", {
              code: "DEPENDENCY_UNAVAILABLE",
              jobId: entry.job.jobId,
            });
          } catch {
            // Diagnostics cannot prevent releasing an observed settlement.
          }
        }
      }
      entry.previewReserved = false;
      entry.settled = true;
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
        .filter(
          (entry) => isTerminal(entry.job) && entry.settled && !entry.removing
        )
        .sort((left, right) => left.job.updatedAtMs - right.job.updatedAtMs);
      if (!oldest) {
        throw new BridgeError(
          "JOB_REGISTRY_FULL",
          "OpenCut job registry is full",
          true
        );
      }
      if (oldest.ownedPreview) {
        this.#queueRemoval(oldest);
        throw new BridgeError(
          "JOB_REGISTRY_FULL",
          "OpenCut job registry is full",
          true
        );
      }
      this.#jobs.delete(oldest.job.jobId);
    }
  }

  #expired(entry: JobEntry) {
    return (
      entry.job.expiresAtMs !== null && entry.job.expiresAtMs <= this.#now()
    );
  }

  #cleanup() {
    for (const entry of this.#jobs.values()) {
      if (entry.settled && this.#expired(entry)) {
        if (entry.ownedPreview) {
          this.#queueRemoval(entry);
        } else {
          this.#jobs.delete(entry.job.jobId);
        }
      }
    }
  }

  #serialize(action: () => Promise<void>) {
    const pending = this.#retention.then(action);
    this.#retention = pending.catch(() => undefined);
    return pending;
  }

  #queueRemoval(entry: JobEntry) {
    if (entry.removing) {
      return;
    }
    entry.removing = true;
    this.#serialize(async () => {
      if (await this.#discardPreview(entry)) {
        this.#jobs.delete(entry.job.jobId);
      }
      entry.removing = false;
    }).catch(() => undefined);
  }

  async #discardPreview(entry: JobEntry) {
    if (!entry.ownedPreview) {
      return true;
    }
    try {
      await this.#disposePreview?.({
        ...entry.job,
        artifact: entry.ownedPreview,
      });
      entry.ownedPreview = undefined;
      entry.disposalDebt = false;
      return true;
    } catch {
      entry.disposalDebt = true;
      this.#logger.error("job.preview.disposal.failed", {
        code: "VALIDATION_FAILED",
        jobId: entry.job.jobId,
      });
      return false;
    }
  }

  #previewFull() {
    return new BridgeError(
      "JOB_REGISTRY_FULL",
      "OpenCut preview artifact retention is full",
      true
    );
  }

  #previewPressure() {
    let count = 0;
    let remaining = this.#maxPreviewBytes;
    for (const candidate of this.#jobs.values()) {
      if (candidate.previewReserved || candidate.ownedPreview) {
        count += 1;
      }
      remaining -= candidate.ownedPreview?.sizeBytes ?? 0;
      if (candidate.disposalDebt || remaining <= 0) {
        return true;
      }
    }
    return count >= this.#maxPreviewCount;
  }

  #reservePreview(entry: JobEntry): Promise<void> | undefined {
    if (!this.#previewPressure()) {
      entry.previewReserved = true;
      return;
    }
    return this.#serialize(async () => {
      // Recover debt first. Only actual disposal releases ownership and charge.
      for (const candidate of this.#jobs.values()) {
        if (candidate.disposalDebt) {
          // biome-ignore lint/performance/noAwaitInLoops: Serialized owned debt recovery.
          if (!(await this.#discardPreview(candidate))) {
            throw this.#previewFull();
          }
          this.#jobs.delete(candidate.job.jobId);
        }
      }
      while (this.#previewPressure()) {
        const [oldest] = [...this.#jobs.values()]
          .filter((candidate) => candidate.settled && candidate.ownedPreview)
          .sort((left, right) => left.job.updatedAtMs - right.job.updatedAtMs);
        // biome-ignore lint/performance/noAwaitInLoops: Settle eviction before replacement dispatch.
        if (!(oldest && (await this.#discardPreview(oldest)))) {
          throw this.#previewFull();
        }
        this.#jobs.delete(oldest.job.jobId);
      }
      entry.previewReserved = true;
    });
  }

  async #retainPreview(entry: JobEntry, artifact: JobCompletion["artifact"]) {
    if (!artifact) {
      return;
    }
    entry.ownedPreview = artifact;
    const full = () =>
      new BridgeError(
        "JOB_REGISTRY_FULL",
        "OpenCut preview artifact retention is full",
        true
      );
    if (entry.controller.signal.aborted) {
      await this.#discardPreview(entry);
      return;
    }
    if (artifact.sizeBytes > this.#maxPreviewBytes) {
      await this.#discardPreview(entry);
      throw full();
    }
    const retained = () =>
      [...this.#jobs.values()].filter(
        (candidate) => candidate !== entry && candidate.ownedPreview
      );
    const overLimit = () => {
      const entries = retained();
      // Subtract from the budget so summation cannot overflow a safe integer.
      let remaining = this.#maxPreviewBytes - artifact.sizeBytes;
      for (const candidate of entries) {
        remaining -= candidate.ownedPreview?.sizeBytes ?? 0;
        if (remaining < 0) {
          return true;
        }
      }
      return entries.length >= this.#maxPreviewCount;
    };
    while (overLimit()) {
      const [oldest] = retained()
        .filter((candidate) => candidate.settled)
        .sort((left, right) => left.job.updatedAtMs - right.job.updatedAtMs);
      // biome-ignore lint/performance/noAwaitInLoops: Dispose each old output before releasing its budget.
      if (!(oldest && (await this.#discardPreview(oldest)))) {
        await this.#discardPreview(entry);
        throw full();
      }
      this.#jobs.delete(oldest.job.jobId);
    }
    // Cancellation can arrive during filesystem eviction.
    if (entry.controller.signal.aborted) {
      await this.#discardPreview(entry);
    }
  }

  #isClosed() {
    return this.#lifecycle.closed;
  }
}

export const jobResultSchema = z.object({ job: jobSchema }).strict();
