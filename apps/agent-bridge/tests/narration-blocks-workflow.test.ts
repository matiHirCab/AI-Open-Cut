import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it, vi } from "vitest";
import type { z } from "zod/v4";
import narrationFixture from "../../../contracts/narration-blocks-v1.json";
import speechFixture from "../../../contracts/speech-provider-v1.json";
import { BridgeError, type HeadlessClient } from "../src/headless";
import type { HeadlessRequest } from "../src/headless-contract";
import type { JobTaskContext } from "../src/jobs";
import { runNarrationCli } from "../src/narration-blocks/cli";
import { NarrationPublicationUnknownError } from "../src/narration-blocks/errors";
import { NarrationBlocksWorkflow } from "../src/narration-blocks/workflow";
import {
  type narrationManifestSchema,
  narrationPlanSchema,
  resolvedNarrationRequestSchema,
} from "../src/schemas";
import { SpeechApplicationService } from "../src/speech";
import { createNarrationTestProvider } from "./fixtures/narration-blocks-provider";

const context = (): JobTaskContext => ({
  markNonCancellable: vi.fn(),
  onProgress: vi.fn(),
  signal: new AbortController().signal,
});
const plan = () =>
  narrationPlanSchema.parse({
    blocks: [
      {
        displayText: "A. ",
        endScalar: 3,
        id: "11111111-1111-4111-8111-111111111111",
        paragraph: 0,
        startScalar: 0,
      },
      {
        displayText: "B.",
        endScalar: 5,
        id: "11111111-1111-4111-8111-111111111112",
        paragraph: 0,
        startScalar: 3,
      },
    ].map((block) => ({
      ...block,
      request: resolvedNarrationRequestSchema.parse({
        ...speechFixture.origin.generation.request,
        narrationProfile: {
          id: "11111111-1111-4111-8111-111111111119",
          name: "Pinned",
          version: 1,
        },
        text: block.displayText.trim(),
      }),
    })),
    id: "11111111-1111-4111-8111-111111111110",
    maxTextCharacters: 4096,
    sourceText: "A. B.",
  });

async function setup() {
  const directory = await mkdtemp(join(tmpdir(), "narration-workflow-unit-"));
  const f = createNarrationTestProvider(directory);
  let manifest: z.infer<typeof narrationManifestSchema> | undefined;
  let assets: Array<{
    id: string;
    contentHash: { algorithm: "sha256"; digest: string };
    origin: unknown;
  }> = [];
  const control: {
    lost?: boolean;
    gate?: Promise<void>;
    beforeCommit?: () => void;
  } = {};
  let commits = 0;
  const planningBudgets: number[] = [];
  const headless: Pick<HeadlessClient, "call"> = {
    call: async <T>(request: HeadlessRequest, _schema: z.ZodType<T>) => {
      if (request.operation === "narration_blocks_plan") {
        const budget = request.input.maxTextCharacters;
        planningBudgets.push(budget);
        if (budget < 1 || budget > 4096) {
          throw new BridgeError(
            "VALIDATION_FAILED",
            "Invalid native planning budget"
          );
        }
        return { ...plan(), maxTextCharacters: budget } as T;
      }
      if (request.operation === "get_state") {
        return {
          project: {
            assets,
            id: "project",
            narrationManifests: manifest ? [manifest] : [],
            revision: 1,
          },
        } as T;
      }
      if (request.operation !== "commit_narration_blocks") {
        throw new Error("unexpected mock transport operation");
      }
      commits += 1;
      control.beforeCommit?.();
      await control.gate;
      const { input } = request;
      let start = input.startMs;
      assets = [];
      const blocks: z.infer<typeof narrationManifestSchema>["blocks"] = [];
      for (const [index, generated] of input.blocks.entries()) {
        const assetId = `asset-${index}`;
        // biome-ignore lint/performance/noAwaitInLoops: Preserve the fake transport request order.
        const bytes = await readFile(generated.path);
        assets.push({
          contentHash: {
            algorithm: "sha256",
            digest: createHash("sha256").update(bytes).digest("hex"),
          },
          id: assetId,
          origin: structuredClone(generated.origin),
        });
        blocks.push({
          ...generated.prepared.block,
          binding: {
            assetId,
            captionRefs: [],
            cueRefs: [],
            durationMs: generated.durationMs,
            itemId: `item-${index}`,
            sourceInMs: 0,
            startMs: start,
            trackId: input.trackId,
          },
          spokenSegments: generated.prepared.spokenSegments,
          spokenText: generated.prepared.spokenText,
        });
        start += generated.durationMs;
      }
      manifest = {
        blocks,
        id: input.plan.id,
        markerPolicy: input.markerPolicy ?? { type: "none" },
        maxTextCharacters: input.plan.maxTextCharacters,
        sourceText: input.plan.sourceText,
        ...(input.caption ? { caption: input.caption } : {}),
      };
      if (control.lost) {
        throw new BridgeError(
          "INTERNAL_ERROR",
          "Simulated lost transport acknowledgement"
        );
      }
      return {
        manifest,
        write: {
          aliases: {},
          changedIds: [],
          projectId: input.projectId,
          revision: 1,
          summary: "unit mock acknowledgement",
          warnings: [],
        },
      } as T;
    },
  };
  const speech = new SpeechApplicationService(f.provider, headless, 60_000);
  const workflow = new NarrationBlocksWorkflow(headless, speech);
  return {
    ...f,
    commits: () => commits,
    control,
    directory,
    dispose: async () => {
      await workflow.close();
      await rm(directory, { force: true, recursive: true });
    },
    eraseManifest: () => {
      manifest = undefined;
    },
    manifest: () => manifest,
    planningBudgets,
    speech,
    workflow,
  };
}
const commit = (groupId: string) => ({
  expectedRevision: 0,
  groupId,
  projectId: "project",
  startMs: 100,
  trackId: "audio",
});

