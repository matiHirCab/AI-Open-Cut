import { expect, it, vi } from "vitest";
import { BridgeError } from "../src/headless";
import { JobRegistry, type JobSettlement } from "../src/jobs";
import { NOOP_LOGGER } from "../src/logger";

const artifact = {
  mimeType: "image/png",
  relativePath: "previews/preview.png",
  sizeBytes: 123,
  warnings: [],
};
const gate = () => {
  let release: () => void = () => undefined;
  const promise = new Promise<void>((resolve) => {
    release = resolve;
  });
  return { promise, release };
};

it("keeps observer settlement charged and awaited by graceful close", async () => {
  const observed = gate();
  const release = gate();
  const settled: JobSettlement[] = [];
  const jobs = new JobRegistry({ maxCount: 1 });
  const job = jobs.startTask(
    "export",
    "project",
    4,
    () => Promise.resolve({}),
    async (value) => {
      settled.push(value);
      observed.release();
      await release.promise;
    }
  );
  await observed.promise;
  expect(jobs.get(job.jobId).status).toBe("completed");
  expect(() =>
    jobs.startTask("export", "project", 4, () => Promise.resolve({}))
  ).toThrow(expect.objectContaining({ code: "JOB_REGISTRY_FULL" }));
  let closed = false;
  const closing = jobs.close().then(() => {
    closed = true;
  });
  await Promise.resolve();
  expect(closed).toBe(false);
  expect(settled).toHaveLength(1);
  expect(settled[0]).toMatchObject({
    completion: {},
    entered: true,
    error: undefined,
  });
  release.release();
  await closing;
});

it("observes a real producer result before separate preview retention fails", async () => {
  const settled: JobSettlement[] = [];
  const observed = gate();
  const jobs = new JobRegistry({
    disposePreview: () => Promise.resolve(),
    maxPreviewBytes: 1,
  });
  const job = jobs.startTask(
    "preview",
    "project",
    4,
    () => Promise.resolve({ artifact }),
    (value) => {
      settled.push(value);
      observed.release();
      return Promise.resolve();
    }
  );
  await observed.promise;
  await jobs.close();
  expect(jobs.get(job.jobId).status).toBe("failed");
  expect(settled).toHaveLength(1);
  expect(settled[0]).toMatchObject({
    completion: { artifact },
    entered: true,
    error: { code: "JOB_REGISTRY_FULL" },
  });
});

it("reports proven pre-entry reservation failure without entering production", async () => {
  const task = vi.fn(() => Promise.resolve({}));
  const settled: JobSettlement[] = [];
  const observed = gate();
  const jobs = new JobRegistry({
    disposePreview: () => Promise.resolve(),
    maxPreviewCount: 0,
  });
  jobs.startTask("preview", "project", 4, task, (value) => {
    settled.push(value);
    observed.release();
    return Promise.resolve();
  });
  await observed.promise;
  await jobs.close();
  expect(task).not.toHaveBeenCalled();
  expect(settled[0]).toMatchObject({ completion: undefined, entered: false });
});

it("keeps post-entry abort distinct from a demonstrated pre-entry cancellation", async () => {
  const entered = gate();
  const settled: JobSettlement[] = [];
  const jobs = new JobRegistry();
  const job = jobs.startTask(
    "export",
    "project",
    4,
    ({ signal }) =>
      new Promise((_, reject) => {
        signal.addEventListener("abort", () =>
          reject(new BridgeError("JOB_CANCELLED", "cancelled"))
        );
        entered.release();
      }),
    (value) => {
      settled.push(value);
      return Promise.resolve();
    }
  );
  await entered.promise;
  jobs.cancel(job.jobId);
  await jobs.close();
  expect(settled[0]).toMatchObject({
    completion: undefined,
    entered: true,
    error: { code: "JOB_CANCELLED" },
  });
});

it("does not rewrite a completed ephemeral result when an optional observer fails", async () => {
  const observed = gate();
  const jobs = new JobRegistry();
  const job = jobs.startTask(
    "export",
    "project",
    4,
    () => Promise.resolve({}),
    () => {
      observed.release();
      return Promise.reject(new Error("metadata unavailable"));
    }
  );
  await observed.promise;
  await jobs.close();
  expect(jobs.get(job.jobId).status).toBe("completed");
});

it("validates observed admission before a producer can enter", async () => {
  const task = vi.fn(() => Promise.resolve({}));
  const observer = vi.fn(() => Promise.resolve());
  const jobs = new JobRegistry({ now: () => Number.NaN });
  expect(() =>
    jobs.startTask("export", "project", 4, task, observer)
  ).toThrow();
  expect(task).not.toHaveBeenCalled();
  expect(observer).not.toHaveBeenCalled();
  await jobs.close();
});

it("delivers captured evidence even when later clock values invalidate the public job", async () => {
  const observed = gate();
  const clock = { now: 1000 };
  const values: JobSettlement[] = [];
  const jobs = new JobRegistry({ now: () => clock.now });
  jobs.startTask(
    "export",
    "project",
    4,
    () => {
      clock.now = Number.NaN;
      return Promise.resolve({ artifact });
    },
    (value) => {
      values.push(value);
      observed.release();
      return Promise.resolve();
    }
  );
  await observed.promise;
  expect(values[0]).toMatchObject({ completion: { artifact }, entered: true });
  await jobs.close();
});

it("reports a startup failure before task entry even before normal running bookkeeping", async () => {
  const task = vi.fn(() => Promise.resolve({}));
  const observed = gate();
  const values: JobSettlement[] = [];
  let calls = 0;
  const jobs = new JobRegistry({
    now: () => {
      calls += 1;
      if (calls === 2) {
        throw new Error("clock unavailable");
      }
      return 1000;
    },
  });
  jobs.startTask("export", "project", 4, task, (value) => {
    values.push(value);
    observed.release();
    return Promise.resolve();
  });
  await observed.promise;
  expect(task).not.toHaveBeenCalled();
  expect(values[0]).toMatchObject({
    completion: undefined,
    entered: false,
    job: { status: "failed" },
  });
  await jobs.close();
});

it("releases capacity when both the observer and its diagnostic writer fail", async () => {
  const logged = gate();
  const jobs = new JobRegistry({
    logger: {
      ...NOOP_LOGGER,
      error: () => {
        logged.release();
        throw new Error("diagnostic writer unavailable");
      },
    },
    maxCount: 1,
  });
  const job = jobs.startTask(
    "export",
    "project",
    4,
    () => Promise.resolve({}),
    () => Promise.reject(new Error("observer unavailable"))
  );
  await logged.promise;
  await Promise.resolve();
  expect(jobs.get(job.jobId).status).toBe("completed");
  expect(() =>
    jobs.startTask("export", "project", 4, () => Promise.resolve({}))
  ).not.toThrow();
  await jobs.close();
});
