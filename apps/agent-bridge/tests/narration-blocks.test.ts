import { expect, it, vi } from "vitest";
import type { z } from "zod/v4";
import contract from "../../../contracts/narration-blocks-v1.json";
import speechFixture from "../../../contracts/speech-provider-v1.json";
import { BridgeError } from "../src/headless";
import { reportedCoreError } from "../src/headless-events";
import type { JobTaskContext } from "../src/jobs";
import { assertNarrationJsonBudget } from "../src/narration-blocks/budget";
import { NarrationPublicationUnknownError } from "../src/narration-blocks/errors";
import {
  narrationPlanInputSchema,
  resolvedNarrationRequestSchema,
  speechVoiceListSchema,
  ttsStatusSchema,
} from "../src/schemas";
import {
  prepareSpeechSegments,
  SpeechApplicationService,
  type SpeechSynthesisRequest,
  type SpeechSynthesizer,
} from "../src/speech";

const context = (): JobTaskContext => ({
  markNonCancellable: vi.fn(),
  onProgress: vi.fn(),
  signal: new AbortController().signal,
});
const request = (): z.infer<typeof resolvedNarrationRequestSchema> =>
  resolvedNarrationRequestSchema.parse({
    ...speechFixture.origin.generation.request,
    narrationProfile: {
      id: "11111111-1111-4111-8111-111111111111",
      name: "Pinned profile",
      version: 1,
    },
  });
function setup(now = () => 1000) {
  const requests: SpeechSynthesisRequest[] = [];
  const cleaned: string[] = [];
  const cleanup = vi.fn(async (path: string) => {
    await Promise.resolve(); // Model the asynchronous provider/transport boundary.

    cleaned.push(path);
  });
  const closed = vi.fn(async () => {
    await Promise.resolve(); // Model the asynchronous provider/transport boundary.
  });
  const provider: SpeechSynthesizer = {
    cancel: vi.fn(),
    cleanup,
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
    synthesize: async (input) => {
      await Promise.resolve(); // Model the asynchronous provider/transport boundary.

      requests.push(structuredClone(input));
      return {
        durationMs: 400,
        modelId: speechFixture.status.modelId,
        modelVersion: speechFixture.status.modelVersion,
        outputPath: `owned-${requests.length}.wav`,
        providerId: speechFixture.status.providerId,
        request: structuredClone(input),
        sampleRateHz: speechFixture.status.sampleRateHz,
      };
    },
  };
  const speech = new SpeechApplicationService(
    provider,
    async () => {
      await Promise.resolve();
      return {
        assetId: "asset",
        itemId: "item",
        projectId: "project",
        revision: 1,
        summary: "inserted",
        warnings: [],
      };
    },
    60_000,
    now
  );
  return { cleaned, cleanup, closed, provider, requests, speech };
}

it("consumes independently authored ordered pronunciation oracles without changing the resolver", () => {
  for (const entry of contract.spokenCases) {
    expect(
      prepareSpeechSegments({
        ...request(),
        text: entry.text,
        textOptions: {
          chunking: "none",
          normalization: "none",
          pronunciations: entry.pronunciations,
          sentencePauseMs: 120,
        },
      })
    ).toEqual(entry.segments);
  }
  expect(
    narrationPlanInputSchema.safeParse({
      boundaries: null,
      expectedVersion: 1,
      maxTextCharacters: 4096,
      profileId: request().narrationProfile.id,
      sourceText: "A.",
    }).success
  ).toBe(false);
});

