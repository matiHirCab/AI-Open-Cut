// Controlled transport evidence; actual composition is verified in core/native suites.
import { spawnSync } from "node:child_process";
import { appendFileSync, existsSync } from "node:fs";
import { createInterface } from "node:readline";
import PROGRESS from "../../../../contracts/render-preparation-progress-v1.json";

const real = process.env.OPENCUT_TEST_REAL_HEADLESS_PATH;
const release = process.env.OPENCUT_TEST_PREPARATION_RELEASE;
const marker = process.env.OPENCUT_TEST_PREPARATION_MARKER;
if (!(real && release && marker)) {
  throw new Error("Missing owned transport fixture setup");
}
const worker = process.argv.includes("--render-worker");
const run = async (request: Record<string, unknown>, requestId: string) => {
  const send = (event: unknown) =>
    process.stdout.write(
      `${JSON.stringify(worker ? { event, requestId } : event)}\n`
    );
  if (
    request.operation === "render_preview_range" &&
    request.expectedRevision === 1
  ) {
    appendFileSync(marker, `${worker ? "worker" : "one-shot"}\n`);
    for (const progress of PROGRESS.sampledProgress.filter(
      (value) => value <= 0.25
    )) {
      send({ progress, type: "progress" });
    }
    const deadline = Date.now() + 5000;
    const waitForRelease = async (): Promise<void> => {
      if (existsSync(release)) {
        return;
      }
      if (Date.now() >= deadline) {
        throw new Error("Owned preparation barrier not released");
      }
      await new Promise((resolve) => setTimeout(resolve, 10));
      return waitForRelease();
    };
    await waitForRelease();
    for (const progress of PROGRESS.sampledProgress.filter(
      (value) => value > 0.25 && value < 1
    )) {
      send({ progress, type: "progress" });
    }
    send({
      error: {
        code: "FFMPEG_FAILED",
        failedStage: "render",
        ffmpegExitCode: 7,
        ffmpegStderrExcerpt: "safe injected failure",
        message: "FFmpeg render failed",
        retryable: false,
      },
      type: "error",
    });
    return;
  }
  const result = spawnSync(real, [], {
    encoding: "utf8",
    env: process.env,
    input: JSON.stringify(request),
  });
  if (result.status !== 0) {
    throw new Error("Real fixture delegate failed");
  }
  for (const line of result.stdout.trim().split("\n")) {
    send(JSON.parse(line));
  }
};
if (worker) {
  process.stdout.write('{"type":"ready","protocolVersion":1}\n');
  for await (const line of createInterface({ input: process.stdin })) {
    const { requestId, request } = JSON.parse(line);
    await run(request, requestId);
  }
} else {
  let input = "";
  for await (const chunk of process.stdin) {
    input += chunk;
  }
  await run(JSON.parse(input), "one-shot");
}
