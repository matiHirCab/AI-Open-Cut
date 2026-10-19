/** Explicit real-runtime acceptance; run with Bun and OPENCUT_HEADLESS_PATH, never readiness-skip. */
import { deepStrictEqual, equal, ok } from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtemp, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import contract from "../../../contracts/short-candidate-planning-cli-v1.json";
import { loadBridgeConfig } from "../src/config";
import { HeadlessClient } from "../src/headless";
import { projectStateSchema, writeResultSchema } from "../src/schemas";
import {
  PLANNING_LIMITS,
  shortPlanningFailureSchema,
  shortPlanningSuccessSchema,
} from "../src/short-candidates/contract";
import { parsePlanningInput } from "../src/short-candidates/planner";

const executable = process.env.OPENCUT_HEADLESS_PATH;
ok(
  executable,
  "Set OPENCUT_HEADLESS_PATH to the built native executable; this acceptance cannot skip."
);
const root = await mkdtemp(join(tmpdir(), "opencut-short-planning-"));
const store = join(root, "projects");
const bridge = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("../src/short-candidates-cli.ts", import.meta.url)
);
const environment = {
  ...process.env,
  OPENCUT_ALLOWED_MEDIA_DIRS: root,
  OPENCUT_CONFIG_ROOT: root,
  OPENCUT_EXPORTS_DIR: join(root, "exports"),
  OPENCUT_HEADLESS_PATH: executable,
  OPENCUT_PROJECTS_DIR: store,
};
const headless = new HeadlessClient(loadBridgeConfig(environment));

const wave = () => {
  const rate = 48_000;
  const samples = rate * 2;
  const bytes = Buffer.alloc(44 + samples * 2);
  bytes.write("RIFF", 0);
  bytes.writeUInt32LE(bytes.length - 8, 4);
  bytes.write("WAVEfmt ", 8);
  bytes.writeUInt32LE(16, 16);
  bytes.writeUInt16LE(1, 20);
  bytes.writeUInt16LE(1, 22);
  bytes.writeUInt32LE(rate, 24);
  bytes.writeUInt32LE(rate * 2, 28);
  bytes.writeUInt16LE(2, 32);
  bytes.writeUInt16LE(16, 34);
  bytes.write("data", 36);
  bytes.writeUInt32LE(samples * 2, 40);
  for (let index = 0; index < samples; index += 1) {
    bytes.writeInt16LE(
      Math.round(4000 * Math.sin((2 * Math.PI * 440 * index) / rate)),
      44 + index * 2
    );
  }
  return bytes;
};
const inventory = async (
  directory: string,
  prefix = ""
): Promise<Record<string, string>> => {
  const entries = await readdir(directory, { withFileTypes: true });
  const values = await Promise.all(
    entries.map(async (entry) => {
      const relative = join(prefix, entry.name);
      const path = join(directory, entry.name);
      if (entry.isDirectory()) {
        return inventory(path, relative);
      }
      ok(
        entry.isFile(),
        "Acceptance inventory must contain only owned regular files."
      );
      return {
        [relative]: createHash("sha256")
          .update(await readFile(path))
          .digest("hex"),
      };
    })
  );
  return Object.assign({}, ...values) as Record<string, string>;
};
const run = async (input: unknown) => {
  const child = spawn(process.execPath, [cli], {
    cwd: bridge,
    env: environment,
    stdio: "pipe",
  });
  const output: Buffer[] = [];
  const errors: Buffer[] = [];
  child.stdout.on("data", (data: Buffer) => output.push(data));
  child.stderr.on("data", (data: Buffer) => errors.push(data));
  child.stdin.on("error", () => {
    // Oversized input can close the owned reader before the writer finishes.
  });
  const timer = setTimeout(() => child.kill("SIGKILL"), 60_000);
  try {
    const completion = new Promise<number | null>((resolve, reject) => {
      child.once("error", reject);
      child.once("close", resolve);
    });
    child.stdin.end(JSON.stringify(input));
    const code = await completion;
    return {
      code,
      stderr: Buffer.concat(errors).toString("utf8"),
      stdout: Buffer.concat(output).toString("utf8"),
    };
  } finally {
    clearTimeout(timer);
  }
};

