import { randomUUID } from "node:crypto";
import type { z } from "zod/v4";
import { errorDefinition } from "../errors";
import { BridgeError, errorBody, type HeadlessClient } from "../headless";
import type { JobRegistry, JobSettlement } from "../jobs";
import { artifactSchema, type Job } from "../schemas";
import {
  canonicalJson,
  DURABLE_LIMITS,
  type DurableOutcome,
  durableOutputSchemas,
  identityInputSchema,
  intentDigest,
  listInputSchema,
  type PublicRecord,
  parseAdmission,
  publicRecordSchema,
  type StoredRecord,
} from "./contract";
import { isConfirmedCoreError } from "./core-outcome";
import { type PreparedProduction, prepareProduction } from "./prepare";
import type { DurableProductionStore } from "./store";

interface Options {
  headless: HeadlessClient;
  jobs: JobRegistry;
  now?: () => number;
  runtimeId?: string;
  store: DurableProductionStore;
}
const unavailable = () =>
  new BridgeError(
    "DEPENDENCY_UNAVAILABLE",
    "Durable production storage is unavailable",
    false
  );
const invalid = () =>
  new BridgeError("VALIDATION_FAILED", "Invalid durable production request");
const terminal = (record: StoredRecord) =>
  !["queued", "running"].includes(record.state);
const outcomeEqual = (left: DurableOutcome | null, right: DurableOutcome) =>
  canonicalJson(left) === canonicalJson(right);
const canonicalErrorPair = ({
  code,
  retryable,
}: {
  code: string;
  retryable: boolean;
}) =>
  Buffer.byteLength(code) <= 128 &&
  errorDefinition(code)?.retryable === retryable;
const failureOutcome = (error: unknown): DurableOutcome => {
  const { code, retryable } = errorBody(error);
  return canonicalErrorPair({ code, retryable })
    ? { code, kind: "error", retryable }
    : { code: "INTERNAL_ERROR", kind: "error", retryable: false };
};
const parse = <T>(schema: z.ZodType<T>, input: unknown): T => {
  const result = schema.safeParse(input);
  if (!result.success) {
    throw invalid();
  }
  return result.data;
};

/** Opt-in application inventory. Execution handles and artifacts remain process-local. */
export class DurableProductionService {
  readonly #options: Options;
  readonly #runtime: string;
  readonly #now: () => number;
  readonly #active = new Set<string>();
  readonly #handles = new Map<string, string>();
  readonly #pending = new Map<string, DurableOutcome>();
  readonly #pendingCreatedAt = new Map<string, number>();
  readonly #claimDebt = new Set<string>();
  readonly #uncertain = new Set<string>();
  #queue: Promise<unknown> = Promise.resolve();
  readonly #lifecycle = new Set<"closing">();

  constructor(options: Options) {
    this.#options = options;
    this.#runtime = options.runtimeId ?? randomUUID();
    this.#now = options.now ?? Date.now;
  }

