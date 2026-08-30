import { existsSync, mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";

import { afterEach, beforeEach, expect, it } from "vitest";

import { loadBridgeConfig } from "../src/config";
import { resolveExecutablePath } from "../src/diagnostics";
import { KokoroSpeechSynthesizer } from "../src/tts";

const environmentNames = [
  "OPENCUT_KOKORO_MODEL_DIR",
  "OPENCUT_KOKORO_PYTHON",
  "OPENCUT_KOKORO_WORKER",
  "OPENCUT_TTS_WORK_DIR",
] as const;
const originalEnvironment = Object.fromEntries(
  environmentNames.map((name) => [name, process.env[name]])
);
let root = "";
let provider: KokoroSpeechSynthesizer | undefined;

beforeEach(() => {
  root = mkdtempSync(join(tmpdir(), "opencut-speech-provider-"));
  const model = join(root, "model");
  const work = join(root, "work");
  mkdirSync(model);
  mkdirSync(work);
  process.env.OPENCUT_KOKORO_MODEL_DIR = model;
  process.env.OPENCUT_TTS_WORK_DIR = work;
  process.env.OPENCUT_KOKORO_PYTHON =
    process.env.OPENCUT_TEST_PYTHON ?? "python";
  process.env.OPENCUT_KOKORO_WORKER = resolve(
    import.meta.dirname,
    "fixtures/fake_tts_worker.py"
  );
  provider = new KokoroSpeechSynthesizer(loadBridgeConfig());
});

afterEach(async () => {
  await provider?.close();
  for (const name of environmentNames) {
    const original = originalEnvironment[name];
    if (original === undefined) {
      delete process.env[name];
    } else {
      process.env[name] = original;
    }
  }
  rmSync(root, { force: true, recursive: true });
});

it("uses worker-advertised provider capabilities and synthesis metadata", async () => {
  if (!provider) {
    throw new Error("provider was not created");
  }
  const status = await provider.status();
  const voices = await provider.listVoices();
  expect(status).toMatchObject({
    modelId: "fake/model",
    providerId: "fake-speech",
    sampleRateHz: 24_000,
  });
  expect(voices.map((voice) => voice.id)).toEqual(status.voices);

  const generated = await provider.synthesize({
    language: "en-US",
    speed: 1,
    text: "hello",
    voiceId: "test_voice",
  });
  expect(generated).toMatchObject({
    modelId: status.modelId,
    providerId: status.providerId,
    sampleRateHz: status.sampleRateHz,
  });
  expect(existsSync(generated.outputPath)).toBe(true);
  await provider.cleanup(generated.outputPath);
  expect(existsSync(generated.outputPath)).toBe(false);
});

it("reports a work directory created by the first status call as ready", async () => {
  await provider?.close();
  rmSync(join(root, "work"), { force: true, recursive: true });
  provider = new KokoroSpeechSynthesizer(loadBridgeConfig());

  await expect(provider.status()).resolves.toMatchObject({
    paths: {
      workDirectory: {
        error: null,
        exists: true,
        readable: true,
        ready: true,
        writable: true,
      },
    },
    ready: true,
    startupError: null,
  });
});

it("resolves the configured bare Python executable through the child PATH", async () => {
  await provider?.close();
  const configuredPython = process.env.OPENCUT_TEST_PYTHON ?? "python";
  const resolvedPython = resolveExecutablePath(configuredPython, process.env);
  provider = new KokoroSpeechSynthesizer(
    loadBridgeConfig({
      ...process.env,
      OPENCUT_KOKORO_PYTHON: basename(resolvedPython),
      PATH: dirname(resolvedPython),
    })
  );

  const status = await provider.status();
  expect(status).toMatchObject({
    paths: { python: { ready: true } },
    ready: true,
  });
  expect(status.paths?.python.resolvedPath.toLowerCase()).toBe(
    resolvedPython.toLowerCase()
  );
});

it("cancels active provider work with a stable retryable error", async () => {
  if (!provider) {
    throw new Error("provider was not created");
  }
  const generation = provider.synthesize({
    language: "en-US",
    speed: 1,
    text: "hang",
    voiceId: "test_voice",
  });
  await new Promise((resolvePromise) => setTimeout(resolvePromise, 50));
  provider.cancel();
  await expect(generation).rejects.toMatchObject({
    code: "JOB_CANCELLED",
    retryable: true,
  });
});

it("times out synthesis and removes owned worker files", async () => {
  await provider?.close();
  provider = new KokoroSpeechSynthesizer(
    loadBridgeConfig({
      ...process.env,
      OPENCUT_TTS_SYNTHESIS_TIMEOUT_MS: "25",
    })
  );
  await expect(
    provider.synthesize({
      language: "en-US",
      speed: 1,
      text: "hang",
      voiceId: "test_voice",
    })
  ).rejects.toMatchObject({ code: "TTS_TIMEOUT", retryable: true });
  await new Promise((resolvePromise) => setTimeout(resolvePromise, 25));
  expect(existsSync(join(root, "work"))).toBe(true);
  await expect(
    (await import("node:fs/promises")).readdir(join(root, "work"))
  ).resolves.toEqual([]);
});

it("uses a bounded FIFO queue and continues after active cancellation", async () => {
  await provider?.close();
  provider = new KokoroSpeechSynthesizer(
    loadBridgeConfig({ ...process.env, OPENCUT_TTS_MAX_QUEUED: "1" })
  );
  const controller = new AbortController();
  const first = provider.synthesize(
    { language: "en-US", speed: 1, text: "hang", voiceId: "test_voice" },
    controller.signal
  );
  const second = provider.synthesize({
    language: "en-US",
    speed: 1,
    text: "second",
    voiceId: "test_voice",
  });
  await expect(
    provider.synthesize({
      language: "en-US",
      speed: 1,
      text: "third",
      voiceId: "test_voice",
    })
  ).rejects.toMatchObject({ code: "TTS_QUEUE_FULL", retryable: true });
  controller.abort();
  await expect(first).rejects.toMatchObject({ code: "JOB_CANCELLED" });
  await expect(second).resolves.toMatchObject({
    request: expect.objectContaining({ text: "second" }),
  });
});

it("handles partial worker lines without losing the response", async () => {
  await expect(
    provider?.synthesize({
      language: "en-US",
      speed: 1,
      text: "partial",
      voiceId: "test_voice",
    })
  ).resolves.toMatchObject({ request: { text: "partial" } });
});

it.each(["malformed", "exit"])(
  "turns %s worker termination into a stable retryable error",
  async (text) => {
    await expect(
      provider?.synthesize({
        language: "en-US",
        speed: 1,
        text,
        voiceId: "test_voice",
      })
    ).rejects.toMatchObject({
      code: "TTS_WORKER_TERMINATED",
      retryable: true,
    });
  }
);

it("reports worker startup errors and can continue with a replacement provider", async () => {
  await provider?.close();
  provider = new KokoroSpeechSynthesizer(
    loadBridgeConfig({
      ...process.env,
      OPENCUT_KOKORO_PYTHON: "definitely-missing-python-command",
    })
  );
  await expect(provider.status()).resolves.toMatchObject({
    ready: false,
    startupError: {
      code: "TTS_UNAVAILABLE",
      message: "Configured Kokoro Python executable is missing",
      retryable: false,
    },
  });
  await provider.close();
  provider = new KokoroSpeechSynthesizer(loadBridgeConfig());
  await expect(provider.status()).resolves.toMatchObject({ ready: true });
});

it("continues the FIFO queue after a failed generation", async () => {
  const first = provider?.synthesize({
    language: "en-US",
    speed: 1,
    text: "fail",
    voiceId: "test_voice",
  });
  const second = provider?.synthesize({
    language: "en-US",
    speed: 1,
    text: "after-failure",
    voiceId: "test_voice",
  });
  await expect(first).rejects.toMatchObject({ code: "TTS_SYNTHESIS_FAILED" });
  await expect(second).resolves.toMatchObject({
    request: { text: "after-failure" },
  });
});
