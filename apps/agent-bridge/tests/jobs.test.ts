import { expect, it } from "vitest";

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
