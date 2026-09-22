import { spawn } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { createInterface } from "node:readline";

const prepareHang = (request: Record<string, unknown>, id: string) => {
  const projects = process.env.OPENCUT_PROJECTS_DIR;
  if (projects) {
    const dir = join(projects, String(request.projectId));
    mkdirSync(join(dir, "previews"), { recursive: true });
    mkdirSync(join(dir, `.opencut-work-${id}`), { recursive: true });
    writeFileSync(join(dir, "previews", `.opencut-${id}.png`), "partial");
    if (request.testMode === "hang-tree") {
      writeFileSync(join(dir, "previews", "published.png"), "published");
      const descendant = spawn(
        process.execPath,
        ["-e", "setInterval(() => {}, 1000)"],
        { stdio: "ignore", windowsHide: true }
      );
      writeFileSync(join(dir, "descendant.pid"), String(descendant.pid));
    }
  }
};

const worker = process.argv.includes("--render-worker");
let count = 0;
const respond = (request: Record<string, unknown>, id: string) => {
  count += 1;
  const send = (event: unknown, requestId = id) => {
    process.stdout.write(
      `${JSON.stringify(worker ? { event, requestId } : event)}\n`
    );
  };
  const mode = request.testMode;
  if (mode === "hang" || mode === "hang-tree") {
    prepareHang(request, id);
    setInterval(() => undefined, 1000);
    return;
  }
  if (mode === "limit" || mode === "oversized") {
    const response = {
      event: {
        result: { count, padding: "", pid: process.pid, worker },
        type: "result",
      },
      requestId: id,
    };
    const length = Buffer.byteLength(JSON.stringify(response));
    response.event.result.padding = "x".repeat(
      16 * 1024 * 1024 - length + (mode === "oversized" ? 1 : 0)
    );
    process.stdout.write(`${JSON.stringify(response)}\n`);
    return;
  }
  if (mode === "invalid-utf8") {
    process.stdout.write(Buffer.from([0xff, 10]));
    return;
  }
  if (mode === "crash") {
    process.exit(2);
  }
  if (mode === "malformed") {
    process.stdout.write("not-json\n");
    return;
  }
  if (mode === "wrong-id") {
    send({ result: {}, type: "result" }, "wrong");
    return;
  }
  if (mode === "error") {
    send({
      error: { code: "REVISION_CONFLICT", message: "stale", retryable: true },
      type: "error",
    });
    return;
  }
  const result = {
    result: { count, pid: process.pid, worker },
    type: "result",
  };
  if (mode === "slow") {
    setTimeout(() => send(result), 500);
    return;
  }
  if (mode === "duplicate") {
    const line = JSON.stringify(
      worker ? { event: result, requestId: id } : result
    );
    process.stdout.write(`${line}\n${line}\n`);
    return;
  }
  if (mode === "partial") {
    const line = `${JSON.stringify({ event: result, requestId: id })}\n`;
    process.stdout.write(line.slice(0, 12));
    setTimeout(() => process.stdout.write(line.slice(12)), 10);
    return;
  }
  send(result);
};

if (worker) {
  const startup = process.env.OPENCUT_TEST_WORKER_STARTUP;
  if (startup === "exit") {
    process.exit(2);
  }
  if (startup === "hang") {
    await new Promise(() => setInterval(() => undefined, 1000));
  }
  if (startup === "version") {
    process.stdout.write('{"type":"ready","protocolVersion":2}\n');
  }
  process.stdout.write(
    `${JSON.stringify({ protocolVersion: 1, type: "ready" })}\n`
  );
  for await (const line of createInterface({ input: process.stdin })) {
    const { requestId, request } = JSON.parse(line);
    respond(request, requestId);
  }
} else {
  let input = "";
  for await (const chunk of process.stdin) {
    input += chunk;
  }
  respond(JSON.parse(input), process.env.OPENCUT_REQUEST_ID ?? "one-shot");
}