it("preflights resolved expansion before any inference and retains the pinned attribution", async () => {
  const f = setup();
  try {
    const expanded = request();
    expanded.text = "AI.";
    expanded.textOptions = {
      chunking: "none",
      normalization: "none",
      pronunciations: [{ spoken: "x".repeat(256), term: "AI" }],
      sentencePauseMs: 120,
    };
    await expect(
      f.speech.prepareNarrationRequest(expanded, 10)
    ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
    expect(f.requests).toHaveLength(0);
    const preview = await f.speech.previewResolvedNarration(
      request(),
      4096,
      context()
    );
    const result = await f.speech.withNarrationPreviews(
      [preview.token],
      async ([artifact]) => {
        await Promise.resolve(); // Model the asynchronous provider/transport boundary.

        expect(artifact?.generated.request.narrationProfile).toEqual(
          request().narrationProfile
        );
        return true;
      }
    );
    expect(result.value).toBe(true);
    expect(f.requests).toHaveLength(1);
    expect(f.cleaned).toEqual(["owned-1.wav"]);
  } finally {
    await f.speech.close();
  }
});

it("rejects duplicate reservations before disturbing valid previews", async () => {
  const f = setup();
  try {
    const p = await f.speech.previewResolvedNarration(
      request(),
      4096,
      context()
    );
    await expect(
      f.speech.withNarrationPreviews([p.token, p.token], async () => {
        await Promise.resolve();
        return true;
      })
    ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
    expect(f.speech.previewAudio(p.token).path).toBe("owned-1.wav");
    expect(f.cleaned).toEqual([]);
  } finally {
    await f.speech.close();
  }
});

it("holds complete preview ownership across expiry and concurrent ordinary consumers", async () => {
  let now = 1000;
  const f = setup(() => now);
  const gate = Promise.withResolvers<void>();
  try {
    const a = await f.speech.previewResolvedNarration(
      request(),
      4096,
      context()
    );
    const b = await f.speech.previewResolvedNarration(
      request(),
      4096,
      context()
    );
    const work = f.speech.withNarrationPreviews(
      [a.token, b.token],
      async (_artifacts, begin) => {
        begin();
        await gate.promise;
        return "ack";
      }
    );
    now += 60_001;
    await f.speech.status();
    expect(f.cleaned).toEqual([]);
    await expect(f.speech.discardPreview(a.token)).rejects.toMatchObject({
      code: "VALIDATION_FAILED",
    });
    await expect(
      f.speech.commitGeneratedArtifact(a.token, 0)
    ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
    await expect(
      f.speech.withNarrationPreviews([b.token], async () => {
        await Promise.resolve();
        return true;
      })
    ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
    gate.resolve();
    expect((await work).value).toBe("ack");
    expect(f.cleaned).toEqual(["owned-1.wav", "owned-2.wav"]);
  } finally {
    gate.resolve();
    await f.speech.close();
  }
});

it("restores original retention after a typed conflict without synthesis replay", async () => {
  const f = setup();
  try {
    const p = await f.speech.previewResolvedNarration(
      request(),
      4096,
      context()
    );
    await expect(
      f.speech.withNarrationPreviews([p.token], async (_a, begin) => {
        await Promise.resolve(); // Model the asynchronous provider/transport boundary.

        begin();
        throw reportedCoreError({
          code: "REVISION_CONFLICT",
          failedStage: null,
          ffmpegExitCode: null,
          ffmpegStderrExcerpt: null,
          message: "stale",
          retryable: true,
        });
      })
    ).rejects.toMatchObject({ code: "REVISION_CONFLICT" });
    expect(f.speech.previewAudio(p.token).path).toBe("owned-1.wav");
    await f.speech.withNarrationPreviews([p.token], async () => {
      await Promise.resolve();
      return "explicit retry ack";
    });
    expect(f.requests).toHaveLength(1);
    expect(f.cleaned).toHaveLength(1);
  } finally {
    await f.speech.close();
  }
});

it("quarantines lost acknowledgements against all consumers until explicit reconciliation", async () => {
  let now = 1000;
  const f = setup(() => now);
  try {
    const p = await f.speech.previewResolvedNarration(
      request(),
      4096,
      context()
    );
    await expect(
      f.speech.withNarrationPreviews([p.token], async (_a, begin) => {
        await Promise.resolve(); // Model the asynchronous provider/transport boundary.

        begin();
        throw new Error("lost acknowledgement");
      })
    ).rejects.toBeInstanceOf(NarrationPublicationUnknownError);
    now += 60_001;
    await f.speech.status();
    expect(f.cleaned).toEqual([]);
    await expect(f.speech.discardPreview(p.token)).rejects.toMatchObject({
      code: "VALIDATION_FAILED",
    });
    await expect(
      f.speech.commitGeneratedArtifact(p.token, 0)
    ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
    await expect(
      f.speech.withNarrationPreviews([p.token], async () => {
        await Promise.resolve();
        return true;
      })
    ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
    await f.speech.acknowledgeNarrationPreviews([p.token]);
    expect(f.cleaned).toEqual(["owned-1.wav"]);
  } finally {
    await f.speech.close();
  }
});

it("treats cancellation before publication as nonpublication and preserves the preview", async () => {
  const f = setup();
  try {
    const p = await f.speech.previewResolvedNarration(
      request(),
      4096,
      context()
    );
    await expect(
      f.speech.withNarrationPreviews([p.token], async () => {
        await Promise.resolve(); // Model the asynchronous provider/transport boundary.

        throw new BridgeError("JOB_CANCELLED", "before publication");
      })
    ).rejects.toMatchObject({ code: "JOB_CANCELLED" });
    expect(f.speech.previewAudio(p.token).path).toBe("owned-1.wav");
  } finally {
    await f.speech.close();
  }
});

it("keeps acknowledged cleanup debt non-reusable and reports a warning", async () => {
  const f = setup();
  try {
    const p = await f.speech.previewResolvedNarration(
      request(),
      4096,
      context()
    );
    f.cleanup.mockRejectedValueOnce(new Error("cleanup denied"));
    const result = await f.speech.withNarrationPreviews(
      [p.token],
      async (_a, begin) => {
        await Promise.resolve(); // Model the asynchronous provider/transport boundary.

        begin();
        return "authoritative ack";
      }
    );
    expect(result.value).toBe("authoritative ack");
    expect(result.warnings).toHaveLength(1);
    await expect(
      f.speech.commitGeneratedArtifact(p.token, 0)
    ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
    expect(() => f.speech.previewAudio(p.token)).toThrow();
  } finally {
    await f.speech.close();
  }
});

it("close awaits active core publication before cleaning owned files and closing the provider", async () => {
  const f = setup();
  const gate = Promise.withResolvers<void>();
  const p = await f.speech.previewResolvedNarration(request(), 4096, context());
  const work = f.speech.withNarrationPreviews([p.token], async (_a, begin) => {
    begin();
    await gate.promise;
    return true;
  });
  const closing = f.speech.close();
  await Promise.resolve();
  expect(f.closed).not.toHaveBeenCalled();
  expect(f.cleaned).toEqual([]);
  gate.resolve();
  await work;
  await closing;
  expect(f.closed).toHaveBeenCalledOnce();
  expect(f.cleaned).toEqual(["owned-1.wav"]);
});

it("distinguishes confirmed core prejournal INTERNAL_ERROR from the same transport code", async () => {
  const f = setup();
  try {
    const p = await f.speech.previewResolvedNarration(
      request(),
      4096,
      context()
    );
    await expect(
      f.speech.withNarrationPreviews([p.token], async (_a, begin) => {
        await Promise.resolve(); // Model the asynchronous provider/transport boundary.

        begin();
        throw reportedCoreError({
          code: "INTERNAL_ERROR",
          failedStage: null,
          ffmpegExitCode: null,
          ffmpegStderrExcerpt: null,
          message: "prejournal persistence failed",
          retryable: false,
        });
      })
    ).rejects.toMatchObject({ code: "INTERNAL_ERROR" });
    expect(f.speech.previewAudio(p.token).path).toBe("owned-1.wav");
    const spoof = Object.assign(
      new BridgeError("INTERNAL_ERROR", "lost protocol acknowledgement"),
      { confirmedCoreError: true }
    );
    await expect(
      f.speech.withNarrationPreviews([p.token], async (_a, begin) => {
        await Promise.resolve(); // Model the asynchronous provider/transport boundary.

        begin();
        throw spoof;
      })
    ).rejects.toBeInstanceOf(NarrationPublicationUnknownError);
    expect(() => f.speech.previewAudio(p.token)).toThrow();
    expect(f.cleaned).toEqual([]);
  } finally {
    await f.speech.close();
  }
});

it("counts exact JSON UTF8 boundaries independently including escaping and surrogate pairs", () => {
  for (const value of [
    null,
    true,
    17,
    -0,
    1e30,
    'AI😀ñ\n\u0000"\\',
    { a: "é", b: ["😀", "\ud800", null], omitted: undefined },
    [1, false, "x"],
  ]) {
    const expected = Buffer.byteLength(JSON.stringify(value), "utf8");
    expect(assertNarrationJsonBudget(value, expected)).toBe(expected);
    expect(() => assertNarrationJsonBudget(value, expected - 1)).toThrow();
  }
  const cycle: Record<string, unknown> = {};
  cycle.self = cycle;
  expect(() => assertNarrationJsonBudget(cycle, 100)).toThrow();
});

it.each([
  "mcp-surface-v1.json",
  "headless-protocol-v1.json",
  "contract-ownership-v1.json",
] as const)(
  "preserves independently frozen raw and compact predecessor bytes: %s",
  async (name) => {
    const { readFile } = await import("node:fs/promises");
    const { createHash } = await import("node:crypto");
    const { restoreNarrationBlocksRaw } = await import(
      "./fixtures/durable-main47-compat"
    );
    const { default: pins } = await import(
      "./fixtures/narration-blocks-predecessor-pins.json"
    );
    const { default: compactPins } = await import(
      "./fixtures/narration-blocks-predecessor-compact-pins.json"
    );
    const raw = await readFile(
      new URL(`../../../contracts/${name}`, import.meta.url),
      "utf8"
    );
    const hash = (value: string) =>
      createHash("sha256").update(value).digest("hex");
    expect(hash(restoreNarrationBlocksRaw(raw))).toBe(pins[name]);
    expect(
      hash(restoreNarrationBlocksRaw(JSON.stringify(JSON.parse(raw))))
    ).toBe(compactPins[name]);
    const drift = JSON.parse(raw);
    drift.unrelated = "preserve me";
    const restored = restoreNarrationBlocksRaw(JSON.stringify(drift));
    expect(JSON.parse(restored).unrelated).toBe("preserve me");
    expect(hash(restored)).not.toBe(compactPins[name]);
  }
);

it("does not invert changed narration schema additions or duplicate operation declarations", async () => {
  const { readFile } = await import("node:fs/promises");
  const { removeNarrationBlockAdditions } = await import(
    "./fixtures/durable-main47-compat"
  );
  const mcp = JSON.parse(
    await readFile(
      new URL("../../../contracts/mcp-surface-v1.json", import.meta.url),
      "utf8"
    )
  );
  mcp.$defs.ProjectGetStateOutputPropertiesProjectProperties.narrationManifests.maxItems = 33;
  expect(() => removeNarrationBlockAdditions(mcp, "mcp-surface-v1")).toThrow();
  const headless = JSON.parse(
    await readFile(
      new URL("../../../contracts/headless-protocol-v1.json", import.meta.url),
      "utf8"
    )
  );
  headless.operations.push("narration_blocks_plan");
  expect(() =>
    removeNarrationBlockAdditions(headless, "headless-protocol-v1")
  ).toThrow();
});

it("consumes independently authored positive wire DTOs and rejects each single-field closed mutation", async () => {
  const schemas = await import("../src/schemas");
  const transport = await import("../src/narration-blocks/transport");
  const targets: Record<string, z.ZodType> = {
    commitInput: transport.commitNarrationBlocksInputSchema,
    manifest: schemas.narrationManifestSchema,
    plan: schemas.narrationPlanSchema,
    planInput: schemas.narrationPlanInputSchema,
    preparedBlock: schemas.narrationPreparedBlockSchema,
  };
  for (const [target, value] of Object.entries(contract.wire)) {
    expect(targets[target]?.parse(value)).toEqual(value);
  }
  for (const example of contract.wireInvalidCases) {
    const value: unknown = structuredClone(
      contract.wire[example.target as keyof typeof contract.wire]
    );
    let parent = value as Record<string, unknown>;
    for (const key of example.path.slice(0, -1)) {
      parent = parent[key] as Record<string, unknown>;
    }
    const field = example.path.at(-1);
    if (typeof field !== "string") {
      throw new Error("Malformed independent fixture mutation");
    }
    parent[field] = example.value;
    const result = targets[example.target]?.safeParse(value);
    expect(result?.success).toBe(false);
    if (result && !result.success) {
      expect(
        result.error.issues.some(
          (issue) =>
            issue.path.at(-1) === field ||
            (issue.code === "unrecognized_keys" && issue.keys.includes(field))
        )
      ).toBe(true);
    }
  }
  expect(
    transport.narrationRequestSchema.parse({
      input: contract.wire.planInput,
      operation: "narration_blocks_plan",
    })
  ).toHaveProperty("operation", "narration_blocks_plan");
  expect(
    transport.narrationRequestSchema.parse({
      input: contract.wire.commitInput,
      operation: "commit_narration_blocks",
    })
  ).toHaveProperty("operation", "commit_narration_blocks");
  expect(
    transport.narrationRequestSchema.parse({
      blockId: contract.wire.plan.blocks[0]?.id,
      expectedRevision: 0,
      manifestId: contract.wire.plan.id,
      operation: "detach_narration_block",
      projectId: "fixture-project",
    })
  ).toHaveProperty("operation", "detach_narration_block");
});
