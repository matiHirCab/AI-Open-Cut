import { rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { vi } from "vitest";
import speechFixture from "../../../../contracts/speech-provider-v1.json";
import { BridgeError } from "../../src/headless";
import { speechVoiceListSchema, ttsStatusSchema } from "../../src/schemas";
import type {
  SpeechSynthesisRequest,
  SpeechSynthesizer,
} from "../../src/speech";

// Real original PCM16 WAVs. Timings are actual sample counts, not alignment.
export const syntheticNarrationWav = (
  durationMs: number,
  frequency: number
) => {
  const samples = durationMs * 24;
  const wav = Buffer.alloc(44 + samples * 2);
  wav.write("RIFF");
  wav.writeUInt32LE(wav.length - 8, 4);
  wav.write("WAVEfmt ", 8);
  wav.writeUInt32LE(16, 16);
  wav.writeUInt16LE(1, 20);
  wav.writeUInt16LE(1, 22);
  wav.writeUInt32LE(24_000, 24);
  wav.writeUInt32LE(48_000, 28);
  wav.writeUInt16LE(2, 32);
  wav.writeUInt16LE(16, 34);
  wav.write("data", 36);
  wav.writeUInt32LE(samples * 2, 40);
  for (let n = 0; n < samples; n += 1) {
    wav.writeInt16LE(
      Math.round(Math.sin((n * frequency * 2 * Math.PI) / 24_000) * 4096),
      44 + n * 2
    );
  }
  return wav;
};

export const createNarrationTestProvider = (directory: string) => {
  const requests: SpeechSynthesisRequest[] = [];
  const cleaned: string[] = [];
  const control: {
    before?:
      | ((number: number, signal?: AbortSignal) => Promise<void>)
      | undefined;
    failAt?: number | undefined;
  } = {};
  const closed = vi.fn(() => Promise.resolve());
  const provider: SpeechSynthesizer = {
    cancel: vi.fn(),
    cleanup: async (path) => {
      cleaned.push(path);
      await rm(path, { force: true });
    },
    close: closed,
    listVoices: async () => {
      await Promise.resolve();
      return speechVoiceListSchema.parse(structuredClone(speechFixture.voices));
    },
    queueStatus: () => ({
      active: 0,
      concurrency: 1,
      fairness: "fifo",
      maxQueued: 4,
      queued: 0,
    }),
    status: async () => {
      await Promise.resolve();
      return ttsStatusSchema.parse(structuredClone(speechFixture.status));
    },
    synthesize: async (request, signal) => {
      requests.push(structuredClone(request));
      const count = requests.length;
      await control.before?.(count, signal);
      if (signal?.aborted) {
        throw new BridgeError("JOB_CANCELLED", "Synthetic inference cancelled");
      }
      if (control.failAt === count) {
        throw new BridgeError(
          "TTS_PROVIDER_FAILED",
          "Synthetic bounded failure"
        );
      }
      const durationMs = count % 2 === 1 ? 400 : 700;
      const path = join(directory, `owned-${count}.wav`);
      await writeFile(
        path,
        syntheticNarrationWav(durationMs, count % 2 === 1 ? 220 : 440)
      );
      return {
        durationMs,
        modelId: speechFixture.status.modelId,
        modelVersion: speechFixture.status.modelVersion,
        outputPath: path,
        providerId: speechFixture.status.providerId,
        request: structuredClone(request),
        sampleRateHz: 24_000,
      };
    },
  };
  return { cleaned, closed, control, provider, requests };
};
