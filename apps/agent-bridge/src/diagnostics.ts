import { constants, existsSync } from "node:fs";
import { access, realpath, stat } from "node:fs/promises";
import { delimiter, isAbsolute, join, resolve } from "node:path";

import type { BridgeConfig } from "./config";

const executableCandidate = (value: string) => {
  if (isAbsolute(value)) {
    return resolve(value);
  }
  const extensions =
    process.platform === "win32"
      ? (process.env.PATHEXT ?? ".EXE;.CMD;.BAT").split(";")
      : [""];
  for (const directory of (process.env.PATH ?? "").split(delimiter)) {
    for (const extension of extensions) {
      const candidate = join(
        directory,
        value.endsWith(extension.toLowerCase())
          ? value
          : `${value}${extension.toLowerCase()}`
      );
      if (existsSync(candidate)) {
        return resolve(candidate);
      }
    }
  }
  return value;
};

const diagnostic = async (
  configured: string,
  kind: "directory" | "executable" | "file"
) => {
  const candidate =
    kind === "executable"
      ? executableCandidate(configured)
      : resolve(configured);
  try {
    const resolvedPath = await realpath(candidate);
    const metadata = await stat(resolvedPath);
    const exists =
      kind === "directory" ? metadata.isDirectory() : metadata.isFile();
    await access(resolvedPath, constants.R_OK);
    let writable = false;
    if (kind === "directory") {
      await access(resolvedPath, constants.W_OK);
      writable = true;
    }
    let executable = false;
    if (kind === "executable") {
      await access(
        resolvedPath,
        process.platform === "win32" ? constants.R_OK : constants.X_OK
      );
      executable = true;
    }
    return {
      error: null,
      executable,
      exists,
      readable: true,
      ready:
        exists &&
        (kind !== "directory" || writable) &&
        (kind !== "executable" || executable),
      resolvedPath,
      writable,
    };
  } catch {
    return {
      error:
        kind === "directory"
          ? "Directory is missing or inaccessible"
          : "File is missing or inaccessible",
      executable: false,
      exists: false,
      readable: false,
      ready: false,
      resolvedPath: candidate,
      writable: false,
    };
  }
};

export const runtimePathDiagnostics = async (
  config: Partial<BridgeConfig>
) => ({
  allowedMediaDirectories: await Promise.all(
    (config.allowedMediaDirectories ?? []).map(
      async (path) => await diagnostic(path, "directory")
    )
  ),
  exportsDirectory: await diagnostic(
    config.exportsDirectory ??
      join(config.configRoot ?? process.cwd(), "local-data", "exports"),
    "directory"
  ),
  ffmpeg: await diagnostic(config.ffmpegPath ?? "ffmpeg", "executable"),
  ffprobe: await diagnostic(config.ffprobePath ?? "ffprobe", "executable"),
  generatedMediaDirectories: await Promise.all(
    (config.generatedMediaDirectories ?? []).map(
      async (path) => await diagnostic(path, "directory")
    )
  ),
  kokoro: {
    modelDirectory: await diagnostic(
      config.ttsModelDirectory ??
        join(
          config.configRoot ?? process.cwd(),
          "local-data",
          "kokoro",
          "model"
        ),
      "directory"
    ),
    python: await diagnostic(config.ttsPythonPath ?? "python", "executable"),
    workDirectory: await diagnostic(
      config.ttsWorkDirectory ??
        join(
          config.configRoot ?? process.cwd(),
          "local-data",
          "kokoro",
          "work"
        ),
      "directory"
    ),
    worker: await diagnostic(
      config.ttsWorkerPath ??
        join(
          config.configRoot ?? process.cwd(),
          "apps",
          "kokoro-tts",
          "worker.py"
        ),
      "file"
    ),
  },
  projectsDirectory: await diagnostic(
    config.projectsDirectory ??
      join(config.configRoot ?? process.cwd(), "local-data", "projects"),
    "directory"
  ),
});
