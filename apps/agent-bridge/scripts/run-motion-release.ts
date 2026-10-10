import { randomUUID } from "node:crypto";
import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  statSync,
} from "node:fs";
import { copyFile } from "node:fs/promises";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { CryptoHasher, spawn, spawnSync, write } from "bun";
import { requireMotionMcpReport } from "./motion-mcp-report";
import {
  RELEASE_CASES,
  releaseCaseArguments,
  requireExactRustCase,
  requireReleaseInventory,
} from "./motion-release-cases";
import {
  RELEASE_FONTS,
  requireMotionCoreReport,
} from "./motion-release-report";

const repository = resolve(import.meta.dirname, "../../..");
if (process.env.OPENCUT_UPDATE_GOLDENS === "1") {
  throw new Error("Golden updates are forbidden during release verification");
}
const requiredFile = (name: string) => {
  const path = process.env[name];
  if (!(path && isAbsolute(path) && statSync(path).isFile())) {
    throw new Error(`Required actual release file: ${name}`);
  }
  return path;
};
const ffmpeg = requiredFile("OPENCUT_FFMPEG_PATH");
const ffprobe = requiredFile("OPENCUT_FFPROBE_PATH");
const font = requiredFile("OPENCUT_TEST_FONT_PATH");
const python = requiredFile("OPENCUT_TEST_PYTHON");
for (const [name, hash] of Object.entries(RELEASE_FONTS)) {
  if (
    new CryptoHasher("sha256")
      .update(readFileSync(join(dirname(font), name)))
      .digest("hex") !== hash
  ) {
    throw new Error(`Required fixed font differs: ${name}`);
  }
}
const evidence = process.env.OPENCUT_MOTION_EVIDENCE_DIR;
if (!(evidence && isAbsolute(evidence))) {
  throw new Error("Required absolute release evidence directory");
}
const nonce = randomUUID();
const root = join(evidence, nonce);
mkdirSync(root, { recursive: true });
const suffix = process.platform === "win32" ? ".exe" : "";
const runner = join(root, `owned-runner${suffix}`);
// Compile the small ownership fixture before it starts any timed process.
const compiled = spawnSync(
  [
    "rustc",
    "--edition=2024",
    "-D",
    "warnings",
    "crates/editor-core/tests/fixtures/owned_release_process.rs",
    "-o",
    runner,
  ],
  { cwd: repository }
);
appendFileSync(join(root, "runner-build.log"), compiled.stderr);
if (compiled.exitCode !== 0) {
  throw new Error("Owned release runner compilation failed");
}
let sequence = 0;
const run = async (
  command: string[],
  timeout: number,
  env = process.env,
  cwd = repository
) => {
  const log = join(root, `${sequence}.log`);
  sequence += 1;
  appendFileSync(log, `${JSON.stringify(command)}\n`);
  const child = spawn([runner, String(timeout), ...command], {
    cwd,
    env,
    stderr: "pipe",
    stdout: "pipe",
  });
  const collect = async (stream: ReadableStream<Uint8Array>) => {
    let tail = "";
    for await (const bytes of stream) {
      appendFileSync(log, bytes);
      tail = (tail + new TextDecoder().decode(bytes)).slice(-512_000);
    }
    return tail;
  };
  let result: [number, string, string];
  try {
    result = await Promise.all([
      child.exited,
      collect(child.stdout),
      collect(child.stderr),
    ]);
  } catch (error) {
    // Closing the Windows runner closes its owned job; Unix SIGTERM is handled
    // by the runner before exit. Settle the owner before propagating log failure.
    child.kill();
    await child.exited;
    throw error;
  }
  const [code, stdout, stderr] = result;
  if (code !== 0) {
    throw new Error(
      `Required release command failed (${code}); ${log}\n${stderr.slice(-4000)}`
    );
  }
  return stdout;
};
const checkout = (await run(["git", "rev-parse", "HEAD"], 30_000)).trim();
const sourceDirty =
  (await run(["git", "status", "--porcelain"], 30_000)).trim().length > 0;
