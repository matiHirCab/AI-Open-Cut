import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { afterEach, expect, it } from "vitest";

import {
  assembleRuntimePackage,
  verifyRuntimePackage,
} from "../scripts/package-runtime";

let temporary = "";
const LINK_FAILURE = /escapes|symbolic link/u;

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

const fixture = async () => {
  temporary = await mkdtemp(join(tmpdir(), "opencut-package-negative-"));
  const input = {
    bridge: join(temporary, "bridge"),
    headless: join(temporary, "headless"),
    transcriptionWorker: join(temporary, "worker.py"),
    worker: join(temporary, "worker.py"),
  };
  await Promise.all([
    writeFile(input.bridge, "bridge"),
    writeFile(input.headless, "headless"),
    writeFile(input.worker, "worker"),
  ]);
  const destination = join(temporary, "runtime");
  const manifest = await assembleRuntimePackage(destination, input);
  return { destination, input, manifest };
};

it.each([
  null,
  [],
  { files: [], version: 2 },
  { files: "wrong", version: 1 },
  { files: [], unknown: true, version: 1 },
])("rejects malformed/future manifest %j", async (value) => {
  const { destination } = await fixture();
  await writeFile(join(destination, "manifest.json"), JSON.stringify(value));
  await expect(verifyRuntimePackage(destination)).rejects.toThrow("manifest");
});

it.each([
  "../bridge",
  "/bridge",
  "C:/bridge",
  "//server/bridge",
  "a\\bridge",
  "./bridge",
  "a/../bridge",
  "bridge.",
  "bridge ",
  "NUL.exe",
  "COM1",
  "COM¹",
  "LPT².exe",
  "a\u0000b",
  "a:b",
  "kokoro-tts/worker.py",
  "HEADLESS",
  "manifest.json",
])("rejects nonportable/colliding path %j", async (path) => {
  const { destination, manifest } = await fixture();
  const [entry] = manifest.files;
  if (!entry) {
    throw new Error("missing fixture entry");
  }
  entry.path = path;
  await writeFile(join(destination, "manifest.json"), JSON.stringify(manifest));
  await expect(verifyRuntimePackage(destination)).rejects.toThrow("portable");
});

it.each([
  { path: 1 },
  { sha256: "incorrect" },
  { sizeBytes: -1 },
  { sizeBytes: 1.5 },
  { sizeBytes: Number.MAX_SAFE_INTEGER + 1 },
  { extra: true },
])("rejects invalid entry %j", async (patch) => {
  const { destination, manifest } = await fixture();
  Object.assign(manifest.files[0] ?? {}, patch);
  await writeFile(join(destination, "manifest.json"), JSON.stringify(manifest));
  await expect(verifyRuntimePackage(destination)).rejects.toThrow("manifest");
});

it("bounds manifest reads", async () => {
  const { destination } = await fixture();
  await writeFile(
    join(destination, "manifest.json"),
    " ".repeat(16 * 1024 + 1)
  );
  await expect(verifyRuntimePackage(destination)).rejects.toThrow("size limit");
});

it.each(["file", "directory"])("rejects undeclared %s", async (kind) => {
  const { destination } = await fixture();
  if (kind === "directory") {
    await mkdir(join(destination, "extra"));
  } else {
    await writeFile(join(destination, "extra"), "extra");
  }
  await expect(verifyRuntimePackage(destination)).rejects.toThrow("undeclared");
});

it("rejects missing declared files", async () => {
  const { destination } = await fixture();
  await rm(join(destination, "headless"));
  await expect(verifyRuntimePackage(destination)).rejects.toThrow("missing");
});

it("rejects matching external worker directory links/junctions", async () => {
  const { destination } = await fixture();
  const external = join(temporary, "external");
  await mkdir(external);
  await writeFile(join(external, "worker.py"), "worker");
  await rm(join(destination, "kokoro-tts"), { recursive: true });
  await symlink(
    external,
    join(destination, "kokoro-tts"),
    process.platform === "win32" ? "junction" : "dir"
  );
  await expect(verifyRuntimePackage(destination)).rejects.toThrow(LINK_FAILURE);
});