try {
  const path = join(root, "original-tone.wav");
  const originalBytes = wave();
  await writeFile(path, originalBytes);
  const created = await headless.call(
    {
      name: "Original read-only candidate planning",
      operation: "create_project",
    },
    writeResultSchema
  );
  const imported = await headless.call(
    {
      expectedRevision: created.revision,
      mediaType: "audio",
      operation: "import_asset",
      path,
      projectId: created.projectId,
    },
    writeResultSchema
  );
  const state = await headless.call(
    { operation: "get_state", projectId: created.projectId },
    projectStateSchema
  );
  const [asset] = state.project.assets;
  ok(asset?.contentHash);
  equal(asset.probe?.durationMs, 2000);
  const input = parsePlanningInput({
    ...contract.valid[0]?.input,
    assetId: asset.id,
    expectedRevision: imported.revision,
    projectId: created.projectId,
  });
  const before = await inventory(store);
  const successful = await run(input);
  equal(successful.code, 0, successful.stderr);
  equal(successful.stderr, "");
  const plan = shortPlanningSuccessSchema.parse(JSON.parse(successful.stdout));
  deepStrictEqual(plan.candidates, contract.valid[0]?.expected.candidates);
  deepStrictEqual(plan.source.contentHash, asset.contentHash);
  equal(plan.source.projectId, created.projectId);
  equal(plan.source.assetId, asset.id);
  equal(plan.source.revision, imported.revision);
  ok(
    !successful.stdout.includes(root),
    "Plan exposed a managed absolute path."
  );

  const edited = await run({ ...input, edits: contract.valid[1]?.input.edits });
  equal(edited.code, 0, edited.stderr);
  deepStrictEqual(
    shortPlanningSuccessSchema.parse(JSON.parse(edited.stdout)).candidates,
    contract.valid[1]?.expected.candidates
  );
  const failures = await Promise.all([
    run({ ...input, expectedRevision: imported.revision - 1 }),
    run({ ...input, assetId: "missing-approved-source" }),
    run({ ...input, approvedTranscript: false }),
    run({ ...input, padding: "x".repeat(PLANNING_LIMITS.inputBytes) }),
  ]);
  const failureKinds = failures.map((result) => {
    equal(result.code, 1, result.stderr);
    equal(result.stdout, "");
    return shortPlanningFailureSchema.parse(JSON.parse(result.stderr)).error
      .kind;
  });
  deepStrictEqual(failureKinds, ["stale", "core", "input", "limit"]);
  equal(JSON.parse(failures[1]?.stderr ?? "").error.code, "ASSET_NOT_FOUND");

  const large = structuredClone(input);
  for (const chunk of large.chunks) {
    chunk.segments = Array.from({ length: 1000 }, (_, index) => ({
      endMs: index + 1,
      id: `s${index}`,
      startMs: index,
      text: "x".repeat(2000),
    }));
  }
  large.edits = ["candidate-1", "candidate-2"].map((candidateId) => ({
    candidateId,
    endMs: 2000,
    startMs: 0,
  }));
  ok(Buffer.byteLength(JSON.stringify(large)) <= PLANNING_LIMITS.inputBytes);
  const excessiveOutput = await run(large);
  equal(excessiveOutput.code, 1, excessiveOutput.stderr);
  equal(excessiveOutput.stdout, "");
  equal(
    shortPlanningFailureSchema.parse(JSON.parse(excessiveOutput.stderr)).error
      .kind,
    "limit"
  );

  const after = await inventory(store);
  deepStrictEqual(after, before);
  deepStrictEqual(await readFile(path), originalBytes);
  const finalState = await headless.call(
    { operation: "get_state", projectId: created.projectId },
    projectStateSchema
  );
  deepStrictEqual(finalState, state);
  process.stdout.write(
    `${JSON.stringify({ authoritativeFiles: Object.keys(before).length, durationMs: plan.source.durationMs, failureKinds, outputLimitVerifiedBeforeStdout: true, platform: process.platform, projectRevision: imported.revision, result: "PASS", sourceAndHistoryUnchanged: true, sourceDigest: plan.source.contentHash.digest, sourceProvenance: "original synthetic tone and explicitly supplied transcript, no inference" })}\n`
  );
} finally {
  await headless.close();
  await rm(root, { force: true, recursive: true });
}
