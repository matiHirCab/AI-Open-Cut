import { expect, it } from "vitest";
import type { z } from "zod/v4";
import contract from "../../../contracts/short-candidate-planning-cli-v1.json";
import { BridgeError } from "../src/headless";
import type { HeadlessRequest } from "../src/headless-contract";
import {
  planningFailure,
  readPlanningInput,
  runPlanningCli,
} from "../src/short-candidates/cli";
import {
  PLANNING_LIMITS,
  PlanningError,
  shortPlanningFailureSchema,
  shortPlanningInputSchema,
  shortPlanningSourceSchema,
  shortPlanningSuccessSchema,
} from "../src/short-candidates/contract";
import {
  parsePlanningInput,
  planShortCandidates,
} from "../src/short-candidates/planner";
import { ShortCandidatePlanner } from "../src/short-candidates/service";
import {
  assertShortPlanningConsumer,
  verifyShortPlanningContract,
} from "./short-candidates-contract";

const source = shortPlanningSourceSchema.parse(contract.source);
const original = () =>
  parsePlanningInput(structuredClone(contract.valid[0]?.input));
const expectKind = (run: () => unknown, kind: string) => {
  try {
    run();
    throw new Error("Expected planning rejection");
  } catch (error) {
    expect(error).toBeInstanceOf(PlanningError);
    expect((error as PlanningError).kind).toBe(kind);
  }
};

it("consumes independent canonical negation, uncertainty, cut and overlap oracles", () =>
  verifyShortPlanningContract());

it("preserves original text and chunk-relative word timing without source mutation", () => {
  const input = original();
  const second = input.chunks[1]?.segments[0];
  if (!second) {
    throw new Error("Missing fixture segment");
  }
  second.words = [{ endMs: 700, startMs: 100, word: "uncertain" }];
  const before = JSON.stringify(input);
  const result = planShortCandidates(input, source);
  expect(JSON.stringify(input)).toBe(before);
  expect(result.candidates[0]?.evidence[0]?.text).toBe(
    "I do not recommend that."
  );
  expect(result.candidates[0]?.evidence[0]).not.toHaveProperty("words");
  expect(result.candidates[1]?.evidence[0]?.words).toEqual([
    { endMs: 1700, startMs: 1100, word: "uncertain" },
  ]);
  second.text = "caller mutated its snapshot";
  expect(result.candidates[1]?.evidence[0]?.text).toBe(
    "The result is uncertain."
  );
});

it("retains the final shorter group and never silently truncates candidate capacity", () => {
  const input = original();
  input.selection = { endMs: 1500, startMs: 0 };
  const result = planShortCandidates(input, source);
  expect(result.candidates.map((candidate) => candidate.durationMs)).toEqual([
    1000, 500,
  ]);
  expect(result.candidates[1]?.warnings).toContain("duration_mismatch");
  input.maxCandidates = 1;
  expectKind(() => planShortCandidates(input, source), "limit");
});

it.each(
  [
    [{ candidateId: "candidate-3", endMs: 1000, startMs: 0 }],
    [{ candidateId: "candidate-1", startMs: 0 }],
    [{ candidateId: "candidate-1", endMs: 0, startMs: 0 }],
    [{ candidateId: "candidate-1", endMs: 2001, startMs: 0 }],
    [{ candidateId: "candidate-1" }, { candidateId: "candidate-1" }],
  ].map((edits) => ({ edits }))
)(
  "rejects invalid/duplicate boundary edits without replacing evidence: %j",
  ({ edits }) => {
    expectKind(
      () => planShortCandidates({ ...original(), edits }, source),
      "input"
    );
  }
);

