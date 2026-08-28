import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const repository = resolve(import.meta.dirname, "../../..");
const temporary = await mkdtemp(join(tmpdir(), "opencut-packaged-smoke-"));
const executableSuffix = process.platform === "win32" ? ".exe" : "";
const bridge = join(temporary, `opencut-agent-bridge${executableSuffix}`);
const headless = join(temporary, `opencut-headless${executableSuffix}`);
const runtime = join(temporary, "runtime");
const packagedBridge = join(runtime, `opencut-agent-bridge${executableSuffix}`);
const packagedHeadless = join(runtime, `opencut-headless${executableSuffix}`);
const { assembleRuntimePackage, verifyRuntimePackage } = await import(
  "./package-runtime"
);

const run = async (
  command: string[],
  environment = process.env,
  cwd = repository
) => {
  const child = Bun.spawn(command, {
    cwd,
    env: environment,
    stderr: "inherit",
    stdout: "inherit",
  });
  const exitCode = await child.exited;
  if (exitCode !== 0) {
    throw new Error(`${command.join(" ")} exited with ${exitCode}`);
  }
};

try {
  await run(["cargo", "build", "--release", "-p", "opencut-headless"]);
  await Bun.write(
    headless,
    Bun.file(
      join(
        repository,
        "target",
        "release",
        `opencut-headless${executableSuffix}`
      )
    )
  );
  await run([
    "bun",
    "build",
    "apps/agent-bridge/src/index.ts",
    "--target=bun",
    "--compile",
    `--outfile=${bridge}`,
  ]);
  await assembleRuntimePackage(runtime, {
    bridge,
    headless,
    transcriptionWorker: join(
      repository,
      "apps",
      "faster-whisper",
      "worker.py"
    ),
    worker: join(repository, "apps", "kokoro-tts", "worker.py"),
  });
  await verifyRuntimePackage(runtime);
  await run(
    ["bun", "x", "vitest", "run", "--config", "vitest.smoke.config.ts"],
    {
      ...process.env,
      OPENCUT_TEST_BRIDGE_PATH: packagedBridge,
      OPENCUT_TEST_HEADLESS_PATH: packagedHeadless,
    },
    resolve(repository, "apps/agent-bridge")
  );
} finally {
  await rm(temporary, { force: true, recursive: true });
}
