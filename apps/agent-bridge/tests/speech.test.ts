import { expect, it, vi } from "vitest";

import { BridgeError } from "../src/headless";
import type { HeadlessRequest } from "../src/headless-contract";
import { schemas } from "../src/schemas";
import {
  prepareSpeechSegments,
  SpeechApplicationService,
  type SpeechSynthesisRequest,
  type SpeechSynthesizer,
} from "../src/speech";

const status = {
  defaultLanguage: "es-UY",
  defaultSpeed: 1.1,
  defaultVoiceId: "voice-uy",
  device: "local-test",
  devices: ["local-test"],
  languages: ["es-UY", "en-US"],
  limits: { maxSpeed: 1.5, maxTextCharacters: 100, minSpeed: 0.75 },
  modelCached: true,
  modelId: "test/model",
  modelLoaded: true,
  models: [{ id: "test/model", sampleRateHz: 16_000, version: "7" }],
  modelVersion: "7",
  providerId: "test-provider",
  ready: true,
  resources: {
    execution: "local" as const,
    minimumLogicalCpus: 2,
    minimumRamBytes: 2_147_483_648,
    recommendedLogicalCpus: 4,
    recommendedRamBytes: 4_294_967_296,
  },
  sampleRateHz: 16_000,
  version: "1.0.0",
  voices: ["voice-uy", "voice-us"],
};

class FakeSpeechSynthesizer implements SpeechSynthesizer {
  cleaned: string[] = [];
  closed = false;
  cancelled = false;
  requests: SpeechSynthesisRequest[] = [];
  failure: BridgeError | undefined;
  cleanupFailure: Error | undefined;
  failedCleanupPaths: string[] = [];
  voiceUyAvailable = true;

  status() {
    return Promise.resolve(status);
  }

  listVoices() {
    return Promise.resolve([
      {
        accent: "Uruguayan Spanish",
        available: this.voiceUyAvailable,
        id: "voice-uy",
        isDefault: true,
        label: "Voice UY",
        language: "es-UY",
        locale: "es-UY",
        modelId: status.modelId,
        previewSupported: true,
        providerId: status.providerId,
      },
      {
        accent: "American English",
        available: true,
        id: "voice-us",
        isDefault: true,
        label: "Voice US",
        language: "en-US",
        locale: "en-US",
        modelId: status.modelId,
        previewSupported: true,
        providerId: status.providerId,
      },
    ]);
  }

  synthesize(synthesisRequest: SpeechSynthesisRequest) {
    this.requests.push(synthesisRequest);
    if (this.failure) {
      return Promise.reject(this.failure);
    }
    return Promise.resolve({
      durationMs: 250,
      modelId: status.modelId,
      modelVersion: status.modelVersion,
      outputPath: "generated.wav",
      providerId: status.providerId,
      request: synthesisRequest,
      sampleRateHz: status.sampleRateHz,
    });
  }

  cleanup(path: string) {
    this.cleaned.push(path);
    if (this.cleanupFailure) {
      this.failedCleanupPaths.push(path);
      return Promise.reject(this.cleanupFailure);
    }
    return Promise.resolve();
  }

  cancel() {
    this.cancelled = true;
  }

  queueStatus() {
    return {
      active: 0,
      concurrency: 1 as const,
      fairness: "fifo" as const,
      maxQueued: 8,
      queued: 0,
    };
  }

  close() {
    this.closed = true;
    const pending = this.failedCleanupPaths.splice(0);
    return Promise.all(
      pending.map(async (path) => await this.cleanup(path))
    ).then(() => undefined);
  }
}

const taskContext = () => ({
  markNonCancellable: () => undefined,
  onProgress: () => undefined,
  signal: new AbortController().signal,
});

const inputRequest = () =>
  schemas.ttsGenerateAndInsert.parse({
    expectedRevision: 4,
    projectId: "project",
    startMs: 500,
    text: "  Hola  ",
    trackId: "audio",
  });

