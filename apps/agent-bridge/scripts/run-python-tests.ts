import { existsSync } from "node:fs";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const repository = resolve(import.meta.dirname, "../../..");
const kokoro = resolve(repository, "apps/kokoro-tts");
const transcription = resolve(repository, "apps/faster-whisper");
const environment = resolve(repository, "local-data/kokoro-test-venv");
const python =
  process.env.OPENCUT_TEST_PYTHON ??
  (process.platform === "win32" ? "python" : "python3");
const environmentPython = resolve(
  environment,
  process.platform === "win32" ? "Scripts/python.exe" : "bin/python"
);
const lockPaths = [
  resolve(kokoro, "requirements-test.lock"),
  resolve(transcription, "requirements-test.lock"),
];
const markerPath = resolve(environment, ".requirements-test.lock");

const run = async (command: string[], cwd = repository) => {
  const child = Bun.spawn(command, {
    cwd,
    stderr: "inherit",
    stdout: "inherit",
  });
  const exitCode = await child.exited;
  if (exitCode !== 0) {
    throw new Error(`${command.join(" ")} exited with ${exitCode}`);
  }
};

if (!existsSync(environmentPython)) {
  await mkdir(environment, { recursive: true });
  await run([python, "-m", "venv", environment]);
}
const lock = (await Promise.all(lockPaths.map((path) => readFile(path, "utf8")))).join("\n");
const installedLock = existsSync(markerPath)
  ? await readFile(markerPath, "utf8")
  : undefined;
if (installedLock !== lock) {
  for (const lockPath of lockPaths) {
    await run([environmentPython, "-m", "pip", "install", "--disable-pip-version-check", "-r", lockPath]);
  }
  await writeFile(markerPath, lock);
}
await run([environmentPython, "-m", "unittest", "test_worker.py"], kokoro);
await run([environmentPython, "-m", "pytest", "test_worker.py", "-q"], transcription);
