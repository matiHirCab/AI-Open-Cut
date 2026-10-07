import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it, vi } from "vitest";
import ownership from "../../../contracts/contract-ownership-v1.json";
import headlessCatalog from "../../../contracts/headless-protocol-v1.json";
import contract from "../../../contracts/known-text-alignment-v1.json";
import mcp from "../../../contracts/mcp-surface-v1.json";
import providerCatalog from "../../../contracts/transcription-provider-v1.json";
import { loadBridgeConfig } from "../src/config";
import { BridgeError, type HeadlessClient } from "../src/headless";
import { NOOP_LOGGER } from "../src/logger";
import {
  knownTextAlignmentSupportSchema,
  speechAlignmentSchema,
  transcriptionStatusSchema,
} from "../src/schemas";
import {
  FasterWhisperTranscriber,
  type Transcriber,
  TranscriptionApplicationService,
} from "../src/transcription";
import ownershipPin from "./fixtures/known-text-ownership-predecessor-pin.json";
import rawPins from "./fixtures/known-text-predecessor-raw-pins.json";
import { projectKnownTextMcpPredecessor } from "./fixtures/known-text-projection";
import { orderedEffectDigest } from "./fixtures/ordered-effect-projection";

const queue = {
  active: 0,
  concurrency: 1 as const,
  fairness: "fifo" as const,
  maxQueued: 4,
  queued: 0,
};
const status = {
  computeType: "int8" as const,
  device: "cpu" as const,
  limits: { maxDurationMs: 60_000 },
  modelCached: true,
  modelId: "small",
  modelLoaded: true,
  modelVersion: null,
  providerId: "fake-aligner",
  queue,
  ready: true,
  version: "transcription-provider-v1",
};
const context = () => ({
  markNonCancellable: () => undefined,
  onProgress: () => undefined,
  signal: new AbortController().signal,
});
const source = {
  assetId: "asset-1",
  contentHash: null,
  path: "/private/allowed.wav",
  probe: { durationMs: 1000, hasAudio: true },
  projectId: "project-1",
  revision: 2,
};
const input = {
  assetId: "asset-1",
  knownText: "Hello world",
  projectId: "project-1",
};
const fixture = () => ({
  alignment: speechAlignmentSchema.parse(contract.alignment),
  durationMs: 1000,
  language: "en",
  segments: [
    {
      endMs: 900,
      startMs: 100,
      text: "Hello world",
      words: [
        { endMs: 400, startMs: 100, word: "Hello" },
        { endMs: 900, startMs: 500, word: "world" },
      ],
    },
  ],
});
const provider = (): Transcriber => ({
  align: async () => fixture(),
  close: async () => undefined,
  queueStatus: () => queue,
  status: async () => ({
    ...status,
    knownTextAlignment: contract.localSupport,
  }),
  transcribe: async () => ({ durationMs: 1000, language: "en", segments: [] }),
});
const headless = (
  call: (request: Record<string, unknown>) => Promise<unknown>
) => ({ call }) as unknown as HeadlessClient;

