import { expect, it } from "vitest";

import { BridgeError, type HeadlessClient } from "../src/headless";
import type { Transcriber } from "../src/transcription";
import { TranscriptionApplicationService } from "../src/transcription";

const provider = (): Transcriber => ({
  close: async () => undefined,
  queueStatus: () => ({
    active: 0,
    concurrency: 1,
    fairness: "fifo",
    maxQueued: 4,
    queued: 0,
  }),
  status: async () => ({
    computeType: "int8",
    device: "cpu",
    limits: { maxDurationMs: 60_000 },
    modelCached: true,
    modelId: "small",
    modelLoaded: true,
    modelVersion: null,
    providerId: "fake-transcriber",
    queue: {
      active: 0,
      concurrency: 1,
      fairness: "fifo",
      maxQueued: 4,
      queued: 0,
    },
    ready: true,
    version: "transcription-provider-v1",
  }),
  transcribe: async () => ({
    durationMs: 1000,
    language: "en",
    segments: [
      {
        endMs: 1000,
        startMs: 0,
        text: "Hello",
        words: [{ endMs: 500, startMs: 0, word: "Hello" }],
      },
    ],
  }),
});

it("retains transcription previews across revision conflicts and deletes them after commit", async () => {
  let commits = 0;
  const headless = {
    call: (request: { operation: string }) => {
      if (request.operation === "resolve_asset_input") {
        return Promise.resolve({
          assetId: "asset-1",
          contentHash: { algorithm: "sha256", digest: "a".repeat(64) },
          path: "C:\\private\\asset.wav",
          probe: {
            audioChannels: 1,
            audioCodec: "pcm_s16le",
            audioSampleRateHz: 16_000,
            durationMs: 1000,
            formatName: "wav",
            hasAudio: true,
            hasVideo: false,
            videoCodec: null,
            videoHeight: null,
            videoWidth: null,
          },
          projectId: "project-1",
          revision: 2,
        });
      }
      commits += 1;
      if (commits === 1) {
        return Promise.reject(
          new BridgeError("REVISION_CONFLICT", "changed", true)
        );
      }
      return Promise.resolve({
        changedIds: ["caption-1"],
        projectId: "project-1",
        revision: 4,
        summary: "Committed transcription",
        warnings: [],
      });
    },
  } as unknown as HeadlessClient;
  const service = new TranscriptionApplicationService(
    provider(),
    headless,
    60_000,
    () => 100
  );
  const preview = await service.preview(
    { assetId: "asset-1", projectId: "project-1" },
    {
      markNonCancellable: () => undefined,
      onProgress: () => undefined,
      signal: new AbortController().signal,
    }
  );
  await expect(
    service.commitPreview({
      expectedRevision: 3,
      projectId: "project-1",
      token: preview.token,
    })
  ).rejects.toMatchObject({ code: "REVISION_CONFLICT" });
  await expect(
    service.commitPreview({
      expectedRevision: 3,
      projectId: "project-1",
      token: preview.token,
    })
  ).resolves.toMatchObject({ revision: 4 });
  expect(() => service.discardPreview(preview.token)).toThrowError(
    expect.objectContaining({ code: "TRANSCRIPTION_PREVIEW_NOT_FOUND" })
  );
  await service.close();
});
