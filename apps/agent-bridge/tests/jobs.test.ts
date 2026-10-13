import { expect, it } from "vitest";
import PREPARATION_PROGRESS from "../../../contracts/render-preparation-progress-v1.json";

import { BridgeError } from "../src/headless";
import { JobRegistry } from "../src/jobs";

const waitForTerminal = async (
  jobs: JobRegistry,
  jobId: string,
  attemptsRemaining = 100
): Promise<ReturnType<JobRegistry["get"]>> => {
  const job = jobs.get(jobId);
  if (job.status === "completed" || job.status === "failed") {
    return job;
  }
  if (attemptsRemaining === 0) {
    throw new Error("job did not finish");
  }
  await new Promise((resolvePromise) => setTimeout(resolvePromise, 1));
  return waitForTerminal(jobs, jobId, attemptsRemaining - 1);
};

it("stores a completed TTS edit result", async () => {
  const jobs = new JobRegistry();
  const queued = jobs.startTask("tts", "project", 4, ({ onProgress }) => {
    onProgress(0.5);
    return Promise.resolve({
      result: {
        assetId: "asset",
        durationMs: 100,
        itemId: "item",
        language: "en-US",
        modelId: "model",
        modelVersion: null,
        providerId: "provider",
        revision: 5,
        voice: "af_heart",
        warnings: [],
      },
    });
  });
  const completed = await waitForTerminal(jobs, queued.jobId);
  expect(completed.status).toBe("completed");
  expect(completed.revision).toBe(4);
  expect(completed.result?.revision).toBe(5);
});

it("preserves a structured worker failure", async () => {
  const jobs = new JobRegistry();
  const queued = jobs.startTask("tts", "project", 4, () =>
    Promise.reject(new BridgeError("TTS_SYNTHESIS_FAILED", "failed", true))
  );
  const failed = await waitForTerminal(jobs, queued.jobId);
  expect(failed.status).toBe("failed");
  expect(failed.error).toMatchObject({
    code: "TTS_SYNTHESIS_FAILED",
    retryable: true,
  });
});

it("clamps finite progress and keeps it monotonic", async () => {
  let finish: (() => void) | undefined;
  const jobs = new JobRegistry();
  const queued = jobs.startTask("export", "project", 1, ({ onProgress }) => {
    onProgress(0.7);
    onProgress(0.2);
    onProgress(4);
    onProgress(Number.NaN);
    return new Promise((resolvePromise) => {
      finish = () => resolvePromise({});
    });
  });
  expect(jobs.get(queued.jobId).progress).toBe(1);
  finish?.();
  await waitForTerminal(jobs, queued.jobId);
});

it("returns stable errors and enforces active capacity", () => {
  const jobs = new JobRegistry({ maxCount: 1 });
  expect(() => jobs.get("missing")).toThrowError(
    expect.objectContaining({ code: "JOB_NOT_FOUND" })
  );
  jobs.startTask("export", "project", 1, () => new Promise(() => undefined));
  expect(() =>
    jobs.startTask("export", "project", 1, () => Promise.resolve({}))
  ).toThrowError(expect.objectContaining({ code: "JOB_REGISTRY_FULL" }));
});

it("cancels eligible jobs but protects the atomic commit phase", async () => {
  const jobs = new JobRegistry();
  const cancellable = jobs.startTask(
    "tts",
    "project",
    1,
    ({ signal }) =>
      new Promise((_resolve, reject) => {
        signal.addEventListener("abort", () => reject(new Error("aborted")));
      })
  );
  expect(jobs.cancel(cancellable.jobId).status).toBe("cancelled");
  expect(jobs.cancel(cancellable.jobId).status).toBe("cancelled");

  let finish: (() => void) | undefined;
  const committing = jobs.startTask(
    "tts",
    "project",
    1,
    ({ markNonCancellable }) => {
      markNonCancellable();
      return new Promise((resolvePromise) => {
        finish = () => resolvePromise({});
      });
    }
  );
  expect(() => jobs.cancel(committing.jobId)).toThrowError(
    expect.objectContaining({ code: "JOB_NOT_CANCELLABLE" })
  );
  finish?.();
  await waitForTerminal(jobs, committing.jobId);
});

it("expires terminal jobs and reports process-local retention", async () => {
  let now = 100;
  const jobs = new JobRegistry({ now: () => now, ttlMs: 10 });
  const queued = jobs.startTask("tts", "project", 1, () => Promise.resolve({}));
  const completed = await waitForTerminal(jobs, queued.jobId);
  expect(completed).toMatchObject({
    expiresAtMs: 110,
    persistence: "process",
  });
  now = 111;
  expect(() => jobs.get(queued.jobId)).toThrowError(
    expect.objectContaining({ code: "JOB_NOT_FOUND" })
  );
});

it("keeps preparation visible while running and freezes it after cancellation", async () => {
  const jobs = new JobRegistry();
  let release: () => void = () => {
    throw new Error("Preparation gate not initialized");
  };
  const gate = new Promise<void>((resolve) => {
    release = resolve;
  });
  let update: (value: number) => void = (_value: number) => {
    throw new Error("Preparation observer not initialized");
  };
  const job = jobs.startTask(
    "preview",
    "preparation",
    1,
    async ({ onProgress }) => {
      update = onProgress;
      onProgress(PREPARATION_PROGRESS.sampledProgress[0] ?? 0);
      await gate;
      onProgress(0.99);
      return {};
    }
  );
  expect(jobs.get(job.jobId)).toMatchObject({
    progress: 0.125,
    status: "running",
  });
  update(0.25);
  jobs.cancel(job.jobId);
  const cancelled = jobs.get(job.jobId);
  expect(cancelled).toMatchObject({ progress: 0.25, status: "cancelled" });
  update(0.5);
  release();
  await jobs.close();
  expect(jobs.get(job.jobId)).toMatchObject({
    progress: 0.25,
    status: "cancelled",
  });
});

it("requires task completion after preparation and encoding progress", async () => {
  const jobs = new JobRegistry();
  let release: () => void = () => {
    throw new Error("Preparation gate not initialized");
  };
  const gate = new Promise<void>((resolve) => {
    release = resolve;
  });
  const job = jobs.startTask(
    "export",
    "preparation",
    1,
    async ({ onProgress }) => {
      for (const progress of PREPARATION_PROGRESS.sampledProgress.filter(
        (value) => value < 1
      )) {
        onProgress(progress);
      }
      await gate;
      return {};
    }
  );
  expect(jobs.get(job.jobId)).toMatchObject({
    progress: 0.99,
    status: "running",
  });
  release();
  expect(await waitForTerminal(jobs, job.jobId)).toMatchObject({
    progress: 1,
    status: "completed",
  });
  await jobs.close();
});