it("partial generation never calls publication and explicit resume preserves completed token", async () => {
  const f = await setup();
  try {
    // The provider fixture owns deterministic failure injection.
    const original = f.provider.synthesize;
    let count = 0;
    f.provider.synthesize = async (...args) => {
      count += 1;
      if (count === 2) {
        throw new BridgeError("TTS_PROVIDER_FAILED", "second block failed");
      }
      return await original(...args);
    };
    const partial = await f.workflow.preview(plan(), context());
    expect(partial.ready).toBe(false);
    expect(partial.completed).toHaveLength(1);
    expect(f.commits()).toBe(0);
    const resumed = await f.workflow.preview(
      plan(),
      context(),
      partial.groupId
    );
    expect(resumed.ready).toBe(true);
    expect(resumed.completed[0]?.token).toBe(partial.completed[0]?.token);
    expect(f.requests).toHaveLength(2);
    const result = await f.workflow.commit(commit(resumed.groupId), context());
    expect(result.write.revision).toBe(1);
    expect(f.commits()).toBe(1);
  } finally {
    await f.dispose();
  }
});

it.each(["exact", "different-policy", "absent-after-undo"] as const)(
  "reconciles only exact published native intent (%s)",
  async (mode) => {
    const f = await setup();
    try {
      const previews = await f.workflow.preview(plan(), context());
      f.control.lost = true;
      await expect(
        f.workflow.commit(commit(previews.groupId), context())
      ).rejects.toBeInstanceOf(NarrationPublicationUnknownError);
      if (mode === "different-policy") {
        const manifest = f.manifest();
        if (!manifest) {
          throw new Error("unit manifest absent");
        }
        manifest.markerPolicy = { type: "sentence" };
      }
      if (mode === "absent-after-undo") {
        f.eraseManifest();
      }
      const result = await f.workflow.reconcile(previews.groupId);
      expect(result.reconciled).toBe(mode === "exact");
      expect(result.publicationUnknown).toBe(mode !== "exact");
      expect(f.commits()).toBe(1);
      if (mode !== "exact") {
        await expect(
          f.workflow.commit(commit(previews.groupId), context())
        ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
        expect(f.cleaned).toEqual([]);
        expect(
          (await f.workflow.discard(previews.groupId, true)).publicationUnknown
        ).toBe(true);
      }
    } finally {
      await f.dispose();
    }
  }
);

it("close computes publication uncertainty after an active lost acknowledgement settles", async () => {
  const f = await setup();
  const gate = Promise.withResolvers<void>();
  const entered = Promise.withResolvers<void>();
  try {
    const previews = await f.workflow.preview(plan(), context());
    f.control.lost = true;
    f.control.gate = gate.promise;
    f.control.beforeCommit = () => entered.resolve();
    const publishing = f.workflow.commit(commit(previews.groupId), context());
    await entered.promise;
    const closing = f.workflow.close();
    expect(f.closed).not.toHaveBeenCalled();
    gate.resolve();
    await expect(publishing).rejects.toBeInstanceOf(
      NarrationPublicationUnknownError
    );
    expect((await closing).publicationUnknown).toBe(true);
    expect(f.commits()).toBe(1);
  } finally {
    gate.resolve();
    await f.dispose();
  }
});

it("close waits for late preview generation before sweeping retained output", async () => {
  const f = await setup();
  const gate = Promise.withResolvers<void>();
  const entered = Promise.withResolvers<void>();
  try {
    const original = f.provider.synthesize;
    f.provider.synthesize = async (...args) => {
      entered.resolve();
      await gate.promise;
      return await original(...args);
    };
    const generation = f.workflow.preview(plan(), context());
    await entered.promise;
    const closing = f.workflow.close();
    await Promise.resolve();
    expect(f.closed).not.toHaveBeenCalled();
    expect(f.cleaned).toEqual([]);
    gate.resolve();
    expect((await generation).ready).toBe(true);
    await closing;
    expect(f.cleaned).toHaveLength(2);
    expect(f.closed).toHaveBeenCalledOnce();
  } finally {
    gate.resolve();
    await f.dispose();
  }
});

it("reports failed retained-file cleanup while still retiring the provider and native transport", async () => {
  const { closeNarrationSession } = await import(
    "../src/narration-blocks/lifecycle"
  );
  const f = await setup();
  const { cleanup } = f.provider;
  try {
    await f.workflow.preview(plan(), context());
    f.provider.cleanup = () =>
      Promise.reject(new Error("owned file unavailable"));
    const headless = { close: vi.fn(() => Promise.resolve()) };
    const guard = { release: vi.fn() };
    const closed = await closeNarrationSession({
      guard,
      headless,
      workflow: f.workflow,
    });
    expect(closed.cleanupConfirmed).toBe(false);
    expect(guard.release).toHaveBeenCalledWith(false);
    expect(headless.close).toHaveBeenCalledOnce();
    expect(f.closed).toHaveBeenCalledOnce();
  } finally {
    f.provider.cleanup = cleanup;
    await f.dispose();
  }
});

it("preserves lost acknowledgement uncertainty when retained-file cleanup also fails", async () => {
  const { closeNarrationSession } = await import(
    "../src/narration-blocks/lifecycle"
  );
  const f = await setup();
  const { cleanup } = f.provider;
  try {
    const group = await f.workflow.preview(plan(), context());
    f.control.lost = true;
    await expect(
      f.workflow.commit(commit(group.groupId), context())
    ).rejects.toBeInstanceOf(NarrationPublicationUnknownError);
    f.provider.cleanup = () => Promise.reject(new Error("cleanup refused"));
    const guard = { release: vi.fn() };
    const result = await closeNarrationSession({ guard, workflow: f.workflow });
    expect(result.publicationUnknown).toBe(true);
    expect(result.cleanupConfirmed).toBe(false);
    expect(guard.release).toHaveBeenCalledWith(false);
  } finally {
    f.provider.cleanup = cleanup;
    await f.dispose();
  }
});

it("rejects the fully prepared 512KiB group before the first inference", async () => {
  const { randomUUID } = await import("node:crypto");
  const f = await setup();
  try {
    const base = plan();
    const [first] = base.blocks;
    if (!first) {
      throw new Error("Missing fixture block");
    }
    const text = "A".repeat(32);
    const pronunciations = [
      { spoken: "x".repeat(128), term: "A" },
      ...Array.from({ length: 79 }, (_, index) => ({
        spoken: "x".repeat(128),
        term: `unused-${index}`,
      })),
    ];
    const input = {
      ...base,
      blocks: Array.from({ length: 32 }, (_, index) => ({
        ...structuredClone(first),
        displayText: text,
        endScalar: (index + 1) * 32,
        id: randomUUID(),
        request: {
          ...first.request,
          text,
          textOptions: { ...first.request.textOptions, pronunciations },
        },
        startScalar: index * 32,
      })),
      sourceText: text.repeat(32),
    };
    expect(Buffer.byteLength(JSON.stringify(input))).toBeLessThan(524_288);
    await expect(f.workflow.preview(input, context())).rejects.toMatchObject({
      code: "VALIDATION_FAILED",
    });
    expect(f.requests).toEqual([]);
  } finally {
    await f.dispose();
  }
});

it("bounds four retained groups and128 live tokens, then releases only discarded ownership", async () => {
  const { randomUUID } = await import("node:crypto");
  const f = await setup();
  try {
    const base = plan();
    const [first] = base.blocks;
    if (!first) {
      throw new Error("Missing fixture block");
    }
    const input = {
      ...base,
      blocks: Array.from({ length: 32 }, (_, index) => ({
        ...structuredClone(first),
        displayText: "A. ",
        endScalar: (index + 1) * 3,
        id: randomUUID(),
        startScalar: index * 3,
      })),
      sourceText: "A. ".repeat(32),
    };
    const groups: Awaited<ReturnType<NarrationBlocksWorkflow["preview"]>>[] =
      [];
    for (let index = 0; index < 4; index += 1) {
      // biome-ignore lint/performance/noAwaitInLoops: Prove admission at each complete retained group boundary.
      groups.push(await f.workflow.preview(input, context()));
    }
    expect(groups.every((entry) => entry.ready)).toBe(true);
    expect(f.requests).toHaveLength(128);
    await expect(f.workflow.preview(input, context())).rejects.toMatchObject({
      code: "VALIDATION_FAILED",
    });
    await expect(
      f.speech.previewResolvedNarration(first.request, 4096, context())
    ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
    expect(f.requests).toHaveLength(128);
    const [group] = groups;
    if (!group) {
      throw new Error("Missing retained group");
    }
    await f.workflow.discard(group.groupId);
    expect((await f.workflow.preview(input, context())).ready).toBe(true);
    expect(f.requests).toHaveLength(160);
  } finally {
    await f.dispose();
  }
});

it("enforces each request's escaped JSON byte budget before inference even for a supplied plan", async () => {
  const f = await setup();
  try {
    const base = plan();
    const [first] = base.blocks;
    if (!first) {
      throw new Error("Missing fixture block");
    }
    const textOptions = (count: number) => ({
      ...first.request.textOptions,
      pronunciations: Array.from({ length: count }, (_, index) => ({
        spoken: "\0".repeat(256),
        term: `${index.toString().padStart(3, "0")}${"\0".repeat(125)}`,
      })),
    });
    const over = {
      ...base,
      blocks: [
        {
          ...first,
          request: { ...first.request, textOptions: textOptions(100) },
        },
      ],
      sourceText: first.displayText,
    };
    expect(Buffer.byteLength(JSON.stringify(over))).toBeLessThan(524_288);
    expect(
      Buffer.byteLength(JSON.stringify(over.blocks.at(0)?.request))
    ).toBeGreaterThan(131_072);
    await expect(f.workflow.preview(over, context())).rejects.toMatchObject({
      code: "VALIDATION_FAILED",
    });
    expect(f.requests).toEqual([]);
    const under = {
      ...over,
      blocks: [
        {
          ...first,
          request: { ...first.request, textOptions: textOptions(50) },
        },
      ],
    };
    const group = await f.workflow.preview(under, context());
    expect(group.ready).toBe(true);
    expect(f.requests).toHaveLength(1);
    await f.workflow.discard(group.groupId);
  } finally {
    await f.dispose();
  }
});

it.each([0, 4097, Number.MAX_SAFE_INTEGER])(
  "source CLI preserves invalid planning budget %s for core rejection without inference",
  async (maxTextCharacters) => {
    const f = await setup();
    const prepared = vi.spyOn(f.speech, "prepareNarrationRequest");
    try {
      async function* input() {
        await Promise.resolve();
        yield new TextEncoder().encode(
          `${JSON.stringify({
            command: "plan",
            input: { ...narrationFixture.wire.planInput, maxTextCharacters },
          })}\n`
        );
      }
      const lines: unknown[] = [];
      expect(
        await runNarrationCli(
          input(),
          f.workflow,
          async (line) => {
            await Promise.resolve();
            lines.push(JSON.parse(line));
          },
          context
        )
      ).toBe(1);
      expect(lines).toMatchObject([{ error: { code: "VALIDATION_FAILED" } }]);
      expect(f.planningBudgets).toEqual([maxTextCharacters]);
      expect(prepared).not.toHaveBeenCalled();
      expect(f.requests).toHaveLength(0);
      expect(f.commits()).toBe(0);
    } finally {
      await f.dispose();
    }
  }
);

it("reduces a valid4096 planning budget to provider capacity before core planning", async () => {
  const f = await setup();
  const original = f.provider.status;
  f.provider.status = async () => {
    const status = await original();
    return { ...status, limits: { ...status.limits, maxTextCharacters: 8 } };
  };
  try {
    const result = await f.workflow.plan(narrationFixture.wire.planInput);
    expect(result.maxTextCharacters).toBe(8);
    expect(f.planningBudgets).toEqual([8]);
    expect(f.requests).toHaveLength(0);
    expect(f.commits()).toBe(0);
  } finally {
    await f.dispose();
  }
});