it("resolves provider defaults and commits complete speech provenance", async () => {
  const provider = new FakeSpeechSynthesizer();
  let committed:
    | Extract<HeadlessRequest, { operation: "commit_generated_asset" }>
    | undefined;
  const service = new SpeechApplicationService(provider, (value) => {
    committed = value;
    return Promise.resolve({
      assetId: "asset",
      itemId: "item",
      projectId: "project",
      revision: 5,
      summary: "Generated and inserted speech",
      warnings: [],
    });
  });

  const result = await service.generateAndInsert(inputRequest(), taskContext());

  expect(provider.requests).toEqual([
    {
      language: "es-UY",
      speed: 1.1,
      text: "Hola",
      textOptions: {
        chunking: "sentence",
        normalization: "basic",
        pronunciations: [],
        sentencePauseMs: 120,
      },
      voiceId: "voice-uy",
    },
  ]);
  expect(committed).toMatchObject({
    expectedRevision: 4,
    operation: "commit_generated_asset",
    origin: {
      generation: {
        modelId: "test/model",
        modelVersion: "7",
        providerId: "test-provider",
        request: provider.requests[0],
        sampleRateHz: 16_000,
      },
      type: "speech_synthesis",
    },
    projectId: "project",
    trackId: "audio",
  });
  expect(result).toMatchObject({
    assetId: "asset",
    itemId: "item",
    language: "es-UY",
    modelId: "test/model",
    providerId: "test-provider",
    revision: 5,
    voice: "voice-uy",
    warnings: [],
  });
  expect(provider.cleaned).toEqual(["generated.wav"]);
});

it("normalizes text, applies ordered pronunciations, and chunks sentences", () => {
  expect(
    prepareSpeechSegments({
      language: "en-US",
      speed: 1,
      text: "  Cafe\u0301\nAI. OpenCut works!  ",
      textOptions: {
        chunking: "sentence",
        normalization: "basic",
        pronunciations: [
          { spoken: "Artificial Intelligence", term: "AI" },
          { spoken: "Open Cut", term: "OpenCut" },
        ],
        sentencePauseMs: 300,
      },
      voiceId: "voice-us",
    })
  ).toEqual(["Café Artificial Intelligence.", "Open Cut works!"]);
});

it("lists enriched voices and estimates local zero-cost resources", async () => {
  const provider = new FakeSpeechSynthesizer();
  const service = new SpeechApplicationService(provider, () =>
    Promise.reject(new Error("commit should not run"))
  );
  await expect(service.listVoices("en-US")).resolves.toEqual([
    expect.objectContaining({
      accent: "American English",
      available: true,
      id: "voice-us",
      label: "Voice US",
      previewSupported: true,
    }),
  ]);
  await expect(
    service.estimate({ text: "One. Two.", type: "request" })
  ).resolves.toMatchObject({
    chunks: 2,
    cost: { amount: 0, billing: "local", currency: null },
    modelCached: true,
    modelLoaded: true,
    resources: {
      minimumLogicalCpus: 2,
      recommendedLogicalCpus: 4,
    },
  });
});