it("keeps legacy support false and rejects each canonical malformed explicit record", () => {
  expect(transcriptionStatusSchema.parse(status).knownTextAlignment).toEqual(
    contract.unsupported
  );
  expect(knownTextAlignmentSupportSchema.parse(contract.localSupport)).toEqual(
    contract.localSupport
  );
  for (const value of contract.invalidSupport) {
    expect(knownTextAlignmentSupportSchema.safeParse(value).success).toBe(
      false
    );
  }
});
it("preserves the exact predecessor and rejects unrelated/new-field drift", () => {
  expect(projectKnownTextMcpPredecessor(mcp)).toBeDefined();
  expect(() =>
    projectKnownTextMcpPredecessor({ ...mcp, unrelated: true })
  ).toThrow("Unrelated");
  const changed = structuredClone(mcp);
  changed.toolDefinitions.transcription_preview.inputSchema.properties.knownText.type =
    "number";
  expect(() => projectKnownTextMcpPredecessor(changed)).toThrow("Incorrect");
});
it("estimates without inference, snapshots aligned results and retries commit without realigning", async () => {
  const p = provider();
  const returned = fixture();
  const align = vi.fn(async () => returned);
  p.align = align;
  let commits = 0;
  const requests: Record<string, unknown>[] = [];
  const service = new TranscriptionApplicationService(
    p,
    headless((request) => {
      requests.push(request);
      if (request.operation === "validate_speech_alignment") {
        return Promise.resolve(source);
      }
      commits += 1;
      if (commits === 1) {
        throw new BridgeError("REVISION_CONFLICT", "changed", true);
      }
      return Promise.resolve({
        changedIds: ["caption"],
        projectId: "project-1",
        revision: 4,
        summary: "caption",
        warnings: [],
      });
    })
  );
  await service.estimate(input);
  expect(align).not.toHaveBeenCalled();
  const preview = await service.preview(input, context());
  expect(align).toHaveBeenCalledOnce();
  expect(JSON.stringify(preview)).not.toContain("/private/");
  expect(preview.alignment).toEqual(contract.alignment);
  for (const word of returned.alignment.words) {
    word.text = "mutated";
  }
  for (const segment of returned.segments) {
    segment.text = "mutated";
  }
  expect(preview.alignment?.words[0]?.text).toBe("Hello");
  const commit = {
    expectedRevision: 3,
    projectId: "project-1",
    token: preview.token,
  };
  await expect(service.commitPreview(commit)).rejects.toMatchObject({
    code: "REVISION_CONFLICT",
  });
  await expect(service.commitPreview(commit)).resolves.toMatchObject({
    revision: 4,
  });
  expect(align).toHaveBeenCalledOnce();
  expect(requests.at(-1)?.segments).toEqual(fixture().segments);
  expect(() => service.discardPreview(preview.token)).toThrow();
  await service.close();
});
it.each([
  "TRANSCRIPTION_UNAVAILABLE",
  "VALIDATION_FAILED",
  "ASSET_NOT_FOUND",
  "REVISION_CONFLICT",
])("does not publish on %s", async (code) => {
  const p = provider();
  const align = vi.fn(async () => fixture());
  p.align = align;
  if (code === "TRANSCRIPTION_UNAVAILABLE") {
    p.status = async () => status;
  }
  if (code === "VALIDATION_FAILED") {
    p.status = async () => ({
      ...status,
      knownTextAlignment: { ...contract.localSupport, maxDurationMs: 500 },
    });
  }
  let calls = 0;
  const service = new TranscriptionApplicationService(
    p,
    headless(() => {
      calls += 1;
      if (
        code === "ASSET_NOT_FOUND" ||
        (code === "REVISION_CONFLICT" && calls === 2)
      ) {
        throw new BridgeError(code, "source", code === "REVISION_CONFLICT");
      }
      return Promise.resolve(source);
    })
  );
  await expect(service.preview(input, context())).rejects.toMatchObject({
    code,
  });
  expect(align).toHaveBeenCalledTimes(code === "REVISION_CONFLICT" ? 1 : 0);
  await service.close();
});
it("rejects missing alignment, discards retained output and honors expiry", async () => {
  const p = provider();
  p.align = async () => ({ durationMs: 1000, language: "en", segments: [] });
  let now = 0;
  const service = new TranscriptionApplicationService(
    p,
    headless(async () => source),
    60_000,
    () => now
  );
  await expect(service.preview(input, context())).rejects.toMatchObject({
    code: "TRANSCRIPTION_INVALID_OUTPUT",
  });
  p.align = async () => fixture();
  const first = await service.preview(input, context());
  service.discardPreview(first.token);
  const second = await service.preview(input, context());
  now = 60_001;
  expect(() => service.discardPreview(second.token)).toThrow();
  await service.close();
});

it.each(["close", "cancel"])(
  "does not retain an inference after late %s",
  async (action) => {
    const reached = Promise.withResolvers<void>();
    const pending = Promise.withResolvers<typeof source>();
    let calls = 0;
    const service = new TranscriptionApplicationService(
      provider(),
      headless(() => {
        calls += 1;
        if (calls === 2) {
          reached.resolve();
          return pending.promise;
        }
        return Promise.resolve(source);
      })
    );
    const controller = new AbortController();
    const result = service.preview(input, {
      ...context(),
      signal: controller.signal,
    });
    await reached.promise;
    if (action === "close") {
      await service.close();
    } else {
      controller.abort();
    }
    pending.resolve(source);
    await expect(result).rejects.toMatchObject({
      code: action === "close" ? "TRANSCRIPTION_UNAVAILABLE" : "JOB_CANCELLED",
    });
    await service.close();
  }
);

const worker = (timeout = 5000) =>
  new FasterWhisperTranscriber(
    loadBridgeConfig({
      ...process.env,
      OPENCUT_TRANSCRIPTION_MAX_QUEUED: "1",
      OPENCUT_TRANSCRIPTION_PYTHON: process.env.OPENCUT_TEST_PYTHON ?? "python",
      OPENCUT_TRANSCRIPTION_TIMEOUT_MS: String(timeout),
      OPENCUT_TRANSCRIPTION_WORKER: resolve(
        import.meta.dirname,
        "fixtures/fake_transcription_worker.py"
      ),
    }),
    NOOP_LOGGER
  );

