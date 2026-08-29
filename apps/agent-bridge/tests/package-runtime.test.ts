import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { afterEach, expect, it } from "vitest";

import {
  assembleRuntimePackage,
  verifyRuntimePackage,
} from "../scripts/package-runtime";

let temporary = "";

afterEach(async () => {
  if (temporary) {
    await rm(temporary, { force: true, recursive: true });
  }
});

it("assembles only allowlisted runtime files with verified checksums", async () => {
  temporary = await mkdtemp(join(tmpdir(), "opencut-package-test-"));
  const sources = join(temporary, "sources");
  await mkdir(sources);
  const bridge = join(sources, "opencut-agent-bridge.exe");
  const headless = join(sources, "opencut-headless.exe");
  const worker = join(sources, "worker.py");
  await Promise.all([
    writeFile(bridge, "bridge"),
    writeFile(headless, "headless"),
    writeFile(worker, "worker"),
    writeFile(join(sources, "setup.ps1"), "must not ship"),
  ]);
  const destination = join(temporary, "runtime");
  const manifest = await assembleRuntimePackage(destination, {
    bridge,
    headless,
    transcriptionWorker: worker,
    worker,
  });
  await expect(verifyRuntimePackage(destination)).resolves.toEqual(manifest);
  expect(manifest.files.map((entry) => entry.path)).toEqual([
    "opencut-agent-bridge.exe",
    "opencut-headless.exe",
    "kokoro-tts/worker.py",
    "faster-whisper/worker.py",
  ]);
  await expect(readFile(join(destination, "setup.ps1"))).rejects.toThrow();
});

it("rejects checksum drift", async () => {
  temporary = await mkdtemp(join(tmpdir(), "opencut-package-test-"));
  const bridge = join(temporary, "bridge");
  const headless = join(temporary, "headless");
  const worker = join(temporary, "worker.py");
  await Promise.all([
    writeFile(bridge, "bridge"),
    writeFile(headless, "headless"),
    writeFile(worker, "worker"),
  ]);
  const destination = join(temporary, "runtime");
  await assembleRuntimePackage(destination, {
    bridge,
    headless,
    transcriptionWorker: worker,
    worker,
  });
  await writeFile(join(destination, "bridge"), "changed");
  await expect(verifyRuntimePackage(destination)).rejects.toThrow(
    "checksum mismatch"
  );
});