it("rejects linked package roots", async () => {
  const { destination } = await fixture();
  const alias = join(temporary, "alias");
  await symlink(
    destination,
    alias,
    process.platform === "win32" ? "junction" : "dir"
  );
  await expect(verifyRuntimePackage(alias)).rejects.toThrow(
    "regular directory"
  );
});

it.each(["missing", "duplicate", "reserved"])(
  "preserves existing destination on %s preflight failure",
  async (failure) => {
    const { destination, input, manifest } = await fixture();
    if (failure === "missing") {
      input.worker = join(temporary, "missing.py");
    } else if (failure === "duplicate") {
      input.headless = input.bridge;
    } else {
      input.headless = join(temporary, "NUL.exe");
    }
    await expect(assembleRuntimePackage(destination, input)).rejects.toThrow();
    await expect(verifyRuntimePackage(destination)).resolves.toEqual(manifest);
  }
);

it("verifies a binary across multiple bounded hash chunks", async () => {
  const { destination, input } = await fixture();
  const contents = Buffer.alloc(256 * 1024 + 7, 0x5a);
  await writeFile(input.bridge, contents);
  const manifest = await assembleRuntimePackage(destination, input);
  await expect(verifyRuntimePackage(destination)).resolves.toEqual(manifest);
  await writeFile(
    join(destination, "bridge"),
    Buffer.alloc(contents.length, 0x5b)
  );
  await expect(verifyRuntimePackage(destination)).rejects.toThrow(
    "checksum mismatch"
  );
});

it("rejects size drift even with the correct digest", async () => {
  const { destination, manifest } = await fixture();
  const [entry] = manifest.files;
  if (!entry) {
    throw new Error("missing fixture entry");
  }
  entry.sizeBytes += 1;
  await writeFile(join(destination, "manifest.json"), JSON.stringify(manifest));
  await expect(verifyRuntimePackage(destination)).rejects.toThrow(
    "checksum mismatch"
  );
});

it("rejects non-regular declared files", async () => {
  const { destination } = await fixture();
  await rm(join(destination, "headless"));
  await mkdir(join(destination, "headless"));
  await expect(verifyRuntimePackage(destination)).rejects.toThrow(
    "non-regular"
  );
});

it("rejects file links on POSIX and directory junctions on Windows", async () => {
  const { destination } = await fixture();
  if (process.platform === "win32") {
    const external = join(temporary, "worker-link");
    await mkdir(external);
    await writeFile(join(external, "worker.py"), "worker");
    await rm(join(destination, "faster-whisper"), { recursive: true });
    await symlink(external, join(destination, "faster-whisper"), "junction");
  } else {
    await rm(join(destination, "bridge"));
    await symlink(join(temporary, "bridge"), join(destination, "bridge"));
  }
  await expect(verifyRuntimePackage(destination)).rejects.toThrow(LINK_FAILURE);
});

it("rejects manifest links/junctions before parsing", async () => {
  const { destination } = await fixture();
  const external = join(temporary, "manifest-source");
  if (process.platform === "win32") {
    await mkdir(external);
    await rm(join(destination, "manifest.json"));
    await symlink(external, join(destination, "manifest.json"), "junction");
  } else {
    await writeFile(
      external,
      await readFile(join(destination, "manifest.json"))
    );
    await rm(join(destination, "manifest.json"));
    await symlink(external, join(destination, "manifest.json"));
  }
  await expect(verifyRuntimePackage(destination)).rejects.toThrow(
    "non-regular"
  );
});

it.each(["kokoro-tts", "KOKORO-TTS", "faster-whisper"])(
  "preserves destination on worker-directory name collision %s",
  async (name) => {
    const { destination, input, manifest } = await fixture();
    input.bridge = join(temporary, name);
    await writeFile(input.bridge, "collision");
    await expect(assembleRuntimePackage(destination, input)).rejects.toThrow(
      "portable"
    );
    await expect(verifyRuntimePackage(destination)).resolves.toEqual(manifest);
  }
);

it("rejects extra files that resemble a declared path after separator conversion", async () => {
  const { destination } = await fixture();
  const extra =
    process.platform === "win32"
      ? join(destination, "kokoro-tts", "extra.py")
      : join(destination, "kokoro-tts\\worker.py");
  await writeFile(extra, "worker");
  await expect(verifyRuntimePackage(destination)).rejects.toThrow("undeclared");
});
