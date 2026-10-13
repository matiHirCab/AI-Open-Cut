// Transport-only canonical failure fixture; it performs no media execution.
import { spawnSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import { createInterface } from "node:readline";

const CATALOG = JSON.parse(
  readFileSync(
    new URL(
      "../../../../contracts/ffmpeg-spawn-diagnostics-v1.json",
      import.meta.url
    ),
    "utf8"
  )
);

const worker = process.argv.includes("--render-worker");
const respond = (request: Record<string, unknown>, requestId?: string) => {
  if (
    !String(request.operation).startsWith("render_") &&
    request.operation !== "export_video"
  ) {
    const real = process.env.OPENCUT_TEST_REAL_HEADLESS_PATH;
    if (!real || worker) {
      throw new Error("Owned non-render delegation is required");
    }
    const output = spawnSync(real, [], {
      encoding: "utf8",
      env: process.env,
      input: JSON.stringify(request),
      maxBuffer: 16 * 1024 * 1024,
    });
    if (output.status !== 0) {
      throw new Error("Owned non-render delegation failed");
    }
    process.stdout.write(output.stdout);
    return;
  }
  const marker = process.env.OPENCUT_TEST_FFMPEG_SPAWN_MARKER;
  if (!marker) {
    throw new Error("Owned transport marker is required");
  }
  appendFileSync(marker, worker ? "worker\n" : "one-shot\n");
  const event = CATALOG.headlessErrorExample;
  process.stdout.write(
    `${JSON.stringify(worker ? { event, requestId } : event)}\n`
  );
};

if (worker) {
  process.stdout.write('{"type":"ready","protocolVersion":1}\n');
  for await (const line of createInterface({ input: process.stdin })) {
    const envelope = JSON.parse(line);
    if (!(envelope.requestId && envelope.request)) {
      throw new Error("Expected a render-worker envelope");
    }
    respond(envelope.request, envelope.requestId);
  }
} else {
  let input = "";
  for await (const chunk of process.stdin) {
    input += chunk;
  }
  respond(JSON.parse(input));
}
