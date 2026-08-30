import { constants, existsSync } from "node:fs";
import { access, realpath, stat } from "node:fs/promises";
import { delimiter, isAbsolute, join, resolve } from "node:path";

import type { BridgeConfig } from "./config";

const PATH_SEPARATOR_PATTERN = /[\\/]/u;

const environmentValue = (
  environment: Readonly<Record<string, string | undefined>>,
  name: string
) => {
  const exact = environment[name];
  if (exact !== undefined || process.platform !== "win32") {
    return exact;
  }
  const normalized = name.toLowerCase();
  return Object.entries(environment).find(
    ([key]) => key.toLowerCase() === normalized
  )?.[1];
};

export const resolveExecutablePath = (
  value: string,
  environment: Readonly<Record<string, string | undefined>> = process.env
) => {
  if (isAbsolute(value) || PATH_SEPARATOR_PATTERN.test(value)) {
    return resolve(value);
  }
  const configuredExtensions =
    process.platform === "win32"
      ? (environmentValue(environment, "PATHEXT") ?? ".EXE;.CMD;.BAT")
          .split(";")
          .filter(Boolean)
      : [""];
  const extensions =
    process.platform === "win32" &&
    configuredExtensions.some((extension) =>
      value.toLowerCase().endsWith(extension.toLowerCase())
    )
      ? [""]
      : configuredExtensions;
  for (const directory of (environmentValue(environment, "PATH") ?? "").split(
    delimiter
  )) {
    const normalizedDirectory = directory.replace(/^"|"$/gu, "");
    if (!normalizedDirectory) {
      continue;
    }
    for (const extension of extensions) {
      const candidate = join(normalizedDirectory, `${value}${extension}`);
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
  const candidate = kind === "executable" ? configured : resolve(configured);
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

export const runtimePathDiagnostics = async (config: Partial<BridgeConfig>) => {
  const executableDiagnostic = async (path: string) =>
    await diagnostic(
      resolveExecutablePath(path, config.environment ?? process.env),
      "executable"
    );
  return {
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
    ffmpeg: await executableDiagnostic(config.ffmpegPath ?? "ffmpeg"),
    ffprobe: await executableDiagnostic(config.ffprobePath ?? "ffprobe"),
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
      python: await executableDiagnostic(config.ttsPythonPath ?? "python"),
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
  };
};
