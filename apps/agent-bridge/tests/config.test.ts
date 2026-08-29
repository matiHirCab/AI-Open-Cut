import { isAbsolute, join, resolve } from "node:path";

import { expect, it } from "vitest";

import { loadBridgeConfig } from "../src/config";

it("resolves immutable defaults without mutating process environment", () => {
  const before = { ...process.env };
  const config = loadBridgeConfig({});
  expect(Object.isFrozen(config)).toBe(true);
  expect(Object.isFrozen(config.environment)).toBe(true);
  expect(config).toMatchObject({
    configRoot: resolve(process.cwd()),
    generatedArtifactTtlMs: 600_000,
    headlessRequestTimeoutMs: 600_000,
    httpHost: "127.0.0.1",
    httpPort: 3002,
    jobMaxCount: 1000,
    jobTtlMs: 3_600_000,
    transcriptionModelId: "small",
    transport: "stdio",
    ttsControlTimeoutMs: 10_000,
    ttsMaxQueued: 8,
    ttsSynthesisTimeoutMs: 300_000,
  });
  expect(isAbsolute(config.ttsWorkDirectory)).toBe(true);
  expect(process.env).toEqual(before);
  expect(config.environment).not.toHaveProperty("OPENCUT_KOKORO_DEVICE");
});

it("resolves every configured relative path from OPENCUT_CONFIG_ROOT", () => {
  const root = resolve("configuration-root");
  const config = loadBridgeConfig({
    OPENCUT_ALLOWED_MEDIA_DIRS: join("media", "imports"),
    OPENCUT_CONFIG_ROOT: root,
    OPENCUT_DEFAULT_FONT_PATH: join("fonts", "default.ttf"),
    OPENCUT_EXPORTS_DIR: "exports",
    OPENCUT_FFMPEG_PATH: join("tools", "ffmpeg"),
    OPENCUT_FFPROBE_PATH: join("tools", "ffprobe"),
    OPENCUT_GENERATED_MEDIA_DIRS: "generated",
    OPENCUT_HEADLESS_PATH: join("bin", "headless"),
    OPENCUT_KOKORO_MODEL_DIR: join("kokoro", "model"),
    OPENCUT_KOKORO_PYTHON: join("python", "python"),
    OPENCUT_KOKORO_WORKER: join("kokoro", "worker.py"),
    OPENCUT_PROJECTS_DIR: "projects",
    OPENCUT_TTS_WORK_DIR: join("kokoro", "work"),
  });
  expect(config).toMatchObject({
    configRoot: root,
    exportsDirectory: join(root, "exports"),
    headlessPath: join(root, "bin", "headless"),
    projectsDirectory: join(root, "projects"),
    ttsModelDirectory: join(root, "kokoro", "model"),
    ttsPythonPath: join(root, "python", "python"),
    ttsWorkDirectory: join(root, "kokoro", "work"),
    ttsWorkerPath: join(root, "kokoro", "worker.py"),
  });
  expect(config.generatedMediaDirectories).toEqual([
    join(root, "generated"),
    join(root, "kokoro", "work"),
  ]);
  expect(config.environment.OPENCUT_DEFAULT_FONT_PATH).toBe(
    join(root, "fonts", "default.ttf")
  );
  expect(config.environment.OPENCUT_FFMPEG_PATH).toBe(
    join(root, "tools", "ffmpeg")
  );
});

it("rejects invalid numeric lifecycle configuration", () => {
  expect(() => loadBridgeConfig({ OPENCUT_JOB_MAX_COUNT: "0" })).toThrowError(
    expect.objectContaining({ code: "INVALID_CONFIGURATION" })
  );
});

it("validates the structured log level", () => {
  expect(loadBridgeConfig({ OPENCUT_LOG_LEVEL: "debug" }).logLevel).toBe(
    "debug"
  );
  expect(() => loadBridgeConfig({ OPENCUT_LOG_LEVEL: "verbose" })).toThrowError(
    expect.objectContaining({ code: "INVALID_CONFIGURATION" })
  );
});

it("requires authentication for non-loopback HTTP", () => {
  expect(() =>
    loadBridgeConfig({
      OPENCUT_HTTP_HOST: "0.0.0.0",
      OPENCUT_TRANSPORT: "http",
    })
  ).toThrowError(expect.objectContaining({ code: "INVALID_CONFIGURATION" }));
  expect(
    loadBridgeConfig({
      OPENCUT_HTTP_AUTH_TOKEN: "secret",
      OPENCUT_HTTP_HOST: "0.0.0.0",
      OPENCUT_TRANSPORT: "http",
    })
  ).toMatchObject({
    httpAuthToken: "secret",
    httpHost: "0.0.0.0",
    transport: "http",
  });
});
