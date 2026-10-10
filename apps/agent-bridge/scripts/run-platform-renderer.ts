import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import {
  assembleRuntimePackage,
  verifyRuntimePackage,
} from "./package-runtime";

import { requirePlatformNativeReport } from "./platform-native-report";

const repository = resolve(import.meta.dirname, "../../..");
const root = await mkdtemp(join(tmpdir(), "opencut-platform-runtime-"));
const suffix = process.platform === "win32" ? ".exe" : "";
const headless = join(
  repository,
  "target/release",
  `opencut-headless${suffix}`
);
const bridge = join(root, `opencut-agent-bridge${suffix}`);
const runtime = join(root, "runtime");
const filterFixture = join(root, `filter-inventory${suffix}`);
const run = async (command: string[], env = process.env, cwd = repository) => {
  const child = Bun.spawn(command, {
    cwd,
    env,
    stderr: "inherit",
    stdout: "inherit",
  });
  const code = await child.exited;
  if (code !== 0) {
    throw new Error(`${command.join(" ")} exited ${code}`);
  }
};
try {
  for (const name of [
    "OPENCUT_FFMPEG_PATH",
    "OPENCUT_FFPROBE_PATH",
    "OPENCUT_TEST_FONT_PATH",
    "OPENCUT_PLATFORM_EVIDENCE_DIR",
  ]) {
    if (!process.env[name]) {
      throw new Error(`Required platform dependency: ${name}`);
    }
  }
  await run(["cargo", "build", "--release", "-p", "opencut-headless"]);
  await run([
    "bun",
    "build",
    "apps/agent-bridge/src/index.ts",
    "--target=bun",
    "--compile",
    `--outfile=${bridge}`,
  ]);
  await run([
    "rustc",
    "apps/agent-bridge/tests/fixtures/platform_filter_inventory.rs",
    "-o",
    filterFixture,
  ]);
  await assembleRuntimePackage(runtime, {
    bridge,
    headless,
    transcriptionWorker: join(repository, "apps/faster-whisper/worker.py"),
    worker: join(repository, "apps/kokoro-tts/worker.py"),
  });
  const manifest = await verifyRuntimePackage(runtime);
  const evidenceDirectory = process.env.OPENCUT_PLATFORM_EVIDENCE_DIR;
  if (!evidenceDirectory) { throw new Error("Missing platform evidence directory"); }
  await Bun.write(
    join(evidenceDirectory, "manifest.json"),
    JSON.stringify(manifest, null, 2)
  );
  await run(
    [
      "bun",
      "x",
      "--no-install",
      "vitest",
      "run",
      "--config",
      "vitest.unit.config.ts",
      "tests/platform-renderer-native.test.ts",
      "--reporter=default",
      "--reporter=json",
      `--outputFile=${join(root, "native-results.json")}`,
    ],
    {
      ...process.env,
      OPENCUT_PLATFORM_FILTER_FIXTURE: filterFixture,
      OPENCUT_PLATFORM_PACKAGE: runtime,
      OPENCUT_PLATFORM_REQUIRED: "1",
      OPENCUT_PLATFORM_SOURCE_HEADLESS: headless,
    },
    join(repository, "apps/agent-bridge")
  );
  const nativeReport = JSON.parse(await readFile(join(root, "native-results.json"), "utf8"));
  requirePlatformNativeReport(nativeReport);
  await Bun.write(join(evidenceDirectory, "native-results.json"), JSON.stringify(nativeReport, null, 2));
  await verifyRuntimePackage(runtime);
} finally {
  await rm(root, { force: true, recursive: true });
}
