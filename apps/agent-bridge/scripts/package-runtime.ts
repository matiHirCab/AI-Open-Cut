import { createHash } from "node:crypto";
import {
  chmod,
  mkdir,
  readdir,
  readFile,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import { basename, dirname, join, relative, resolve } from "node:path";

export interface RuntimePackageSources {
  bridge: string;
  headless: string;
  transcriptionWorker: string;
  worker: string;
}

export interface RuntimeManifestEntry {
  path: string;
  sha256: string;
  sizeBytes: number;
}

export interface RuntimeManifest {
  files: RuntimeManifestEntry[];
  version: 1;
}

const digest = (contents: Uint8Array) =>
  createHash("sha256").update(contents).digest("hex");

export const assembleRuntimePackage = async (
  destination: string,
  sources: RuntimePackageSources
) => {
  const files = await Promise.all(
    [
      { destination: basename(sources.bridge), source: sources.bridge },
      { destination: basename(sources.headless), source: sources.headless },
      { destination: "kokoro-tts/worker.py", source: sources.worker },
      {
        destination: "faster-whisper/worker.py",
        source: sources.transcriptionWorker,
      },
    ].map(async (file) => ({ ...file, contents: await readFile(file.source) }))
  );
  await rm(destination, { force: true, recursive: true });
  await mkdir(destination, { recursive: true });
  const entries: RuntimeManifestEntry[] = [];
  for (const file of files) {
    const target = join(destination, file.destination);
    await mkdir(dirname(target), { recursive: true });
    await writeFile(target, file.contents);
    if (process.platform !== "win32" && !target.endsWith(".py")) {
      await chmod(target, 0o755);
    }
    const contents = await readFile(target);
    entries.push({
      path: file.destination.replaceAll("\\", "/"),
      sha256: digest(contents),
      sizeBytes: contents.byteLength,
    });
  }
  const manifest: RuntimeManifest = { files: entries, version: 1 };
  await writeFile(
    join(destination, "manifest.json"),
    `${JSON.stringify(manifest, null, 2)}\n`
  );
  return manifest;
};

export const verifyRuntimePackage = async (directory: string) => {
  const manifest = JSON.parse(
    await readFile(join(directory, "manifest.json"), "utf8")
  ) as RuntimeManifest;
  if (manifest.version !== 1 || manifest.files.length !== 4) {
    throw new Error("Runtime manifest has an unsupported shape");
  }
  const declared = new Set(["manifest.json"]);
  for (const entry of manifest.files) {
    const path = resolve(directory, entry.path);
    if (relative(directory, path).startsWith("..")) {
      throw new Error("Runtime manifest path escapes the package");
    }
    const contents = await readFile(path);
    const facts = await stat(path);
    if (facts.size !== entry.sizeBytes || digest(contents) !== entry.sha256) {
      throw new Error(`Runtime checksum mismatch: ${entry.path}`);
    }
    declared.add(entry.path.replaceAll("\\", "/"));
  }
  const actual: string[] = [];
  const walk = async (directoryPath: string) => {
    for (const entry of await readdir(directoryPath, { withFileTypes: true })) {
      const path = join(directoryPath, entry.name);
      if (entry.isDirectory()) {
        await walk(path);
      } else {
        actual.push(relative(directory, path).replaceAll("\\", "/"));
      }
    }
  };
  await walk(directory);
  if (
    actual.length !== declared.size ||
    actual.some((path) => !declared.has(path))
  ) {
    throw new Error("Runtime package contains undeclared files");
  }
  return manifest;
};