it("rejects selection outside source, silence-only edits and extensions outside selected scope", () => {
  expectKind(
    () =>
      planShortCandidates(
        { ...original(), selection: { endMs: 2001, startMs: 0 } },
        source
      ),
    "input"
  );
  const input = original();
  const first = input.chunks[0]?.segments[0];
  if (!first) {
    throw new Error("Missing fixture segment");
  }
  first.endMs = 500;
  input.edits = [{ candidateId: "candidate-1", endMs: 900, startMs: 600 }];
  expectKind(() => planShortCandidates(input, source), "input");
  expectKind(
    () =>
      planShortCandidates(
        {
          ...original(),
          edits: [{ candidateId: "candidate-1", endMs: 1000, startMs: 0 }],
          selection: { endMs: 1750, startMs: 250 },
        },
        source
      ),
    "input"
  );
});

it("rejects chunk gaps, duplicates, incomplete coverage and ambiguous local timing", () => {
  for (const mutate of [
    (input: ReturnType<typeof original>) => {
      if (input.chunks[1]) {
        input.chunks[1].id = "c1";
      }
    },
    (input: ReturnType<typeof original>) => {
      input.chunks.pop();
    },
    (input: ReturnType<typeof original>) => {
      if (input.chunks[0]) {
        input.chunks[0].segments.push({
          endMs: 1000,
          id: "s1",
          startMs: 500,
          text: "duplicate",
        });
      }
    },
    (input: ReturnType<typeof original>) => {
      if (input.chunks[0]?.segments[0]) {
        input.chunks[0].segments[0].endMs = Number.MAX_SAFE_INTEGER;
      }
    },
  ]) {
    const input = original();
    mutate(input);
    expectKind(() => planShortCandidates(input, source), "input");
  }
});

it("enforces aggregate segment and word limits across otherwise valid chunks", () => {
  const input = original();
  for (let chunkIndex = 0; chunkIndex < input.chunks.length; chunkIndex += 1) {
    const chunk = input.chunks[chunkIndex];
    if (!chunk) {
      throw new Error("Missing chunk");
    }
    chunk.offsetMs = chunkIndex * 5000;
    chunk.durationMs = 5000;
    chunk.maxDurationMs = 5000;
    chunk.segments = Array.from(
      { length: chunkIndex === 0 ? 4096 : 1 },
      (_, index) => ({
        endMs: index + 1,
        id: `s${index}`,
        startMs: index,
        text: "x",
      })
    );
  }
  expectKind(
    () => planShortCandidates(input, { ...source, durationMs: 10_000 }),
    "limit"
  );
  const words = original();
  words.chunks = [0, 1].map((index) => ({
    durationMs: 20_000,
    id: `c${index}`,
    maxDurationMs: 20_000,
    modelId: "supplied",
    offsetMs: index * 20_000,
    providerId: "supplied",
    segments: [
      {
        endMs: 20_000,
        id: "s",
        startMs: 0,
        text: "x",
        words: Array.from({ length: 16_385 }, (_, i) => ({
          endMs: i + 1,
          startMs: i,
          word: "x",
        })),
      },
    ],
  }));
  expectKind(
    () => planShortCandidates(words, { ...source, durationMs: 40_000 }),
    "limit"
  );
});

const resolved = () => ({
  assetId: source.assetId,
  contentHash: { ...source.contentHash },
  path: "/private/managed-source.wav",
  probe: {
    audioChannels: 1,
    audioCodec: "pcm_s16le",
    audioSampleRateHz: 48_000,
    durationMs: 2000,
    formatName: "wav",
    hasAudio: true,
    hasVideo: false,
    videoCodec: null,
    videoHeight: null,
    videoWidth: null,
  },
  projectId: source.projectId,
  revision: source.revision,
});
const reader = (read: (index: number) => unknown) => {
  const calls: HeadlessRequest[] = [];
  const planner = new ShortCandidatePlanner({
    call: <T>(request: HeadlessRequest, schema: z.ZodType<T>) => {
      calls.push(request);
      return Promise.resolve(schema.parse(read(calls.length - 1)));
    },
  });
  return { calls, planner };
};

