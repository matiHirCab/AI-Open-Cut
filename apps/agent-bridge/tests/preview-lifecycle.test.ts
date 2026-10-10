import { randomUUID } from "node:crypto";
import { expect, it } from "vitest";
import { JobRegistry } from "../src/jobs";

import type { Job } from "../src/schemas";

const artifact = (sizeBytes = 4) => ({
  mimeType: "image/png",
  relativePath: `previews/preview-${randomUUID()}.png`,
  sizeBytes,
  warnings: [],
});
const tick = () => new Promise<void>((resolve) => setTimeout(resolve, 1));
const done = async (jobs: JobRegistry, id: string): Promise<Job> => {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    const job = jobs.get(id);
    if (job.status === "completed" || job.status === "failed") {
      return job;
    }
    // biome-ignore lint/performance/noAwaitInLoops: Poll asynchronous job settlement.
    await tick();
  }
  throw new Error("job did not finish");
};

it("charges expired cancelled producers and close awaits late disposal", async () => {
  let now = 0;
  let finish!: (value: { artifact: ReturnType<typeof artifact> }) => void;
  let progress!: (value: number) => void;
  const removed: Job[] = [];
  const jobs = new JobRegistry({
    disposePreview: (outputJob) => {
      removed.push(outputJob);
      return Promise.resolve();
    },
    maxCount: 1,
    now: () => now,
    ttlMs: 5,
  });
  const job = jobs.startTask("preview", "project", 8, (context) => {
    progress = context.onProgress;
    return new Promise((resolve) => {
      finish = resolve;
    });
  });
  jobs.cancel(job.jobId);
  expect(jobs.cancel(job.jobId).status).toBe("cancelled");
  progress(1);
  expect(jobs.get(job.jobId).progress).toBe(0);
  now = 6;
  expect(() => jobs.get(job.jobId)).toThrowError(
    expect.objectContaining({ code: "JOB_NOT_FOUND" })
  );
  expect(() =>
    jobs.startTask("preview", "project", 9, async () => ({}))
  ).toThrowError(
    expect.objectContaining({ code: "JOB_REGISTRY_FULL", retryable: true })
  );
  let closed = false;
  const firstClose = jobs.close();
  expect(jobs.close()).toBe(firstClose);
  const closing = firstClose.then(() => {
    closed = true;
  });
  await tick();
  expect(closed).toBe(false);
  finish({ artifact: artifact() });
  await closing;
  expect(removed).toHaveLength(1);
  expect(removed[0]?.revision).toBe(8);
});

it("accepts inclusive count/byte bounds and evicts before replacement", async () => {
  const removed: Job[] = [];
  const jobs = new JobRegistry({
    disposePreview: (outputJob) => {
      removed.push(outputJob);
      return Promise.resolve();
    },
    maxPreviewBytes: 8,
    maxPreviewCount: 2,
  });
  const first = jobs.startTask("preview", "project", 1, async () => ({
    artifact: artifact(),
  }));
  await done(jobs, first.jobId);
  const second = jobs.startTask("preview", "project", 2, async () => ({
    artifact: artifact(),
  }));
  await done(jobs, second.jobId);
  expect(removed).toHaveLength(0);
  const third = jobs.startTask("preview", "project", 3, async () => ({
    artifact: artifact(),
  }));
  await done(jobs, third.jobId);
  expect(removed.map((job) => job.jobId)).toEqual([first.jobId]);
  expect(() => jobs.get(first.jobId)).toThrowError(
    expect.objectContaining({ code: "JOB_NOT_FOUND" })
  );
  expect(jobs.get(second.jobId).revision).toBe(2);
  const large = jobs.startTask("preview", "project", 4, async () => ({
    artifact: artifact(9),
  }));
  expect(await done(jobs, large.jobId)).toMatchObject({
    error: { code: "JOB_REGISTRY_FULL", retryable: true },
    status: "failed",
  });
  expect(removed.at(-1)?.jobId).toBe(large.jobId);
  expect(jobs.get(third.jobId).status).toBe("completed");
  await jobs.close();
});

