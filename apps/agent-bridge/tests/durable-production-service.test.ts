import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, expect, it, vi } from "vitest";
import fixture from "../../../contracts/durable-production-requests-v1.json";
import { BridgeError, type HeadlessClient } from "../src/headless";
import type { HeadlessRequest } from "../src/headless-contract";
import { JobRuntimeGuard } from "../src/job-runtime";
import { JobRegistry } from "../src/jobs";
import { DURABLE_LIMITS } from "../src/production-requests/contract";
import { confirmedCoreError } from "../src/production-requests/core-outcome";
import { DurableProductionService } from "../src/production-requests/service";
import {
  DurableProductionStore,
  type StoreFaultPoint,
} from "../src/production-requests/store";

const PRIVATE_FIELDS = /jobId|token|relativePath|previews/;
const roots: string[] = [];
afterEach(() => {
  for (const root of roots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});
const requestId = "00000000-0000-4000-8000-000000000001";
const runtimeId = "00000000-0000-4000-8000-000000000003";
const admission = {
  intent: {
    kind: "preview_frame",
    request: {
      expectedRevision: 4,
      projectId: "project-original",
      timeMs: 250,
    },
  },
  requestId,
  version: 1,
};
const identity = { requestId, version: 1 };
const artifact = {
  mimeType: "image/png",
  relativePath: "previews/result.png",
  sizeBytes: 123,
  warnings: [],
};
const setup = (
  fault?: (point: StoreFaultPoint) => void,
  registry?: JobRegistry
) => {
  const root = mkdtempSync(join(tmpdir(), "opencut-durable-service-"));
  roots.push(root);
  const clock = { now: 1000 };
  const producer = vi.fn(() => Promise.resolve(artifact));
  const call = vi.fn((request: HeadlessRequest) => {
    if (request.operation === "get_state") {
      return Promise.resolve({
        project: { revision: 4, settings: { height: 360, width: 640 } },
      });
    }
    return producer();
  });
  const headless = { call } as unknown as HeadlessClient;
  const guard = new JobRuntimeGuard(root);
  const store = new DurableProductionStore(root, {
    fault,
    runtimeGuard: guard,
  });
  const jobs = registry ?? new JobRegistry({ now: () => clock.now });
  const service = new DurableProductionService({
    headless,
    jobs,
    now: () => clock.now,
    runtimeId,
    store,
  });
  const close = async () => {
    await service.close();
    guard.release();
  };
  return {
    call,
    clock,
    close,
    guard,
    headless,
    jobs,
    producer,
    root,
    service,
    store,
  };
};
const settled = async (service: DurableProductionService) => {
  await vi.waitFor(async () => {
    expect((await service.get(identity)).request?.state).not.toBe("running");
  });
};

it("matches independently authored queued/running/completed snapshots without persisting tokens", async () => {
  const { clock, close, producer, service, store } = setup();
  const admitted = await service.admit(admission);
  expect(admitted).toEqual({
    created: true,
    request: fixture.completePublicSnapshots.queued,
    version: 1,
  });
  expect(producer).not.toHaveBeenCalled();
  clock.now = 1100;
  const started = await service.start(identity);
  expect(started.request).toEqual(fixture.completePublicSnapshots.running);
  clock.now = 1200;
  await settled(service);
  expect((await service.get(identity)).request).toEqual(
    fixture.completePublicSnapshots.completed
  );
  expect(JSON.stringify(store.snapshot())).not.toMatch(PRIVATE_FIELDS);
  await close();
});

it("looks up retained identities before core preflight, including terminal starts", async () => {
  const { call, close, service } = setup();
  await service.admit(admission);
  call.mockRejectedValue(new BridgeError("PROJECT_NOT_FOUND", "gone"));
  call.mockClear();
  expect((await service.admit(admission)).created).toBe(false);
  await expect(
    service.admit({
      ...admission,
      intent: {
        ...admission.intent,
        request: { ...admission.intent.request, timeMs: 251 },
      },
    })
  ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
  expect(call).not.toHaveBeenCalled();
  await service.cancelQueued(identity);
  expect((await service.start(identity)).request.state).toBe("cancelled");
  expect(call).not.toHaveBeenCalled();
  await close();
});

it("serializes concurrent explicit starts and never dispatches twice", async () => {
  const { close, producer, service } = setup();
  await service.admit(admission);
  const starts = await Promise.all(
    Array.from({ length: 12 }, () => service.start(identity))
  );
  expect(new Set(starts.map((value) => value.job?.jobId)).size).toBe(1);
  await settled(service);
  expect(producer).toHaveBeenCalledTimes(1);
  await close();
});

it("preserves queued intent across close/reopen without adopting old jobs", async () => {
  const { close, guard, headless, root, service } = setup();
  await service.admit(admission);
  await close();
  const store = new DurableProductionStore(root, { runtimeGuard: guard });
  const fresh = new DurableProductionService({
    headless,
    jobs: new JobRegistry(),
    now: () => 1000,
    store,
  });
  expect((await fresh.get(identity)).request).toEqual(
    fixture.completePublicSnapshots.queued
  );
  expect((await fresh.list({ version: 1 })).requests).toHaveLength(1);
  await fresh.close();
});

it.each([true, false])(
  "distinguishes validated core errors from same-code transport errors (confirmed=%s)",
  async (confirmed) => {
    const { close, guard, producer, service } = setup();
    const error = new BridgeError("REVISION_CONFLICT", "stale", true);
    producer.mockRejectedValue(confirmed ? confirmedCoreError(error) : error);
    await service.admit(admission);
    await service.start(identity);
    await settled(service);
    const record = (await service.get(identity)).request;
    expect(record?.state).toBe(confirmed ? "failed" : "uncertain");
    expect(record?.uncertainty).toBe(confirmed ? null : "dispatch_unconfirmed");
    if (confirmed) {
      await close();
    } else {
      await expect(service.close()).rejects.toMatchObject({
        code: "DEPENDENCY_UNAVAILABLE",
      });
      expect(guard.owned).toBe(true);
    }
  }
);

it("records a proven preparation failure before producer entry", async () => {
  const { call, close, producer, service, store } = setup();
  await service.admit(admission);
  call.mockRejectedValue(new BridgeError("REVISION_CONFLICT", "changed", true));
  expect((await service.start(identity)).request).toMatchObject({
    outcome: { code: "REVISION_CONFLICT", kind: "error", retryable: true },
    state: "failed",
  });
  expect(store.snapshot()?.records[0]?.state).toBe("failed");
  expect(producer).not.toHaveBeenCalled();
  await close();
});

it("keeps a confirmed native success when separate preview retention fails", async () => {
  const jobs = new JobRegistry({
    disposePreview: () => Promise.resolve(),
    maxPreviewBytes: 1,
  });
  const { close, service } = setup(undefined, jobs);
  await service.admit(admission);
  const started = await service.start(identity);
  await settled(service);
  expect((await service.get(identity)).request?.state).toBe("completed");
  expect(started.job && jobs.get(started.job.jobId).status).toBe("failed");
  await close();
});

it("does not manufacture an uncommitted admission during close recovery", async () => {
  const faults = new Set<StoreFaultPoint>(["before_rename"]);
  const { close, producer, service } = setup((point) => {
    if (faults.has(point)) {
      throw new Error("fault");
    }
  });
  await expect(service.admit(admission)).rejects.toMatchObject({
    code: "DEPENDENCY_UNAVAILABLE",
  });
  expect(await service.get(identity)).toMatchObject({
    request: null,
    storage: { reason: "metadata_unconfirmed" },
  });
  faults.clear();
  await close();
  expect(producer).not.toHaveBeenCalled();
});

it("does not dispatch an unacknowledged running claim and closes it as proven pre-entry failure", async () => {
  const faults = new Set<StoreFaultPoint>();
  const { close, producer, service, store } = setup((point) => {
    if (faults.has(point)) {
      throw new Error("fault");
    }
  });
  await service.admit(admission);
  faults.add("before_readback");
  await expect(service.start(identity)).rejects.toMatchObject({
    code: "DEPENDENCY_UNAVAILABLE",
  });
  expect((await service.get(identity)).request).toMatchObject({
    confirmedState: "running",
    state: "uncertain",
    uncertainty: "metadata_unconfirmed",
  });
  expect(producer).not.toHaveBeenCalled();
  faults.clear();
  await close();
  expect(store.snapshot()?.records[0]).toMatchObject({
    outcome: { code: "DEPENDENCY_UNAVAILABLE" },
    state: "failed",
  });
});

it("keeps terminal write loss uncertain until one metadata-only close succeeds", async () => {
  const faults = new Set<StoreFaultPoint>();
  const { close, producer, service, store } = setup((point) => {
    if (faults.has(point)) {
      throw new Error("fault");
    }
  });
  await service.admit(admission);
  producer.mockImplementation(() => {
    faults.add("before_rename");
    return Promise.resolve(artifact);
  });
  await service.start(identity);
  await settled(service);
  expect((await service.get(identity)).request).toMatchObject({
    outcome: null,
    state: "uncertain",
    uncertainty: "metadata_unconfirmed",
  });
  faults.clear();
  await close();
  expect(store.snapshot()?.records[0]).toMatchObject({
    outcome: { kind: "rendered", sizeBytes: 123 },
    state: "completed",
  });
  expect(producer).toHaveBeenCalledTimes(1);
});

it("uses admission age for retention and rollback never expires future metadata", async () => {
  const { clock, close, service } = setup();
  await service.admit(admission);
  clock.now = 999;
  expect((await service.get(identity)).request?.state).toBe("queued");
  clock.now = 1000 + DURABLE_LIMITS.retentionMs;
  expect(await service.get(identity)).toMatchObject({
    request: null,
    retention: { absenceMeaning: "unknown", expiredCount: 1 },
  });
  await close();
});

it("does not dispatch when the queued intent expires during deferred preflight", async () => {
  const { call, clock, close, producer, service, store } = setup();
  await service.admit(admission);
  let release: () => void = () => undefined;
  const preflight = new Promise<void>((resolve) => {
    release = resolve;
  });
  let entered: () => void = () => undefined;
  const preparing = new Promise<void>((resolve) => {
    entered = resolve;
  });
  call.mockImplementation(async () => {
    entered();
    await preflight;
    return { project: { revision: 4, settings: { height: 360, width: 640 } } };
  });
  const start = service.start(identity);
  await preparing;
  clock.now = 1000 + DURABLE_LIMITS.retentionMs;
  release();
  await expect(start).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
  expect(producer).not.toHaveBeenCalled();
  expect(store.snapshot()?.records[0]?.state).toBe("queued");
  await close();
});

it("never returns a cancelled record that expired before its transaction", async () => {
  const { clock, close, service, store } = setup();
  await service.admit(admission);
  let reads = 0;
  Object.defineProperty(clock, "now", {
    get: () => {
      reads += 1;
      return reads === 1 ? 1000 : 1000 + DURABLE_LIMITS.retentionMs;
    },
  });
  await expect(service.cancelQueued(identity)).rejects.toMatchObject({
    code: "VALIDATION_FAILED",
  });
  expect(store.snapshot()?.records[0]?.state).toBe("queued");
  await close();
});

it("merges multiple already-active native outcomes in one metadata-only close generation", async () => {
  const faults = new Set<StoreFaultPoint>();
  const { close, producer, service, store } = setup((point) => {
    if (faults.has(point)) {
      throw new Error("fault");
    }
  });
  const second = {
    ...identity,
    requestId: "00000000-0000-4000-8000-000000000004",
  };
  await service.admit(admission);
  await service.admit({ ...admission, requestId: second.requestId });
  const completions: (() => void)[] = [];
  producer.mockImplementation(
    () => new Promise((resolve) => completions.push(() => resolve(artifact)))
  );
  await service.start(identity);
  await service.start(second);
  expect(store.snapshot()?.generation).toBe(4);
  faults.add("before_rename");
  for (const complete of completions) {
    complete();
  }
  await vi.waitFor(async () => {
    const records = (await service.list({ version: 1 })).requests;
    expect(records.map((record) => record.uncertainty)).toEqual([
      "metadata_unconfirmed",
      "metadata_unconfirmed",
    ]);
  });
  faults.clear();
  await close();
  expect(store.snapshot()?.generation).toBe(5);
  expect(store.snapshot()?.records.map((record) => record.state)).toEqual([
    "completed",
    "completed",
  ]);
  expect(producer).toHaveBeenCalledTimes(2);
});

it("keeps an active expired producer visible and charged until actual settlement", async () => {
  const { clock, close, producer, service } = setup();
  let finish: () => void = () => undefined;
  producer.mockImplementation(
    () =>
      new Promise((resolve) => {
        finish = () => resolve(artifact);
      })
  );
  await service.admit(admission);
  const started = await service.start(identity);
  clock.now = 1000 + DURABLE_LIMITS.retentionMs;
  expect((await service.get(identity)).request?.state).toBe("running");
  expect((await service.list({ version: 1 })).retention.expiredCount).toBe(0);
  finish();
  await vi.waitFor(async () => {
    expect((await service.get(identity)).request).toBeNull();
  });
  expect(started.job).not.toBeNull();
  await close();
});

it("does not discard expired pending evidence through logical get/list filtering", async () => {
  const faults = new Set<StoreFaultPoint>();
  const { clock, close, producer, service, store } = setup((point) => {
    if (faults.has(point)) {
      throw new Error("fault");
    }
  });
  await service.admit(admission);
  producer.mockImplementation(() => {
    faults.add("before_rename");
    return Promise.resolve(artifact);
  });
  await service.start(identity);
  await settled(service);
  clock.now = 1000 + DURABLE_LIMITS.retentionMs;
  expect(await service.get(identity)).toMatchObject({
    request: null,
    storage: { reason: "metadata_unconfirmed", writable: false },
  });
  expect(store.snapshot()?.records[0]?.state).toBe("running");
  faults.clear();
  await close();
  expect(store.snapshot()?.records).toEqual([]);
  expect(producer).toHaveBeenCalledTimes(1);
});

it.each(["start", "cancel"] as const)(
  "rechecks retained state under newly acquired ownership before %s",
  async (operation) => {
    const original = setup();
    await original.service.admit(admission);
    await original.close();
    const guard = new JobRuntimeGuard(original.root);
    const acquire = guard.acquire.bind(guard);
    const once = new Set(["write"]);
    vi.spyOn(guard, "acquire").mockImplementation(() => {
      if (once.delete("write")) {
        original.store.update((records) =>
          records.map((row) => ({
            ...row,
            outcome: { format: "png", kind: "rendered", sizeBytes: 123 },
            runtimeId,
            state: "completed",
            updatedAtMs: 1200,
          }))
        );
        original.store.close();
        original.guard.release();
      }
      acquire();
    });
    const store = new DurableProductionStore(original.root, {
      runtimeGuard: guard,
    });
    const service = new DurableProductionService({
      headless: original.headless,
      jobs: new JobRegistry(),
      now: () => 1500,
      store,
    });
    original.call.mockClear();
    if (operation === "start") {
      expect((await service.start(identity)).request.state).toBe("completed");
    } else {
      await expect(service.cancelQueued(identity)).rejects.toMatchObject({
        code: "VALIDATION_FAILED",
      });
    }
    expect(original.call).not.toHaveBeenCalled();
    expect(store.snapshot()?.records[0]?.state).toBe("completed");
    await service.close();
    guard.release();
  }
);

it.each([
  {
    code: "UnknownDurableCoreError".repeat(1024),
    name: "oversized code",
    retryable: false,
  },
  {
    code: "REVISION_CONFLICT",
    name: "inconsistent retryability",
    retryable: false,
  },
])(
  "does not retain unsupported post-entry error payloads ($name)",
  async ({ code, retryable }) => {
    const { guard, producer, service, store } = setup();
    await service.admit(admission);
    producer.mockRejectedValue(
      confirmedCoreError(new BridgeError(code, "native error", retryable))
    );
    await service.start(identity);
    await settled(service);
    expect((await service.get(identity)).request).toMatchObject({
      outcome: null,
      state: "uncertain",
      uncertainty: "dispatch_unconfirmed",
    });
    expect(store.snapshot()?.records[0]?.state).toBe("running");
    await expect(service.close()).rejects.toMatchObject({
      code: "DEPENDENCY_UNAVAILABLE",
    });
    expect(guard.owned).toBe(true);
  }
);

it.each([
  {
    code: "UnknownDurableCoreError".repeat(1024),
    name: "oversized code",
    retryable: false,
  },
  {
    code: "REVISION_CONFLICT",
    name: "inconsistent retryability",
    retryable: false,
  },
])(
  "uses a bounded canonical fallback for demonstrated pre-entry errors ($name)",
  async ({ code, retryable }) => {
    const { call, close, producer, service } = setup();
    await service.admit(admission);
    call.mockRejectedValue(new BridgeError(code, "preflight error", retryable));
    expect((await service.start(identity)).request).toMatchObject({
      outcome: { code: "INTERNAL_ERROR", kind: "error", retryable: false },
      state: "failed",
    });
    expect(producer).not.toHaveBeenCalled();
    await close();
  }
);

it.each(["admission", "reservation"] as const)(
  "persists a proven JobRegistry %s rejection without producer entry",
  async (phase) => {
    const jobs =
      phase === "admission"
        ? new JobRegistry({ maxCount: 0 })
        : new JobRegistry({
            disposePreview: () => Promise.resolve(),
            maxPreviewCount: 0,
          });
    const { close, producer, service } = setup(undefined, jobs);
    await service.admit(admission);
    await service.start(identity);
    await settled(service);
    expect((await service.get(identity)).request).toMatchObject({
      outcome: { code: "JOB_REGISTRY_FULL", kind: "error", retryable: true },
      state: "failed",
    });
    expect(producer).not.toHaveBeenCalled();
    await close();
  }
);

it("keeps queued cancellation idempotent without refreshing timestamps or catalog generations", async () => {
  const { clock, close, producer, service, store } = setup();
  await service.admit(admission);
  clock.now = 1100;
  const cancelled = await service.cancelQueued(identity);
  const generation = store.snapshot()?.generation;
  clock.now = 1200;
  expect(await service.cancelQueued(identity)).toEqual(cancelled);
  expect(store.snapshot()?.generation).toBe(generation);
  expect(producer).not.toHaveBeenCalled();
  await close();
});