it("uses only existing core reads and never exposes managed paths or starts inference", async () => {
  const { calls, planner } = reader(() => resolved());
  const result = await planner.plan(original());
  expect(calls).toEqual(
    [0, 1].map(() => ({
      assetId: source.assetId,
      operation: "resolve_asset_input",
      projectId: source.projectId,
    }))
  );
  expect(JSON.stringify(result)).not.toContain("/private");
  expect(result.source.contentHash).toEqual(source.contentHash);
});

it.each(["revision", "digest", "duration"])(
  "rejects source %s changes before publication",
  async (change) => {
    const { planner } = reader((index) => {
      const value = resolved();
      if (index === 1) {
        if (change === "revision") {
          value.revision += 1;
        }
        if (change === "digest") {
          value.contentHash.digest = "b".repeat(64);
        }
        if (change === "duration") {
          value.probe.durationMs += 1;
        }
      }
      return value;
    });
    await expect(planner.plan(original())).rejects.toMatchObject({
      kind: "stale",
    });
  }
);

it("rejects source identity mismatches, missing hashes/audio and unapproved input before side effects", async () => {
  await Promise.all(
    [
      { ...resolved(), assetId: "other" },
      { ...resolved(), contentHash: null },
      { ...resolved(), probe: { ...resolved().probe, hasAudio: false } },
    ].map((value) =>
      expect(
        reader(() => value).planner.plan(original())
      ).rejects.toMatchObject({ kind: "source" })
    )
  );
  const { planner, calls } = reader(() => resolved());
  await expect(
    planner.plan({ ...original(), approvedTranscript: false })
  ).rejects.toMatchObject({ kind: "input" });
  expect(calls).toEqual([]);
});

const collect =
  (target: string[]) =>
  (value: string): Promise<void> => {
    target.push(value);
    return Promise.resolve();
  };

async function* bytes(value: string) {
  yield await Promise.resolve(Buffer.from(value));
}

it("runs the bounded CLI and reserves stdout for fully serialized successful plans", async () => {
  const output: string[] = [];
  const errors: string[] = [];
  const code = await runPlanningCli(
    bytes(JSON.stringify(original())),
    reader(() => resolved()).planner,
    collect(output),
    collect(errors)
  );
  expect(code).toBe(0);
  expect(output).toHaveLength(1);
  expect(JSON.parse(output[0] ?? "")).toEqual(contract.valid[0]?.expected);
  expect(errors).toEqual([]);
});

it("rejects malformed/invalid UTF8/oversized streamed input before planning", async () => {
  await expect(readPlanningInput(bytes("{"))).rejects.toMatchObject({
    kind: "input",
  });
  async function* invalidUtf8() {
    yield await Promise.resolve(new Uint8Array([255]));
  }
  await expect(readPlanningInput(invalidUtf8())).rejects.toMatchObject({
    kind: "input",
  });
  async function* excessive() {
    yield await Promise.resolve(Buffer.alloc(PLANNING_LIMITS.inputBytes));
    yield await Promise.resolve(Buffer.from("x"));
  }
  await expect(readPlanningInput(excessive())).rejects.toMatchObject({
    kind: "limit",
  });
});

it("reports core codes/retryability without diagnostic paths and contains failed output", async () => {
  expect(
    planningFailure(
      new BridgeError("ASSET_NOT_FOUND", "/secret/private/source.wav")
    )
  ).toEqual(contract.errors[1]);
  const outputs: string[] = [];
  const errors: string[] = [];
  const code = await runPlanningCli(
    bytes("{}"),
    {
      plan: () => Promise.reject(new BridgeError("ASSET_NOT_FOUND", "/secret")),
    },
    collect(outputs),
    collect(errors)
  );
  expect(code).toBe(1);
  expect(outputs).toEqual([]);
  expect(JSON.parse(errors[0] ?? "")).toEqual(contract.errors[1]);
  expect(
    await runPlanningCli(
      bytes(JSON.stringify(original())),
      reader(() => resolved()).planner,
      () => Promise.reject(new Error("EPIPE")),
      () => Promise.reject(new Error("stderr closed"))
    )
  ).toBe(1);
});