it("serializes simultaneous completions and charges failed disposal", async () => {
  const disposal = { fail: true };
  const jobs = new JobRegistry({
    disposePreview: () =>
      // biome-ignore lint/suspicious/noUnnecessaryConditions: Failure state changes before shutdown retry.
      disposal.fail
        ? Promise.reject(new Error("private path"))
        : Promise.resolve(),
    maxPreviewCount: 1,
  });
  const first = jobs.startTask("preview", "project", 1, async () => ({
    artifact: artifact(),
  }));
  const second = jobs.startTask("preview", "project", 2, async () => ({
    artifact: artifact(),
  }));
  expect((await done(jobs, first.jobId)).status).toBe("completed");
  expect((await done(jobs, second.jobId)).error?.code).toBe(
    "JOB_REGISTRY_FULL"
  );
  expect(jobs.get(first.jobId).status).toBe("completed");
  disposal.fail = false;
  await jobs.close();
});

it("expires previews without deleting exports or extending TTL on polling", async () => {
  let now = 10;
  const removed: Job[] = [];
  const jobs = new JobRegistry({
    disposePreview: (outputJob) => {
      removed.push(outputJob);
      return Promise.resolve();
    },
    now: () => now,
    ttlMs: 5,
  });
  const preview = jobs.startTask("preview", "project", 1, async () => ({
    artifact: artifact(),
  }));
  await done(jobs, preview.jobId);
  const exported = jobs.startTask("export", "project", 1, async () => ({
    artifact: artifact(),
  }));
  await done(jobs, exported.jobId);
  now = 14;
  expect(jobs.get(preview.jobId).expiresAtMs).toBe(15);
  now = 15;
  expect(() => jobs.get(preview.jobId)).toThrowError(
    expect.objectContaining({ code: "JOB_NOT_FOUND" })
  );
  await tick();
  expect(removed.map((job) => job.jobId)).toEqual([preview.jobId]);
  await jobs.close();
  expect(removed).toHaveLength(1);
});

it.each([
  { maxPreviewBytes: 8, maxPreviewCount: 1 },
  { maxPreviewBytes: 4, maxPreviewCount: 2 },
])("enforces independently limiting preview capacity %j", async (limits) => {
  const removed: string[] = [];
  const jobs = new JobRegistry({
    ...limits,
    disposePreview: (outputJob) => {
      removed.push(outputJob.jobId);
      return Promise.resolve();
    },
  });
  const first = jobs.startTask("preview", "project", 1, async () => ({
    artifact: artifact(),
  }));
  await done(jobs, first.jobId);
  const second = jobs.startTask("preview", "project", 2, async () => ({
    artifact: artifact(),
  }));
  expect((await done(jobs, second.jobId)).status).toBe("completed");
  expect(removed).toEqual([first.jobId]);
  await jobs.close();
});

it("waits for preview disposal before reclaiming job capacity", async () => {
  let release!: () => void;
  const jobs = new JobRegistry({
    disposePreview: () =>
      new Promise<void>((resolve) => {
        release = resolve;
      }),
    maxCount: 1,
  });
  const first = jobs.startTask("preview", "project", 1, async () => ({
    artifact: artifact(),
  }));
  await done(jobs, first.jobId);
  expect(() =>
    jobs.startTask("preview", "project", 2, async () => ({}))
  ).toThrowError(
    expect.objectContaining({ code: "JOB_REGISTRY_FULL", retryable: true })
  );
  await tick();
  expect(() =>
    jobs.startTask("preview", "project", 2, async () => ({}))
  ).toThrowError(expect.objectContaining({ code: "JOB_REGISTRY_FULL" }));
  release();
  await tick();
  const next = jobs.startTask("preview", "project", 2, async () => ({}));
  await done(jobs, next.jobId);
  await jobs.close();
});