it("preserves independently captured prior operation/provider/ownership catalogs exactly", () => {
  for (const [name, hash] of Object.entries(rawPins)) {
    expect(
      createHash("sha256")
        .update(readFileSync(resolve(import.meta.dirname, "fixtures", name)))
        .digest("hex")
    ).toBe(hash);
  }
  const priorHeadless: unknown = JSON.parse(
    readFileSync(
      resolve(
        import.meta.dirname,
        "fixtures/known-text-headless-predecessor.raw"
      ),
      "utf8"
    )
  );
  const priorProvider: unknown = JSON.parse(
    readFileSync(
      resolve(
        import.meta.dirname,
        "fixtures/known-text-provider-predecessor.raw"
      ),
      "utf8"
    )
  );
  const protocol = structuredClone(headlessCatalog);
  expect(
    protocol.operations.filter((value) => value === contract.operation)
  ).toHaveLength(1);
  expect(
    protocol.status.editorCapabilities.filter(
      (value) => value === contract.capability
    )
  ).toHaveLength(1);
  expect(protocol.requests.validateSpeechAlignment).toEqual(contract.source);
  Reflect.deleteProperty(protocol.requests, "validateSpeechAlignment");
  protocol.operations = protocol.operations.filter(
    (value) => value !== contract.operation
  );
  protocol.status.editorCapabilities =
    protocol.status.editorCapabilities.filter(
      (value) => value !== contract.capability
    );
  expect(protocol).toEqual(priorHeadless);
  const providerProjection = structuredClone(providerCatalog);
  expect(providerProjection.requests.align.operation).toBe("align");
  expect(providerProjection.responses.align.alignment).toBe(
    "closed speech-alignment-v1 record with forced quality and actual producer/model identity"
  );
  expect(providerProjection.responses.status.knownTextAlignment).toBe(
    "optional closed support/limits record; omitted legacy means unsupported"
  );
  Reflect.deleteProperty(providerProjection.requests, "align");
  Reflect.deleteProperty(providerProjection.responses, "align");
  Reflect.deleteProperty(
    providerProjection.responses.status,
    "knownTextAlignment"
  );
  expect(providerProjection).toEqual(priorProvider);
  const owners = structuredClone(ownership);
  expect(owners.categories.knownTextAlignment.canonical).toBe(
    "contracts/known-text-alignment-v1.json"
  );
  Reflect.deleteProperty(owners.categories, "knownTextAlignment");
  expect(orderedEffectDigest(owners)).toBe(ownershipPin.semanticSha256);
});

it("rejects malformed worker output with the dedicated stable code", async () => {
  const p = worker();
  try {
    await p.status();
    await expect(
      p.align(
        "provider-only.wav",
        "__malformed_alignment__",
        undefined,
        1000,
        new AbortController().signal
      )
    ).rejects.toMatchObject({
      code: "TRANSCRIPTION_INVALID_OUTPUT",
      retryable: false,
    });
  } finally {
    await p.close();
  }
});

it("rejects malformed explicit support and missing advertised implementation", async () => {
  const p = provider();
  p.status = async () =>
    ({ ...status, knownTextAlignment: null }) as unknown as Awaited<
      ReturnType<Transcriber["status"]>
    >;
  const service = new TranscriptionApplicationService(
    p,
    headless(() => Promise.resolve(source))
  );
  await expect(service.status()).rejects.toMatchObject({
    code: "TRANSCRIPTION_INVALID_OUTPUT",
    retryable: false,
  });
  p.status = async () => ({
    ...status,
    knownTextAlignment: contract.localSupport,
  });
  Reflect.deleteProperty(p, "align");
  await expect(service.preview(input, context())).rejects.toMatchObject({
    code: "TRANSCRIPTION_UNAVAILABLE",
  });
  await service.close();
});

it("shares one bounded FIFO between alignment and ordinary transcription", async () => {
  const p = worker();
  try {
    await p.status();
    const first = p.align(
      "provider-only.wav",
      "__slow_alignment__",
      undefined,
      1000,
      new AbortController().signal
    );
    await Promise.resolve();
    const second = p.transcribe(
      "provider-only.wav",
      undefined,
      1000,
      new AbortController().signal
    );
    await expect(
      p.align(
        "provider-only.wav",
        "third",
        undefined,
        1000,
        new AbortController().signal
      )
    ).rejects.toMatchObject({
      code: "TRANSCRIPTION_QUEUE_FULL",
      retryable: true,
    });
    expect(p.queueStatus()).toMatchObject({ active: 1, queued: 1 });
    expect((await first).alignment?.quality).toBe("forced");
    expect((await second).segments[0]?.text).toBe("Packaged caption");
    expect(p.queueStatus()).toMatchObject({ active: 0, queued: 0 });
  } finally {
    await p.close();
  }
});

it.each(["timeout", "cancel", "pre-cancel"])(
  "preserves typed %s failure for known-text inference",
  async (action) => {
    const p = worker(500);
    try {
      await p.status();
      const controller = new AbortController();
      if (action === "pre-cancel") {
        controller.abort();
      }
      const result = p.align(
        "provider-only.wav",
        "__timeout_alignment__",
        undefined,
        1000,
        controller.signal
      );
      const assertion = expect(result).rejects.toMatchObject({
        code: action === "timeout" ? "TRANSCRIPTION_TIMEOUT" : "JOB_CANCELLED",
        retryable: true,
      });
      if (action === "cancel") {
        await Promise.resolve();
        controller.abort();
      }
      await assertion;
    } finally {
      await p.close();
    }
  }
);