  admit(input: unknown) {
    const { requestId, intent } = parseAdmission(input);
    return this.#serialize(async () => {
      this.#open();
      const view = this.#view();
      const existing = view.records.find(
        (candidate) => candidate.requestId === requestId
      );
      if (existing) {
        if (existing.intentSha256 !== intentDigest(intent)) {
          throw invalid();
        }
        return durableOutputSchemas.admit.parse({
          created: false,
          request: this.#project(existing),
          version: 1,
        });
      }
      this.#writable();
      this.#options.store.acquire();
      const retained = this.#view().records.find(
        (candidate) => candidate.requestId === requestId
      );
      if (retained) {
        if (retained.intentSha256 !== intentDigest(intent)) {
          throw invalid();
        }
        return durableOutputSchemas.admit.parse({
          created: false,
          request: this.#project(retained),
          version: 1,
        });
      }
      await prepareProduction(this.#options.headless, intent);
      const now = this.#clock();
      const record: StoredRecord = {
        createdAtMs: now,
        intent,
        intentSha256: intentDigest(intent),
        outcome: null,
        requestId,
        runtimeId: null,
        state: "queued",
        updatedAtMs: now,
      };
      this.#options.store.update((records) => [
        ...this.#retained(records, now),
        record,
      ]);
      return durableOutputSchemas.admit.parse({
        created: true,
        request: this.#project(record),
        version: 1,
      });
    });
  }

  get(input: unknown) {
    const { requestId } = parse(identityInputSchema, input);
    return this.#serialize(() => {
      const view = this.#view();
      const record = view.records.find(
        (candidate) => candidate.requestId === requestId
      );
      return durableOutputSchemas.get.parse({
        request: record ? this.#project(record) : null,
        retention: view.retention,
        storage: view.storage,
        version: 1,
      });
    });
  }

  list(input: unknown) {
    parse(listInputSchema, input);
    return this.#serialize(() => {
      const view = this.#view();
      const records = [...view.records].sort(
        (left, right) =>
          left.createdAtMs - right.createdAtMs ||
          left.requestId.localeCompare(right.requestId)
      );
      return durableOutputSchemas.list.parse({
        requests: records.map((record) => this.#project(record)),
        retention: view.retention,
        storage: view.storage,
        version: 1,
      });
    });
  }

  start(input: unknown) {
    const { requestId } = parse(identityInputSchema, input);
    return this.#serialize(async () => {
      this.#open();
      const record = this.#view().records.find(
        (candidate) => candidate.requestId === requestId
      );
      if (!record) {
        throw invalid();
      }
      if (record.state !== "queued" || this.#pending.has(requestId)) {
        return this.#startResult(record);
      }
      this.#writable();
      this.#options.store.acquire();
      const guarded = this.#queuedUnderGuard(record);
      if (guarded.state !== "queued") {
        return this.#startResult(guarded);
      }
      let prepared: PreparedProduction;
      try {
        prepared = await prepareProduction(
          this.#options.headless,
          record.intent
        );
      } catch (error) {
        this.#remember(record, failureOutcome(error));
        this.#flushKnown();
        return this.#startResult(this.#requiredRecord(requestId));
      }
      const claimTime = this.#clock();
      const running: StoredRecord = {
        ...record,
        runtimeId: this.#runtime,
        state: "running",
        updatedAtMs: Math.max(record.updatedAtMs, claimTime),
      };
      try {
        const committed = this.#options.store.update((records) => {
          const retained = this.#retained(records, claimTime);
          const target = retained.find(
            (candidate) => candidate.requestId === requestId
          );
          if (!target || canonicalJson(target) !== canonicalJson(record)) {
            throw invalid();
          }
          return retained.map((candidate) =>
            candidate.requestId === requestId ? running : candidate
          );
        });
        if (
          !committed.records.some(
            (candidate) => canonicalJson(candidate) === canonicalJson(running)
          )
        ) {
          throw unavailable();
        }
      } catch (error) {
        if (this.#options.store.hasDebt) {
          this.#claimDebt.add(requestId);
        }
        throw error;
      }
      this.#dispatch(running, prepared);
      return this.#startResult(this.#requiredRecord(requestId));
    });
  }

  cancelQueued(input: unknown) {
    const { requestId } = parse(identityInputSchema, input);
    return this.#serialize(() => {
      this.#open();
      const record = this.#view().records.find(
        (candidate) => candidate.requestId === requestId
      );
      if (!(record && ["queued", "cancelled"].includes(record.state))) {
        throw invalid();
      }
      if (record.state === "cancelled") {
        return durableOutputSchemas.cancelQueued.parse({
          request: this.#project(record),
          version: 1,
        });
      }
      this.#writable();
      this.#options.store.acquire();
      const guarded = this.#queuedUnderGuard(record);
      if (guarded.state === "cancelled") {
        return durableOutputSchemas.cancelQueued.parse({
          request: this.#project(guarded),
          version: 1,
        });
      }
      if (guarded.state !== "queued") {
        throw invalid();
      }
      const cancelTime = this.#clock();
      if (this.#retained([record], cancelTime).length === 0) {
        throw invalid();
      }
      const cancelled: StoredRecord = {
        ...record,
        outcome: { kind: "cancelled" },
        state: "cancelled",
        updatedAtMs: Math.max(record.updatedAtMs, cancelTime),
      };
      this.#options.store.update((records) =>
        this.#retained(records, cancelTime).map((candidate) =>
          candidate.requestId === requestId ? cancelled : candidate
        )
      );
      return durableOutputSchemas.cancelQueued.parse({
        request: this.#project(cancelled),
        version: 1,
      });
    });
  }

  beginClose() {
    this.#lifecycle.add("closing");
  }

  async close() {
    this.beginClose();
    await this.#options.jobs.close();
    await this.#serialize(() => {
      if (
        this.#options.store.hasDebt ||
        this.#pending.size ||
        this.#claimDebt.size
      ) {
        const forgotten = new Set<string>();
        this.#options.store.recover((catalog) => {
          const records = catalog?.records ?? [];
          for (const requestId of this.#claimDebt) {
            const record = records.find(
              (candidate) => candidate.requestId === requestId
            );
            if (
              record?.state === "running" &&
              record.runtimeId === this.#runtime
            ) {
              this.#remember(record, failureOutcome(unavailable()));
            }
          }
          const next = this.#mergeKnown(records);
          const retained = new Set(next.map((record) => record.requestId));
          for (const record of records) {
            if (!retained.has(record.requestId)) {
              forgotten.add(record.requestId);
            }
          }
          return next;
        });
        this.#confirmPending(this.#options.store.snapshot()?.records ?? []);
        this.#claimDebt.clear();
        for (const requestId of forgotten) {
          this.#forget(requestId);
        }
      }
      if (this.#pending.size || this.#uncertain.size) {
        throw unavailable();
      }
      this.#options.store.close();
    });
  }

  #dispatch(record: StoredRecord, prepared: PreparedProduction) {
    this.#active.add(record.requestId);
    try {
      const job = this.#options.jobs.startTask(
        prepared.kind,
        record.intent.request.projectId,
        record.intent.request.expectedRevision,
        async ({ signal, onProgress }) => {
          const artifact = await this.#options.headless.call(
            prepared.request,
            artifactSchema,
            { onProgress, signal }
          );
          // Validate returned metadata even if an injected client did not apply its schema.
          const parsed = artifactSchema.parse(artifact);
          const expected =
            record.intent.kind === "preview_frame" ? "image/png" : "video/mp4";
          if (parsed.mimeType !== expected) {
            throw unavailable();
          }
          return { artifact: parsed };
        },
        (settlement) => this.#settle(record, settlement)
      );
      this.#handles.set(record.requestId, job.jobId);
    } catch (error) {
      this.#active.delete(record.requestId);
      this.#remember(record, failureOutcome(error));
      this.#flushKnown();
    }
  }

  #settle(record: StoredRecord, settlement: JobSettlement) {
    const { requestId } = record;
    return this.#serialize(() => {
      try {
        const outcome = this.#knownOutcome(settlement);
        if (outcome) {
          this.#remember(record, outcome);
          this.#flushKnown();
        } else {
          this.#uncertain.add(requestId);
        }
      } finally {
        this.#active.delete(requestId);
      }
    });
  }

  #knownOutcome(settlement: JobSettlement): DurableOutcome | null {
    const artifact = settlement.completion?.artifact;
    if (artifact) {
      return {
        format: artifact.mimeType === "image/png" ? "png" : "mp4",
        kind: "rendered",
        sizeBytes: artifact.sizeBytes,
      };
    }
    if (!settlement.entered) {
      if (settlement.job.status === "cancelled") {
        return { kind: "cancelled" };
      }
      return failureOutcome(settlement.error);
    }
    return isConfirmedCoreError(settlement.error) &&
      canonicalErrorPair(settlement.error)
      ? failureOutcome(settlement.error)
      : null;
  }

  #flushKnown() {
    if (this.#options.store.hasDebt) {
      return;
    }
    try {
      const committed = this.#options.store.update((records) =>
        this.#mergeKnown(records)
      );
      this.#confirmPending(committed.records);
    } catch {
      // The pending native evidence remains bounded and blocks all new mutations.
    }
  }

  #mergeKnown(records: readonly StoredRecord[]) {
    const now = this.#clock();
    const existing = new Set(records.map((record) => record.requestId));
    for (const requestId of this.#pending.keys()) {
      if (!(existing.has(requestId) || this.#expiredPending(requestId, now))) {
        throw unavailable();
      }
    }
    return this.#retained(records, now).map((record) => {
      const outcome = this.#pending.get(record.requestId);
      if (!outcome) {
        return record;
      }
      if (terminal(record)) {
        if (!outcomeEqual(record.outcome, outcome)) {
          throw unavailable();
        }
        return record;
      }
      const states = {
        cancelled: "cancelled",
        error: "failed",
        rendered: "completed",
      } as const;
      const state = states[outcome.kind];
      return {
        ...record,
        outcome,
        runtimeId: this.#runtime,
        state,
        updatedAtMs: Math.max(record.updatedAtMs, now),
      } satisfies StoredRecord;
    });
  }

  #confirmPending(records: readonly StoredRecord[]) {
    if (this.#pending.size === 0) {
      return;
    }
    for (const record of records) {
      const pending = this.#pending.get(record.requestId);
      if (
        pending &&
        terminal(record) &&
        outcomeEqual(record.outcome, pending)
      ) {
        this.#forget(record.requestId);
      }
    }
    if (this.#pending.size === 0) {
      return;
    }
    const present = new Set(records.map((record) => record.requestId));
    const now = this.#clock();
    for (const requestId of this.#pending.keys()) {
      if (!present.has(requestId) && this.#expiredPending(requestId, now)) {
        this.#forget(requestId);
      }
    }
  }

  #remember(record: StoredRecord, outcome: DurableOutcome) {
    this.#pending.set(record.requestId, outcome);
    this.#pendingCreatedAt.set(record.requestId, record.createdAtMs);
  }

  #forget(requestId: string) {
    this.#pending.delete(requestId);
    this.#pendingCreatedAt.delete(requestId);
  }

  #expiredPending(requestId: string, now: number) {
    const createdAt = this.#pendingCreatedAt.get(requestId);
    return (
      createdAt !== undefined &&
      !this.#active.has(requestId) &&
      now >= createdAt + DURABLE_LIMITS.retentionMs
    );
  }

  #view() {
    const catalog = this.#options.store.snapshot();
    const storage = this.#options.store.status();
    if (!storage.readable) {
      throw unavailable();
    }
    const records = this.#retained(catalog?.records ?? [], this.#clock());
    const retained = new Set(records.map((record) => record.requestId));
    for (const requestId of this.#handles.keys()) {
      if (!retained.has(requestId)) {
        this.#handles.delete(requestId);
      }
    }
    this.#confirmPending(catalog?.records ?? []);
    return {
      records,
      retention: {
        absenceMeaning: "unknown" as const,
        expiredCount: (catalog?.records.length ?? 0) - records.length,
        maxAgeMs: DURABLE_LIMITS.retentionMs,
      },
      storage:
        this.#pending.size || this.#claimDebt.size || this.#uncertain.size
          ? {
              readable: true,
              reason: "metadata_unconfirmed" as const,
              writable: false,
            }
          : storage,
    };
  }

  #project(record: StoredRecord): PublicRecord {
    let uncertainty: PublicRecord["uncertainty"] = null;
    if (!terminal(record)) {
      if (
        this.#pending.has(record.requestId) ||
        (record.state === "running" && this.#claimDebt.has(record.requestId))
      ) {
        uncertainty = "metadata_unconfirmed";
      } else if (
        record.state === "running" &&
        record.runtimeId !== this.#runtime
      ) {
        uncertainty = "previous_runtime";
      } else if (this.#uncertain.has(record.requestId)) {
        uncertainty = "dispatch_unconfirmed";
      }
    }
    return publicRecordSchema.parse({
      confirmedState: record.state,
      createdAtMs: record.createdAtMs,
      expiresAtMs: record.createdAtMs + DURABLE_LIMITS.retentionMs,
      intent: record.intent,
      outcome: uncertainty ? null : record.outcome,
      requestId: record.requestId,
      state: uncertainty ? "uncertain" : record.state,
      uncertainty,
      updatedAtMs: record.updatedAtMs,
      version: 1,
    });
  }

  #startResult(record: StoredRecord) {
    let job: Job | null = null;
    const jobId = this.#handles.get(record.requestId);
    if (jobId) {
      try {
        job = this.#options.jobs.get(jobId);
      } catch {
        /* An expired handle is never reconstructed. */
      }
    }
    return durableOutputSchemas.start.parse({
      job,
      request: this.#project(record),
      version: 1,
    });
  }

  #requiredRecord(requestId: string) {
    const record = this.#options.store
      .snapshot()
      ?.records.find((candidate) => candidate.requestId === requestId);
    if (!record) {
      throw unavailable();
    }
    return record;
  }

  #queuedUnderGuard(previous: StoredRecord) {
    const current = this.#view().records.find(
      (candidate) => candidate.requestId === previous.requestId
    );
    if (!current || current.intentSha256 !== previous.intentSha256) {
      throw invalid();
    }
    if (
      current.state === "queued" &&
      canonicalJson(current) !== canonicalJson(previous)
    ) {
      throw invalid();
    }
    return current;
  }

  #retained(records: readonly StoredRecord[], now: number) {
    return records.filter(
      (record) =>
        this.#active.has(record.requestId) ||
        now < record.createdAtMs + DURABLE_LIMITS.retentionMs
    );
  }

  #clock() {
    const now = this.#now();
    if (
      !Number.isSafeInteger(now) ||
      now < 0 ||
      now > Number.MAX_SAFE_INTEGER - DURABLE_LIMITS.retentionMs
    ) {
      throw unavailable();
    }
    return now;
  }

  #writable() {
    if (
      this.#options.store.hasDebt ||
      this.#pending.size ||
      this.#claimDebt.size ||
      this.#uncertain.size
    ) {
      throw unavailable();
    }
  }

  #open() {
    if (this.#lifecycle.has("closing")) {
      throw new BridgeError(
        "BRIDGE_SHUTTING_DOWN",
        "OpenCut bridge is shutting down",
        true
      );
    }
  }

  #serialize<T>(action: () => T | Promise<T>): Promise<T> {
    const pending = this.#queue.then(action);
    this.#queue = pending.catch(() => undefined);
    return pending;
  }
}
