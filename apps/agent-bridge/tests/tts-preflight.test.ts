import { mkdirSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import { stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, join, resolve } from "node:path";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { loadBridgeConfig } from "../src/config";
import { resolveExecutablePath } from "../src/diagnostics";
import { KokoroSpeechSynthesizer } from "../src/tts";

vi.mock("node:fs/promises", async (importOriginal) => {
  const original = await importOriginal<typeof import("node:fs/promises")>();
  return { ...original, stat: vi.fn(original.stat) };
});
let root = "";
let provider: KokoroSpeechSynthesizer;
let environment: NodeJS.ProcessEnv;
beforeEach(() => {
  root = mkdtempSync(join(tmpdir(), "opencut-tts-preflight-"));
  mkdirSync(join(root, "model"));
  mkdirSync(join(root, "work"));
  environment = {
    ...process.env,
    OPENCUT_KOKORO_MODEL_DIR: join(root, "model"),
    OPENCUT_KOKORO_PYTHON: process.env.OPENCUT_TEST_PYTHON ?? "python",
    OPENCUT_KOKORO_WORKER: resolve(
      import.meta.dirname,
      "fixtures/fake_tts_worker.py"
    ),
    OPENCUT_TTS_SYNTHESIS_TIMEOUT_MS: "500",
    OPENCUT_TTS_WORK_DIR: join(root, "work"),
  };
  provider = new KokoroSpeechSynthesizer(loadBridgeConfig(environment));
});
afterEach(async () => {
  await provider.close();
  rmSync(root, { force: true, recursive: true });
});

it("cancels during executable preflight before dispatch and releases the FIFO successor", async () => {
  await provider.status();
  const metadata = await stat(
    resolveExecutablePath(
      environment.OPENCUT_KOKORO_PYTHON ?? "python",
      environment
    )
  );
  const entered = Promise.withResolvers<void>();
  const release = Promise.withResolvers<void>();
  const inspection = vi.mocked(stat).mockImplementationOnce(async () => {
    entered.resolve();
    await release.promise;
    return metadata;
  });
  let successor: ReturnType<KokoroSpeechSynthesizer["synthesize"]> | undefined;
  const controller = new AbortController();
  const first = provider.synthesize(
    { language: "en-US", speed: 1, text: "hang", voiceId: "test_voice" },
    controller.signal
  );
  const rejected = expect(first).rejects.toMatchObject({
    code: "JOB_CANCELLED",
    retryable: true,
  });
  try {
    await entered.promise;
    const second = provider.synthesize({
      language: "en-US",
      speed: 1,
      text: "successor",
      voiceId: "test_voice",
    });
    successor = second;
    controller.abort();
    release.resolve();
    await rejected;
    const output = await second;
    expect(output.request.text).toBe("successor");
    expect(readdirSync(join(root, "work"))).toEqual([
      basename(output.outputPath),
    ]);
    await provider.cleanup(output.outputPath);
    expect(readdirSync(join(root, "work"))).toEqual([]);
  } finally {
    release.resolve();
    await Promise.allSettled([first, ...(successor ? [successor] : [])]);
    inspection.mockClear();
  }
}, 20_000);