const requested = process.env.OPENCUT_MOTION_REQUESTED_HEAD ?? checkout;
await run(["git", "merge-base", "--is-ancestor", requested, checkout], 30_000);
await write(
  join(root, "source-identity.json"),
  JSON.stringify(
    {
      checkoutHead: checkout,
      requestedHead: requested,
      runNonce: nonce,
      sourceDirty,
      version: 1,
    },
    null,
    2
  )
);
await run([ffmpeg, "-version"], 30_000);
await run([ffprobe, "-version"], 30_000);
await run([python, "--version"], 30_000);
const inventory = { cases: RELEASE_CASES, version: 1 };
requireReleaseInventory(inventory);
await write(
  join(root, "case-inventory.json"),
  JSON.stringify(inventory, null, 2)
);
for (const row of RELEASE_CASES) {
  // biome-ignore lint/performance/noAwaitInLoops: Isolate Cargo execution and sampler controls; concurrent native work changes observations.
  const output = await run(releaseCaseArguments(row), 900_000);
  requireExactRustCase(row.selector, output, 0);
}
for (const selector of [
  "measurement::tests::actual_owned_child_allocation_and_stop_join",
  "measurement::tests::collector_readiness_observation_and_exit_failures_settle",
]) {
  // biome-ignore lint/performance/noAwaitInLoops: Isolate Cargo execution and sampler controls; concurrent native work changes observations.
  const output = await run(
    [
      "cargo",
      "test",
      "--release",
      "--no-default-features",
      "-p",
      "opencut-editor-core",
      "--test",
      "motion_release",
      selector,
      "--",
      "--exact",
      "--color",
      "never",
    ],
    900_000
  );
  requireExactRustCase(selector, output, 0);
}
const core = join(root, "core");
const coreOutput = await run(
  [
    "cargo",
    "test",
    "--release",
    "--no-default-features",
    "-p",
    "opencut-editor-core",
    "--test",
    "motion_release",
    "default_native_release_scene",
    "--",
    "--exact",
    "--ignored",
    "--nocapture",
    "--color",
    "never",
  ],
  3_600_000,
  {
    ...process.env,
    OPENCUT_MOTION_CORE_EVIDENCE_DIR: core,
    OPENCUT_MOTION_REAL_FFMPEG: ffmpeg,
    OPENCUT_MOTION_RELEASE_REQUIRED: "1",
    OPENCUT_MOTION_RUN_NONCE: nonce,
  }
);
requireExactRustCase("default_native_release_scene", coreOutput, 0);
requireMotionCoreReport(
  JSON.parse(readFileSync(join(core, "core-report.json"), "utf8")),
  {
    architecture:
      ({ arm64: "aarch64", x64: "x86_64" } as Record<string, string>)[
        process.arch
      ] ?? process.arch,
    nonce,
    platform:
      ({ darwin: "macos", win32: "windows" } as Record<string, string>)[
        process.platform
      ] ?? process.platform,
  }
);
// The default proof has completed before this explicitly private feature build.
await run(
  [
    "cargo",
    "build",
    "--release",
    "-p",
    "opencut-headless",
    "--features",
    "raster-cache-test-hooks",
  ],
  1_800_000
);
const instrumented = join(root, `instrumented-headless${suffix}`);
await copyFile(
  join(repository, "target/release", `opencut-headless${suffix}`),
  instrumented
);
const mcpReport = join(root, "mcp-results.json");
await run(
  [
    "bun",
    "x",
    "--no-install",
    "vitest",
    "run",
    "--config",
    "vitest.unit.config.ts",
    "tests/reference-scene-native.test.ts",
    "--reporter=default",
    "--reporter=json",
    `--outputFile=${mcpReport}`,
  ],
  3_300_000,
  {
    ...process.env,
    OPENCUT_REFERENCE_REPORT_DIR: join(root, "instrumented-mcp"),
    OPENCUT_REFERENCE_REQUIRED: "1",
    OPENCUT_TEST_HEADLESS_PATH: instrumented,
  },
  join(repository, "apps/agent-bridge")
);
requireMotionMcpReport(JSON.parse(readFileSync(mcpReport, "utf8")));
for (const path of [
  "core/core-report.json",
  "core/frame650.png",
  "core/reference.mp4",
  "mcp-results.json",
  "instrumented-mcp/source/evidence.json",
  "instrumented-mcp/source/reference.mp4",
  "instrumented-mcp/compiled/evidence.json",
  "instrumented-mcp/compiled/reference.mp4",
]) {
  const absolute = join(root, path);
  if (
    !(existsSync(absolute) && statSync(absolute).isFile()) ||
    statSync(absolute).size === 0
  ) {
    throw new Error(`Required release artifact missing: ${path}`);
  }
}
await write(
  join(root, "accepted.json"),
  JSON.stringify(
    {
      canonicalCases: RELEASE_CASES.length,
      checkoutHead: checkout,
      defaultNativeCases: 1,
      instrumentedMcpCases: 2,
      requestedHead: requested,
      runNonce: nonce,
      sourceDirty,
      version: 1,
    },
    null,
    2
  )
);
console.log(`Verified motion release evidence: ${root}`);