it.each([
  { maxPreviewBytes: 8, maxPreviewCount: 1 },
  { maxPreviewBytes: 4, maxPreviewCount: 2 },
])(
  "blocks persistent cleanup debt before dispatch and retries close safely %j",
  async (limits) => {
    const state = { fail: true, produced: 0 };
    const jobs = new JobRegistry({
      disposePreview: () =>
        // biome-ignore lint/suspicious/noUnnecessaryConditions: Fault state changes for retry.
        state.fail ? Promise.reject(new Error("secret")) : Promise.resolve(),
      ...limits,
    });
    const produce = () => {
      state.produced += 1;
      return Promise.resolve({ artifact: artifact() });
    };
    const first = jobs.startTask("preview", "project", 1, produce);
    await done(jobs, first.jobId);
    for (let n = 0; n < 4; n += 1) {
      const replacement = jobs.startTask("preview", "project", 1, produce);
      // biome-ignore lint/performance/noAwaitInLoops: Exercise sequential fault retries.
      expect((await done(jobs, replacement.jobId)).error?.code).toBe(
        "JOB_REGISTRY_FULL"
      );
    }
    expect(state.produced).toBe(1);
    const other = jobs.startTask("export", "project", 1, async () => ({}));
    expect((await done(jobs, other.jobId)).status).toBe("completed");
    await expect(jobs.close()).rejects.toMatchObject({
      code: "VALIDATION_FAILED",
    });
    expect(() => jobs.startTask("preview", "project", 1, produce)).toThrowError(
      expect.objectContaining({ code: "BRIDGE_SHUTTING_DOWN" })
    );
    state.fail = false;
    await jobs.close();
  }
);

it("keeps cancelled unsettled producer slots charged until actual completion", async () => {
  let release!: (completion: { artifact: ReturnType<typeof artifact> }) => void;
  let produced = 0;
  const jobs = new JobRegistry({
    disposePreview: () => Promise.resolve(),
    maxPreviewCount: 1,
  });
  const first = jobs.startTask("preview", "project", 1, () => {
    produced += 1;
    return new Promise((resolve) => {
      release = resolve;
    });
  });
  jobs.cancel(first.jobId);
  const second = jobs.startTask("preview", "project", 1, () => {
    produced += 1;
    return Promise.resolve({ artifact: artifact() });
  });
  expect((await done(jobs, second.jobId)).error?.code).toBe(
    "JOB_REGISTRY_FULL"
  );
  expect(produced).toBe(1);
  release({ artifact: artifact() });
  await tick();
  const third = jobs.startTask("preview", "project", 1, () => {
    produced += 1;
    return Promise.resolve({ artifact: artifact() });
  });
  expect((await done(jobs, third.jobId)).status).toBe("completed");
  expect(produced).toBe(2);
  await jobs.close();
});

it("retains oversized output debt and recovers before the next producer", async () => {
  const state = { fail: true, produced: 0 };
  const jobs = new JobRegistry({
    disposePreview: () => {
      // biome-ignore lint/suspicious/noUnnecessaryConditions: Inject failure then allow recovery.
      return state.fail
        ? Promise.reject(new Error("private"))
        : Promise.resolve();
    },
    maxPreviewBytes: 4,
  });
  const first = jobs.startTask("preview", "project", 1, async () => ({
    artifact: artifact(5),
  }));
  expect((await done(jobs, first.jobId)).error?.code).toBe("JOB_REGISTRY_FULL");
  const produce = () => {
    state.produced += 1;
    return Promise.resolve({ artifact: artifact() });
  };
  const second = jobs.startTask("preview", "project", 1, produce);
  expect((await done(jobs, second.jobId)).error?.code).toBe(
    "JOB_REGISTRY_FULL"
  );
  expect(state.produced).toBe(0);
  state.fail = false;
  const third = jobs.startTask("preview", "project", 1, produce);
  expect((await done(jobs, third.jobId)).status).toBe("completed");
  expect(state.produced).toBe(1);
  await jobs.close();
});
