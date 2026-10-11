import { writeFileSync, writeSync } from "node:fs";
import { join } from "node:path";
import { BridgeError } from "../../src/headless";
import { JobRuntimeGuard } from "../../src/job-runtime";
import { JobRegistry } from "../../src/jobs";
import { NOOP_LOGGER } from "../../src/logger";

const [mode, root, ...ids] = process.argv.slice(2);
if (!root) {
  throw new Error("Missing fixture root");
}
const guard = new JobRuntimeGuard(root);
const jobs = new JobRegistry({
  beforeStart: () => guard.acquire(),
  logger: {
    ...NOOP_LOGGER,
    info: (event, fields) => {
      if (mode === "produce-queued" && event === "job.admitted") {
        writeSync(
          1,
          `${JSON.stringify({ jobs: [jobs.get(String(fields?.jobId))] })}\n`
        );
        process.kill(process.pid, "SIGKILL");
      }
    },
  },
});
if (mode === "probe") {
  const missing = ids.map((id) => {
    try {
      jobs.get(id);
      return "unexpected";
    } catch (error) {
      return (error as BridgeError).code;
    }
  });
  let admission: string | undefined;
  try {
    jobs.startTask("preview", "private-project", 7, () => {
      writeFileSync(join(root, "unexpected-output"), "private");
      return Promise.resolve({});
    });
  } catch (error) {
    admission = (error as BridgeError).code;
  }
  process.stdout.write(`${JSON.stringify({ admission, missing })}\n`);
} else {
  const waiting = () => new Promise<never>(() => undefined);
  // Pending producers retain a running registry envelope. The separate
  // produce-queued branch kills admission before synchronous dispatch.
  const waitingProducer = jobs.startTask("tts", "private-project", 7, waiting);
  const running = jobs.startTask("preview", "private-project", 7, waiting);
  const completed = jobs.startTask("preview", "private-project", 7, () => {
    writeFileSync(join(root, "private-output"), "private bytes");
    return Promise.resolve({});
  });
  const failed = jobs.startTask("tts", "private-project", 7, () =>
    Promise.reject(new BridgeError("TTS_SYNTHESIS_FAILED", "safe failure"))
  );
  const cancelled = jobs.startTask("preview", "private-project", 7, waiting);
  jobs.cancel(cancelled.jobId);
  await new Promise((resolve) => setTimeout(resolve, 0));
  process.stdout.write(
    `${JSON.stringify({
      jobs: [waitingProducer, running, completed, failed, cancelled].map(
        (job) => jobs.get(job.jobId)
      ),
    })}\n`
  );
  setInterval(() => undefined, 1000);
}