it("reports unavailable voices but refuses to queue them", async () => {
  const provider = new FakeSpeechSynthesizer();
  provider.voiceUyAvailable = false;
  const service = new SpeechApplicationService(provider, () =>
    Promise.reject(new Error("commit should not run"))
  );
  await expect(service.listVoices("es-UY")).resolves.toEqual([
    expect.objectContaining({ available: false, id: "voice-uy" }),
  ]);
  await expect(
    service.estimate({ text: "No disponible", type: "request" })
  ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
});

it("previews, retries a conflicting commit without synthesis, and discards", async () => {
  const provider = new FakeSpeechSynthesizer();
  let attempts = 0;
  const service = new SpeechApplicationService(provider, () => {
    attempts += 1;
    if (attempts === 1) {
      return Promise.reject(new BridgeError("REVISION_CONFLICT", "refresh"));
    }
    return Promise.resolve({
      assetId: "preview-asset",
      itemId: "preview-item",
      projectId: "project",
      revision: 8,
      summary: "done",
      warnings: [],
    });
  });
  const preview = await service.preview(
    { text: "Preview me.", type: "request" },
    taskContext()
  );
  expect(service.previewAudio(preview.token)).toEqual({
    mimeType: "audio/wav",
    path: "generated.wav",
  });
  await expect(
    service.commitPreview(preview.token, "project", 6, {
      startMs: 0,
      trackId: "audio",
      type: "insert",
    })
  ).rejects.toMatchObject({ code: "REVISION_CONFLICT" });
  await expect(
    service.commitGeneratedArtifact(preview.token, 7)
  ).resolves.toMatchObject({ assetId: "preview-asset", revision: 8 });
  expect(provider.requests).toHaveLength(1);

  const discarded = await service.preview(
    { text: "Discard me.", type: "request" },
    taskContext()
  );
  await expect(service.discardPreview(discarded.token)).resolves.toEqual({
    discarded: true,
    token: discarded.token,
  });
  expect(provider.cleaned).toEqual(["generated.wav", "generated.wav"]);
});

it("expires preview tokens and schedules owned output cleanup", async () => {
  const provider = new FakeSpeechSynthesizer();
  let now = 1000;
  const service = new SpeechApplicationService(
    provider,
    () => Promise.reject(new Error("commit should not run")),
    100,
    () => now
  );
  const preview = await service.preview(
    { text: "Expiring preview.", type: "request" },
    taskContext()
  );
  now = 1101;
  expect(() => service.previewAudio(preview.token)).toThrowError(
    expect.objectContaining({ code: "GENERATED_ARTIFACT_NOT_FOUND" })
  );
  await vi.waitFor(() => expect(provider.cleaned).toEqual(["generated.wav"]));
});

it("keeps a committed edit successful and warns when cleanup fails", async () => {
  const provider = new FakeSpeechSynthesizer();
  provider.cleanupFailure = new Error("locked");
  const service = new SpeechApplicationService(provider, () =>
    Promise.resolve({
      assetId: "asset",
      itemId: "item",
      projectId: "project",
      revision: 5,
      summary: "done",
      warnings: [],
    })
  );

  await expect(
    service.generateAndInsert(inputRequest(), taskContext())
  ).resolves.toMatchObject({
    revision: 5,
    warnings: ["TEMP_FILE_CLEANUP_FAILED"],
  });
  provider.cleanupFailure = undefined;
  await service.close();
  expect(provider.cleaned).toEqual(["generated.wav", "generated.wav"]);
});

it("does not let cleanup failure mask a commit error", async () => {
  const provider = new FakeSpeechSynthesizer();
  provider.cleanupFailure = new Error("locked");
  const service = new SpeechApplicationService(provider, () =>
    Promise.reject(new BridgeError("TRACK_NOT_FOUND", "missing track"))
  );

  await expect(
    service.generateAndInsert(inputRequest(), taskContext())
  ).rejects.toMatchObject({ code: "TRACK_NOT_FOUND" });
});

it("supports another language without changing orchestration", async () => {
  const provider = new FakeSpeechSynthesizer();
  const service = new SpeechApplicationService(provider, () =>
    Promise.resolve({
      assetId: "asset",
      itemId: "item",
      projectId: "project",
      revision: 5,
      summary: "done",
      warnings: [],
    })
  );
  const input = inputRequest();
  const result = await service.generateAndInsert(
    { ...input, language: "en-US" },
    taskContext()
  );
  expect(result.voice).toBe("voice-us");
  expect(provider.requests[0]?.language).toBe("en-US");
});

it("preserves provider failures without attempting a commit", async () => {
  const provider = new FakeSpeechSynthesizer();
  provider.failure = new BridgeError(
    "TTS_SYNTHESIS_FAILED",
    "provider failed",
    true
  );
  let commitCalled = false;
  const service = new SpeechApplicationService(provider, () => {
    commitCalled = true;
    throw new Error("must not commit");
  });

  await expect(
    service.generateAndInsert(inputRequest(), taskContext())
  ).rejects.toMatchObject({ code: "TTS_SYNTHESIS_FAILED", retryable: true });
  expect(commitCalled).toBe(false);
});

it("retains generated output when the core reports a revision conflict", async () => {
  const provider = new FakeSpeechSynthesizer();
  const service = new SpeechApplicationService(provider, () =>
    Promise.reject(new BridgeError("REVISION_CONFLICT", "conflict", true))
  );
  const failure = service.generateAndInsert(inputRequest(), taskContext());
  await expect(failure).rejects.toMatchObject({
    code: "REVISION_CONFLICT",
    generatedArtifact: {
      expiresAtMs: expect.any(Number),
      token: expect.any(String),
    },
  });
  expect(provider.cleaned).toEqual([]);
  await service.close();
  expect(provider.cleaned).toEqual(["generated.wav"]);
});

it("recommits a retained artifact without rerunning synthesis", async () => {
  const provider = new FakeSpeechSynthesizer();
  let attempts = 0;
  const service = new SpeechApplicationService(provider, () => {
    attempts += 1;
    if (attempts === 1) {
      return Promise.reject(
        new BridgeError("REVISION_CONFLICT", "refresh revision", true)
      );
    }
    return Promise.resolve({
      assetId: "asset",
      itemId: "item",
      projectId: "project",
      revision: 6,
      summary: "done",
      warnings: [],
    });
  });
  let token = "";
  try {
    await service.generateAndInsert(inputRequest(), taskContext());
  } catch (error) {
    const {
      generatedArtifact: { token: artifactToken },
    } = error as {
      generatedArtifact: { token: string };
    };
    token = artifactToken;
  }
  const result = await service.commitGeneratedArtifact(token, 5);
  expect(result).toMatchObject({ assetId: "asset", revision: 6 });
  expect(provider.requests).toHaveLength(1);
  expect(provider.cleaned).toEqual(["generated.wav"]);
  await expect(service.commitGeneratedArtifact(token, 6)).rejects.toMatchObject(
    {
      code: "GENERATED_ARTIFACT_NOT_FOUND",
    }
  );
});
