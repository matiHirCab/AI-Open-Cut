import { existsSync } from "node:fs";
import { delimiter, dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { BridgeError } from "./headless";
import type { LogLevel } from "./logger";

const PATH_SEPARATOR_PATTERN = /[\\/]/u;

export interface BridgeConfig {
  readonly configRoot: string;
  readonly environment: Readonly<Record<string, string>>;
  readonly exportsDirectory: string | undefined;
  readonly generatedArtifactTtlMs: number;
  readonly generatedMediaDirectories: readonly string[];
  readonly headlessArguments: readonly string[];
  readonly headlessPath: string;
  readonly headlessRequestTimeoutMs: number;
  readonly httpAllowedOrigins: readonly string[];
  readonly httpAuthToken: string | undefined;
  readonly httpHost: string;
  readonly httpMaxBodyBytes: number;
  readonly httpPort: number;
  readonly jobMaxCount: number;
  readonly jobTtlMs: number;
  readonly logLevel: LogLevel;
  readonly projectsDirectory: string | undefined;
  readonly speechProviderId: string;
  readonly transcriptionControlTimeoutMs: number;
  readonly transcriptionMaxQueued: number;
  readonly transcriptionModelDirectory: string;
  readonly transcriptionModelId: string;
  readonly transcriptionPythonPath: string;
  readonly transcriptionTimeoutMs: number;
  readonly transcriptionWorkerPath: string;
  readonly transport: "stdio" | "http";
  readonly ttsControlTimeoutMs: number;
  readonly ttsMaxQueued: number;
  readonly ttsModelDirectory: string;
  readonly ttsPythonPath: string;
  readonly ttsSynthesisTimeoutMs: number;
  readonly ttsWorkDirectory: string;
  readonly ttsWorkerPath: string;
}

const positiveInteger = (
  environment: NodeJS.ProcessEnv,
  name: string,
  defaultValue: number
) => {
  const raw = environment[name];
  if (raw === undefined) {
    return defaultValue;
  }
  const value = Number(raw);
  if (!(Number.isSafeInteger(value) && value > 0)) {
    throw new BridgeError(
      "INVALID_CONFIGURATION",
      `${name} must be a positive integer`
    );
  }
  return value;
};

const configuredAbsolute = (
  environment: NodeJS.ProcessEnv,
  name: string,
  fallback: string,
  configRoot: string
) => {
  const value = environment[name] ?? fallback;
  return isAbsolute(value) ? resolve(value) : resolve(configRoot, value);
};

const configuredExecutable = (
  environment: NodeJS.ProcessEnv,
  name: string,
  fallback: string,
  configRoot: string
) => {
  const configured = environment[name];
  if (configured === undefined) {
    return fallback;
  }
  return isAbsolute(configured) || !PATH_SEPARATOR_PATTERN.test(configured)
    ? configured
    : resolve(configRoot, configured);
};

const resolveHeadlessPath = (
  environment: NodeJS.ProcessEnv,
  configRoot: string
) => {
  const configured = environment.OPENCUT_HEADLESS_PATH;
  if (configured) {
    return isAbsolute(configured)
      ? resolve(configured)
      : resolve(configRoot, configured);
  }
  const executable =
    process.platform === "win32" ? "opencut-headless.exe" : "opencut-headless";
  const packagedSibling = resolve(dirname(process.execPath), executable);
  if (existsSync(packagedSibling)) {
    return packagedSibling;
  }
  const currentDirectory = dirname(fileURLToPath(import.meta.url));
  const sibling = resolve(currentDirectory, executable);
  return existsSync(sibling)
    ? sibling
    : resolve(currentDirectory, "../../../target/release", executable);
};

const resolveWorkerPath = (
  environment: NodeJS.ProcessEnv,
  configRoot: string
) => {
  const configured = environment.OPENCUT_KOKORO_WORKER;
  if (configured) {
    return isAbsolute(configured)
      ? resolve(configured)
      : resolve(configRoot, configured);
  }
  const currentDirectory = dirname(fileURLToPath(import.meta.url));
  const packaged = resolve(
    dirname(process.execPath),
    "kokoro-tts",
    "worker.py"
  );
  const source = resolve(currentDirectory, "../../kokoro-tts/worker.py");
  return existsSync(packaged) ? packaged : source;
};

const resolveTranscriptionWorkerPath = (
  environment: NodeJS.ProcessEnv,
  configRoot: string
) => {
  const configured = environment.OPENCUT_TRANSCRIPTION_WORKER;
  if (configured) {
    return isAbsolute(configured)
      ? resolve(configured)
      : resolve(configRoot, configured);
  }
  const currentDirectory = dirname(fileURLToPath(import.meta.url));
  const packaged = resolve(
    dirname(process.execPath),
    "faster-whisper",
    "worker.py"
  );
  const source = resolve(currentDirectory, "../../faster-whisper/worker.py");
  return existsSync(packaged) ? packaged : source;
};

const resolveHttpSettings = (source: NodeJS.ProcessEnv) => {
  const transport = source.OPENCUT_TRANSPORT ?? "stdio";
  if (transport !== "stdio" && transport !== "http") {
    throw new BridgeError(
      "INVALID_CONFIGURATION",
      "OPENCUT_TRANSPORT must be stdio or http"
    );
  }
  const httpHost = source.OPENCUT_HTTP_HOST ?? "127.0.0.1";
  const httpAuthToken = source.OPENCUT_HTTP_AUTH_TOKEN;
  if (
    transport === "http" &&
    !["127.0.0.1", "::1", "localhost"].includes(httpHost) &&
    !httpAuthToken
  ) {
    throw new BridgeError(
      "INVALID_CONFIGURATION",
      "A bearer token is required for non-loopback HTTP binds"
    );
  }
  return { httpAuthToken, httpHost, transport } as const;
};

export const loadBridgeConfig = (
  source: NodeJS.ProcessEnv = process.env
): BridgeConfig => {
  const configRoot = source.OPENCUT_CONFIG_ROOT
    ? configuredAbsolute(
        source,
        "OPENCUT_CONFIG_ROOT",
        process.cwd(),
        process.cwd()
      )
    : resolve(process.cwd());
  const defaultRoot = resolve(configRoot, "local-data", "kokoro");
  const ttsModelDirectory = configuredAbsolute(
    source,
    "OPENCUT_KOKORO_MODEL_DIR",
    join(defaultRoot, "model"),
    configRoot
  );
  const ttsWorkDirectory = configuredAbsolute(
    source,
    "OPENCUT_TTS_WORK_DIR",
    join(defaultRoot, "work"),
    configRoot
  );
  const ttsPythonPath = configuredExecutable(
    source,
    "OPENCUT_KOKORO_PYTHON",
    process.platform === "win32"
      ? join(defaultRoot, "venv", "Scripts", "python.exe")
      : join(defaultRoot, "venv", "bin", "python"),
    configRoot
  );
  const transcriptionRoot = resolve(configRoot, "local-data", "transcription");
  const transcriptionModelDirectory = configuredAbsolute(
    source,
    "OPENCUT_TRANSCRIPTION_MODEL_DIR",
    join(transcriptionRoot, "model"),
    configRoot
  );
  const transcriptionPythonPath = configuredExecutable(
    source,
    "OPENCUT_TRANSCRIPTION_PYTHON",
    process.platform === "win32"
      ? join(transcriptionRoot, "venv", "Scripts", "python.exe")
      : join(transcriptionRoot, "venv", "bin", "python"),
    configRoot
  );
  const projectsDirectory = source.OPENCUT_PROJECTS_DIR
    ? resolve(configRoot, source.OPENCUT_PROJECTS_DIR)
    : undefined;
  const exportsDirectory = source.OPENCUT_EXPORTS_DIR
    ? resolve(configRoot, source.OPENCUT_EXPORTS_DIR)
    : undefined;
  const generatedMediaDirectories = [
    ...(source.OPENCUT_GENERATED_MEDIA_DIRS?.split(delimiter)
      .filter(Boolean)
      .map((path) => resolve(configRoot, path)) ?? []),
    ttsWorkDirectory,
  ].filter((path, index, paths) => paths.indexOf(path) === index);
  const allowedMediaDirectories = source.OPENCUT_ALLOWED_MEDIA_DIRS?.split(
    delimiter
  )
    .filter(Boolean)
    .map((path) => resolve(configRoot, path));
  const { httpAuthToken, httpHost, transport } = resolveHttpSettings(source);
  const environment = Object.freeze({
    ...Object.fromEntries(
      Object.entries(source).filter(
        (entry): entry is [string, string] => entry[1] !== undefined
      )
    ),
    OPENCUT_CONFIG_ROOT: configRoot,
    OPENCUT_GENERATED_MEDIA_DIRS: generatedMediaDirectories.join(delimiter),
    OPENCUT_KOKORO_MODEL_DIR: ttsModelDirectory,
    OPENCUT_KOKORO_PYTHON: ttsPythonPath,
    OPENCUT_TTS_WORK_DIR: ttsWorkDirectory,
    ...(allowedMediaDirectories
      ? { OPENCUT_ALLOWED_MEDIA_DIRS: allowedMediaDirectories.join(delimiter) }
      : {}),
    ...(source.OPENCUT_DEFAULT_FONT_PATH
      ? {
          OPENCUT_DEFAULT_FONT_PATH: resolve(
            configRoot,
            source.OPENCUT_DEFAULT_FONT_PATH
          ),
        }
      : {}),
    ...(source.OPENCUT_FFMPEG_PATH
      ? {
          OPENCUT_FFMPEG_PATH: configuredExecutable(
            source,
            "OPENCUT_FFMPEG_PATH",
            "ffmpeg",
            configRoot
          ),
        }
      : {}),
    ...(source.OPENCUT_FFPROBE_PATH
      ? {
          OPENCUT_FFPROBE_PATH: configuredExecutable(
            source,
            "OPENCUT_FFPROBE_PATH",
            "ffprobe",
            configRoot
          ),
        }
      : {}),
    ...(projectsDirectory ? { OPENCUT_PROJECTS_DIR: projectsDirectory } : {}),
    ...(exportsDirectory ? { OPENCUT_EXPORTS_DIR: exportsDirectory } : {}),
  });
  return Object.freeze({
    configRoot,
    environment,
    exportsDirectory,
    generatedArtifactTtlMs: positiveInteger(
      source,
      "OPENCUT_GENERATED_ARTIFACT_TTL_MS",
      600_000
    ),
    generatedMediaDirectories: Object.freeze(generatedMediaDirectories),
    headlessArguments: [],
    headlessPath: resolveHeadlessPath(source, configRoot),
    headlessRequestTimeoutMs: positiveInteger(
      source,
      "OPENCUT_HEADLESS_REQUEST_TIMEOUT_MS",
      600_000
    ),
    httpAllowedOrigins: Object.freeze(
      (source.OPENCUT_HTTP_ALLOWED_ORIGINS ?? "")
        .split(",")
        .map((value) => value.trim())
        .filter(Boolean)
    ),
    httpAuthToken,
    httpHost,
    httpMaxBodyBytes: positiveInteger(
      source,
      "OPENCUT_HTTP_MAX_BODY_BYTES",
      1_048_576
    ),
    httpPort: positiveInteger(source, "OPENCUT_HTTP_PORT", 3002),
    jobMaxCount: positiveInteger(source, "OPENCUT_JOB_MAX_COUNT", 1000),
    jobTtlMs: positiveInteger(source, "OPENCUT_JOB_TTL_MS", 3_600_000),
    logLevel: (() => {
      const value = source.OPENCUT_LOG_LEVEL ?? "info";
      if (!["error", "warn", "info", "debug"].includes(value)) {
        throw new BridgeError(
          "INVALID_CONFIGURATION",
          "OPENCUT_LOG_LEVEL is invalid"
        );
      }
      return value as LogLevel;
    })(),
    projectsDirectory,
    speechProviderId: source.OPENCUT_SPEECH_PROVIDER ?? "kokoro",
    transcriptionControlTimeoutMs: positiveInteger(
      source,
      "OPENCUT_TRANSCRIPTION_CONTROL_TIMEOUT_MS",
      10_000
    ),
    transcriptionMaxQueued: positiveInteger(
      source,
      "OPENCUT_TRANSCRIPTION_MAX_QUEUED",
      4
    ),
    transcriptionModelDirectory,
    transcriptionModelId: source.OPENCUT_TRANSCRIPTION_MODEL ?? "small",
    transcriptionPythonPath,
    transcriptionTimeoutMs: positiveInteger(
      source,
      "OPENCUT_TRANSCRIPTION_TIMEOUT_MS",
      900_000
    ),
    transcriptionWorkerPath: resolveTranscriptionWorkerPath(source, configRoot),
    transport,
    ttsControlTimeoutMs: positiveInteger(
      source,
      "OPENCUT_TTS_CONTROL_TIMEOUT_MS",
      10_000
    ),
    ttsMaxQueued: positiveInteger(source, "OPENCUT_TTS_MAX_QUEUED", 8),
    ttsModelDirectory,
    ttsPythonPath,
    ttsSynthesisTimeoutMs: positiveInteger(
      source,
      "OPENCUT_TTS_SYNTHESIS_TIMEOUT_MS",
      300_000
    ),
    ttsWorkDirectory,
    ttsWorkerPath: resolveWorkerPath(source, configRoot),
  });
};
