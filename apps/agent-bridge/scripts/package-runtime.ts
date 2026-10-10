import { createHash } from "node:crypto";
import {
  chmod,
  lstat,
  mkdir,
  open,
  readdir,
  readFile,
  realpath,
  rm,
  writeFile,
} from "node:fs/promises";
import {
  basename,
  dirname,
  isAbsolute,
  join,
  relative,
  resolve,
  sep,
} from "node:path";

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

const MANIFEST_LIMIT = 16 * 1024;
const INVALID_CHARACTERS = /[\\<>:"|?*]/u;
const INVALID_TRAILING = /[. ]$/u;
const RESERVED_NAME = /^(con|prn|aux|nul|com[1-9¹²³]|lpt[1-9¹²³])(?:\.|$)/iu;
const SHA256 = /^[a-f0-9]{64}$/u;
const sequence = <T>(
  values: readonly T[],
  action: (value: T) => Promise<void>
) =>
  values.reduce(
    (previous, value) => previous.then(() => action(value)),
    Promise.resolve()
  );
const WORKER_PATHS = ["kokoro-tts/worker.py", "faster-whisper/worker.py"];

const recordWithKeys = (value: unknown, keys: string[]) =>
  typeof value === "object" &&
  value !== null &&
  !Array.isArray(value) &&
  Object.keys(value).length === keys.length &&
  keys.every((key) => Object.hasOwn(value, key));

const portablePath = (path: string) => {
  const segments = path.split("/");
  return segments.every(
    (segment) =>
      segment.length > 0 &&
      segment !== "." &&
      segment !== ".." &&
      !INVALID_CHARACTERS.test(segment) &&
      ![...segment].some(
        (character) =>
          character.charCodeAt(0) < 32 || character.charCodeAt(0) === 127
      ) &&
      !INVALID_TRAILING.test(segment) &&
      !RESERVED_NAME.test(segment)
  );
};

const validatePaths = (paths: string[]) => {
  const canonical = paths.map((path) => path.normalize("NFC").toLowerCase());
  if (
    paths.length !== 4 ||
    paths.some((path) => !portablePath(path)) ||
    new Set(canonical).size !== paths.length ||
    canonical.includes("manifest.json") ||
    canonical.some((path) => ["kokoro-tts", "faster-whisper"].includes(path)) ||
    !WORKER_PATHS.every((path) => paths.includes(path)) ||
    paths.filter((path) => !path.includes("/")).length !== 2
  ) {
    throw new Error("Runtime manifest has invalid portable file roles");
  }
};

const parseManifest = (value: unknown): RuntimeManifest => {
  if (!recordWithKeys(value, ["files", "version"])) {
    throw new Error("Runtime manifest has an unsupported shape");
  }
  const manifest = value as Record<string, unknown>;
  if (manifest.version !== 1 || !Array.isArray(manifest.files)) {
    throw new Error("Runtime manifest has an unsupported shape");
  }
  for (const file of manifest.files) {
    if (!recordWithKeys(file, ["path", "sha256", "sizeBytes"])) {
      throw new Error("Runtime manifest has an unsupported entry shape");
    }
    const entry = file as Record<string, unknown>;
    if (
      typeof entry.path !== "string" ||
      typeof entry.sha256 !== "string" ||
      !SHA256.test(entry.sha256) ||
      typeof entry.sizeBytes !== "number" ||
      !Number.isSafeInteger(entry.sizeBytes) ||
      entry.sizeBytes < 0
    ) {
      throw new Error("Runtime manifest has invalid entry values");
    }
  }
  const result = value as RuntimeManifest;
  validatePaths(result.files.map((entry) => entry.path));
  return result;
};

const regularFile = async (path: string) => {
  const facts = await lstat(path);
  if (!facts.isFile() || facts.isSymbolicLink()) {
    throw new Error(`Runtime package contains a non-regular file: ${path}`);
  }
  return facts;
};

export const assembleRuntimePackage = async (
  destination: string,
  sources: RuntimePackageSources
) => {
  const roles = [
    { destination: basename(sources.bridge), source: sources.bridge },
    { destination: basename(sources.headless), source: sources.headless },
    { destination: "kokoro-tts/worker.py", source: sources.worker },
    {
      destination: "faster-whisper/worker.py",
      source: sources.transcriptionWorker,
    },
  ];
  validatePaths(roles.map((file) => file.destination));
  const files = await Promise.all(
    roles.map(async (file) => ({
      ...file,
      contents: await readFile(file.source),
    }))
  );
  await rm(destination, { force: true, recursive: true });
  await mkdir(destination, { recursive: true });
  const entries: RuntimeManifestEntry[] = [];
  await sequence(files, async (file) => {
    const target = join(destination, file.destination);
    await mkdir(dirname(target), { recursive: true });
    await writeFile(target, file.contents);
    if (process.platform !== "win32" && !file.destination.includes("/")) {
      await chmod(target, 0o755);
    }
    const contents = await readFile(target);
    entries.push({
      path: file.destination.replaceAll("\\", "/"),
      sha256: digest(contents),
      sizeBytes: contents.byteLength,
    });
  });
  const manifest: RuntimeManifest = { files: entries, version: 1 };
  await writeFile(
    join(destination, "manifest.json"),
    `${JSON.stringify(manifest, null, 2)}\n`
  );
  return manifest;
};

export const verifyRuntimePackage = async (directory: string) => {
  const root = resolve(directory);
  const rootFacts = await lstat(root);
  if (!rootFacts.isDirectory() || rootFacts.isSymbolicLink()) {
    throw new Error("Runtime package root must be a regular directory");
  }
  const canonicalRoot = await realpath(root);
  const manifestPath = join(root, "manifest.json");
  const manifestFacts = await regularFile(manifestPath);
  if (manifestFacts.size > MANIFEST_LIMIT) {
    throw new Error("Runtime manifest exceeds the size limit");
  }
  const manifest = parseManifest(
    JSON.parse(await readFile(manifestPath, "utf8"))
  );
  const declared = new Set([
    "manifest.json",
    ...manifest.files.map((entry) => entry.path),
  ]);
  const directories = new Set(WORKER_PATHS.map((path) => dirname(path)));
  const actual = new Set<string>();
  const walk = async (directoryPath: string) => {
    await sequence(
      await readdir(directoryPath, { withFileTypes: true }),
      async (entry) => {
        const path = join(directoryPath, entry.name);
        const name = relative(root, path).split(sep).join("/");
        const facts = await lstat(path);
        const canonical = relative(canonicalRoot, await realpath(path));
        if (
          isAbsolute(canonical) ||
          canonical === ".." ||
          canonical.startsWith(`..${process.platform === "win32" ? "\\" : "/"}`)
        ) {
          throw new Error("Runtime manifest path escapes the package");
        }
        if (facts.isSymbolicLink()) {
          throw new Error(
            "Runtime package contains a symbolic link or junction"
          );
        }
        if (facts.isDirectory() && directories.has(name)) {
          await walk(path);
        } else if (facts.isFile() && declared.has(name)) {
          actual.add(name);
        } else {
          throw new Error(
            "Runtime package contains undeclared or non-regular inventory"
          );
        }
      }
    );
  };
  await walk(root);
  if (actual.size !== declared.size) {
    throw new Error("Runtime package is missing declared files");
  }
  await sequence(manifest.files, async (entry) => {
    const path = join(root, entry.path);
    const facts = await regularFile(path);
    if (
      process.platform !== "win32" &&
      !entry.path.includes("/") &&
      ![1, 8, 64].some((bit) => Math.floor(facts.mode / bit) % 2 === 1)
    ) {
      throw new Error(`Runtime executable is not executable: ${entry.path}`);
    }
    const handle = await open(path, "r");
    try {
      const hash = createHash("sha256");
      for await (const chunk of handle.createReadStream({
        autoClose: false,
        highWaterMark: 64 * 1024,
      })) {
        hash.update(chunk);
      }
      const openedFacts = await handle.stat();
      if (
        !openedFacts.isFile() ||
        openedFacts.size !== entry.sizeBytes ||
        hash.digest("hex") !== entry.sha256
      ) {
        throw new Error(`Runtime checksum mismatch: ${entry.path}`);
      }
    } finally {
      await handle.close();
    }
  });
  return manifest;
};