it.each(
  [
    [],
    ["chunks", 0],
    ["chunks", 0, "segments", 0],
    ["chunks", 0, "segments", 0, "words", 0],
    ["edits", 0],
    ["selection"],
  ].map((path) => ({ path }))
)(
  "rejects original JSON prototype keys at closed input path $path",
  ({ path }) => {
    const value = original();
    const segment = value.chunks[0]?.segments[0];
    if (!segment) {
      throw new Error("Missing segment");
    }
    segment.words = [{ endMs: 10, startMs: 0, word: "not" }];
    value.edits = [{ candidateId: "candidate-1" }];
    value.selection = { endMs: 2000, startMs: 0 };
    let target = value as unknown as Record<string | number, unknown>;
    for (const part of path) {
      target = target[part] as Record<string | number, unknown>;
    }
    Object.defineProperty(target, "__proto__", { enumerable: true, value: {} });
    const parsed: unknown = JSON.parse(JSON.stringify(value));
    expect(shortPlanningInputSchema.safeParse(parsed).success).toBe(false);
    expectKind(() => planShortCandidates(parsed, source), "input");
  }
);

it("keeps success/source/error JSON records closed before special keys can be dropped", () => {
  const withPrototypeKey = (value: unknown): unknown =>
    JSON.parse(`{"__proto__":{},${JSON.stringify(value).slice(1)}`);
  expect(
    shortPlanningSourceSchema.safeParse(withPrototypeKey(source)).success
  ).toBe(false);
  expect(
    shortPlanningSuccessSchema.safeParse(
      withPrototypeKey(contract.valid[0]?.expected)
    ).success
  ).toBe(false);
  expect(
    shortPlanningFailureSchema.safeParse(withPrototypeKey(contract.errors[0]))
      .success
  ).toBe(false);
});

it("rejects omitted, commented, skipped or masked contract consumer registrations", () => {
  const prefix =
    'import { verifyShortPlanningContract } from "./short-candidates-contract";';
  expect(() =>
    assertShortPlanningConsumer(
      `${prefix}\nit("required", verifyShortPlanningContract);`
    )
  ).not.toThrow();
  for (const body of [
    "",
    '// it("required", verifyShortPlanningContract);',
    'it.skip("required", verifyShortPlanningContract);',
    'it("required", () => true);',
  ]) {
    expect(() => assertShortPlanningConsumer(`${prefix}\n${body}`)).toThrow(
      "consumer is absent or masked"
    );
  }
});

it("keeps fixed planning messages even if an internal Error.message was modified", () => {
  const error = new PlanningError("input");
  error.message = "/private/supplied-text";
  expect(planningFailure(error)).toEqual(contract.errors[0]);
  expect(
    shortPlanningFailureSchema.safeParse({
      ...contract.errors[0],
      error: { kind: "input", message: "invented" },
    }).success
  ).toBe(false);
});

it("rejects an oversized successful payload before any stdout write", async () => {
  const oversized = planShortCandidates(original(), source);
  const quote = oversized.candidates[0]?.evidence[0];
  if (!quote) {
    throw new Error("Missing quote");
  }
  quote.text = "x".repeat(PLANNING_LIMITS.outputBytes + 1);
  const outputs: string[] = [];
  const errors: string[] = [];
  expect(
    await runPlanningCli(
      bytes(JSON.stringify(original())),
      { plan: () => Promise.resolve(oversized) },
      collect(outputs),
      collect(errors)
    )
  ).toBe(1);
  expect(outputs).toEqual([]);
  expect(JSON.parse(errors[0] ?? "").error.kind).toBe("limit");
});
