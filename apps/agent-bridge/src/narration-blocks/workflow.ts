import { createHash, randomUUID } from "node:crypto";
import { createReadStream } from "node:fs";
import { isDeepStrictEqual } from "node:util";
import type { z } from "zod/v4";
import { BridgeError, type HeadlessClient } from "../headless";
import type { JobTaskContext } from "../jobs";
import {
  narrationPlanInputSchema,
  narrationPlanSchema,
  type narrationPreparedBlockSchema,
  projectStateSchema,
} from "../schemas";
import {
  prepareSpeechSegments,
  type SpeechApplicationService,
} from "../speech";
import { assertNarrationJsonBudget } from "./budget";
import { NarrationPublicationUnknownError } from "./errors";
import {
  commitNarrationBlocksInputSchema,
  commitNarrationBlocksResultSchema,
  type narrationCaptionOptionsSchema,
} from "./transport";

type Plan = z.infer<typeof narrationPlanSchema>;
type Prepared = z.infer<typeof narrationPreparedBlockSchema>;
type CommitInput = z.infer<typeof commitNarrationBlocksInputSchema>;
interface Completed {
  durationMs: number;
  expiresAtMs: number;
  prepared: Prepared;
  token: string;
}
interface Group {
  completed: Completed[];
  expected?: { input: CommitInput; hashes: string[] };
  plan: Plan;
  prepared: Prepared[];
  state: "partial" | "ready" | "quarantined";
}

const resumable = (group: Group | undefined, plan: Plan) =>
  !!group &&
  group.state !== "quarantined" &&
  isDeepStrictEqual(group.plan, plan);

const invalid = (message: string) =>
  new BridgeError("VALIDATION_FAILED", message);
export const narrationFailure = (error: unknown) => ({
  code: error instanceof BridgeError ? error.code : "INTERNAL_ERROR",
  message:
    error instanceof NarrationPublicationUnknownError
      ? error.message
      : "Narration command failed; inspect the current group, provider and project before an explicit retry",
  retryable: error instanceof BridgeError && error.retryable,
});

const at = <T>(values: readonly T[], index: number): T => {
  const value = values[index];
  if (value === undefined) {
    throw invalid("Narration group has an incomplete internal mapping");
  }
  return value;
};

const hashFile = async (path: string) => {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(path)) {
    hash.update(chunk);
  }
  return hash.digest("hex");
};

export class NarrationBlocksWorkflow {
  readonly #headless: Pick<HeadlessClient, "call">;
  readonly #speech: SpeechApplicationService;
  readonly #groups = new Map<string, Group>();
  #active: Promise<unknown> | undefined;
  #closed = false;

  constructor(
    headless: Pick<HeadlessClient, "call">,
    speech: SpeechApplicationService
  ) {
    this.#headless = headless;
    this.#speech = speech;
  }

  #busy() {
    return this.#active !== undefined;
  }

  async #exclusive<T>(run: () => Promise<T>) {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: close changes lifecycle state between commands.
    if (this.#closed || this.#busy()) {
      throw invalid(
        "Narration session is closed or already processing a command"
      );
    }
    const work = Promise.resolve().then(run);
    this.#active = work;
    try {
      return await work;
    } finally {
      this.#active = undefined;
    }
  }

  async plan(input: z.infer<typeof narrationPlanInputSchema>) {
    return await this.#exclusive(async () => {
      const status = await this.#speech.status();
      const parsed = narrationPlanInputSchema.parse(input);
      // Preserve invalid caller budgets for authoritative core rejection.
      const budget =
        parsed.maxTextCharacters >= 1 && parsed.maxTextCharacters <= 4096
          ? Math.min(status.limits.maxTextCharacters, parsed.maxTextCharacters)
          : parsed.maxTextCharacters;
      const plan = await this.#headless.call(
        {
          input: { ...parsed, maxTextCharacters: budget },
          operation: "narration_blocks_plan",
        },
        narrationPlanSchema
      );
      // Every resolved block is checked before the caller can request inference.
      for (const block of plan.blocks) {
        // biome-ignore lint/performance/noAwaitInLoops: Check each pinned request before allowing inference.
        await this.#speech.prepareNarrationRequest(
          block.request,
          plan.maxTextCharacters
        );
      }
      return plan;
    });
  }

  async #prepareBlocks(plan: Plan, context: JobTaskContext) {
    const prepared: Prepared[] = [];
    for (const block of plan.blocks) {
      if (context.signal.aborted) {
        throw new BridgeError(
          "JOB_CANCELLED",
          "Narration preflight was cancelled"
        );
      }
      // biome-ignore lint/performance/noAwaitInLoops: Preflight every block before starting any inference.
      const resolved = await this.#speech.prepareNarrationRequest(
        block.request,
        plan.maxTextCharacters
      );
      prepared.push({
        block: structuredClone(block),
        spokenSegments: resolved.spokenSegments,
        spokenText: resolved.spokenText,
      });
    }

    return prepared;
  }

  async preview(
    input: Plan,
    context: JobTaskContext,
    existingGroupId?: string
  ) {
    return await this.#exclusive(async () => {
      assertNarrationJsonBudget(input, 524_288);
      const plan = narrationPlanSchema.parse(input);
      if (plan.blocks.length === 0 || plan.blocks.length > 32) {
        throw invalid("Narration plan exceeds bounded group size");
      }
      let id = existingGroupId;
      let group = id ? this.#groups.get(id) : undefined;
      if (id && !resumable(group, plan)) {
        throw invalid(
          "Explicit preview resume requires the same non-quarantined retained plan"
        );
      }
      if (!group) {
        if (this.#groups.size >= 4) {
          throw invalid("Narration session retains at most four groups");
        }
        const prepared = await this.#prepareBlocks(plan, context);
        assertNarrationJsonBudget(
          {
            blocks: prepared,
            id: plan.id,
            maxTextCharacters: plan.maxTextCharacters,
            sourceText: plan.sourceText,
          },
          524_288
        );
        id = randomUUID();
        group = {
          completed: [],
          plan: structuredClone(plan),
          prepared,
          state: "partial",
        };
        this.#groups.set(id, group);
      }
      const groupId = id as string;
      try {
        for (
          let index = group.completed.length;
          index < group.prepared.length;
          index += 1
        ) {
          if (context.signal.aborted) {
            throw new BridgeError(
              "JOB_CANCELLED",
              "Narration preview was cancelled"
            );
          }
          const prepared = at(group.prepared, index);
          // biome-ignore lint/performance/noAwaitInLoops: Generate and retain blocks in source order so explicit resume skips completed work.
          const result = await this.#speech.previewResolvedNarration(
            prepared.block.request,
            plan.maxTextCharacters,
            context
          );
          group.completed.push({
            durationMs: result.durationMs,
            expiresAtMs: result.expiresAtMs,
            prepared,
            token: result.token,
          });
        }
        group.state = "ready";
        return {
          completed: group.completed.map(
            ({ token, expiresAtMs, durationMs }, index) => ({
              blockId: at(group.plan.blocks, index).id,
              durationMs,
              expiresAtMs,
              token,
            })
          ),
          groupId,
          publicationUnknown: false,
          ready: true,
        };
      } catch (error) {
        return {
          completed: group.completed.map(
            ({ token, expiresAtMs, durationMs }, index) => ({
              blockId: at(group.plan.blocks, index).id,
              durationMs,
              expiresAtMs,
              token,
            })
          ),
          error: narrationFailure(error),
          groupId,
          publicationUnknown: false,
          ready: false,
        };
      }
    });
  }

  async #collectArtifacts(
    artifacts: Parameters<
      Parameters<SpeechApplicationService["withNarrationPreviews"]>[1]
    >[0],
    group: Group
  ) {
    const blocks: CommitInput["blocks"] = [];
    const hashes: string[] = [];
    for (let index = 0; index < artifacts.length; index += 1) {
      const artifact = at(artifacts, index);
      const prepared = at(group.prepared, index);
      if (
        !isDeepStrictEqual(artifact.generated.request, prepared.block.request)
      ) {
        throw invalid(
          "Retained generated intent no longer matches the approved block"
        );
      }
      const fresh = prepareSpeechSegments(artifact.generated.request);
      if (
        !isDeepStrictEqual(fresh, prepared.spokenSegments) ||
        fresh.join("\n") !== prepared.spokenText
      ) {
        throw invalid(
          "Retained spoken preparation differs from the approved plan"
        );
      }
      // biome-ignore lint/performance/noAwaitInLoops: Hash one retained artifact before preparing its corresponding native descriptor.
      hashes.push(await hashFile(artifact.generated.outputPath));
      blocks.push({
        durationMs: artifact.generated.durationMs,
        origin: artifact.origin,
        path: artifact.generated.outputPath,
        prepared: structuredClone(prepared),
        probe: {
          audioChannels: null,
          audioCodec: null,
          audioSampleRateHz: artifact.generated.sampleRateHz,
          durationMs: artifact.generated.durationMs,
          formatName: null,
          hasAudio: true,
          hasVideo: false,
          videoCodec: null,
          videoHeight: null,
          videoWidth: null,
        },
      });
    }

    return { blocks, hashes };
  }

  async commit(
    input: {
      groupId: string;
      projectId: string;
      expectedRevision: number;
      trackId: string;
      startMs: number;
      markerPolicy?: CommitInput["markerPolicy"];
      caption?: z.infer<typeof narrationCaptionOptionsSchema> | undefined;
    },
    context: JobTaskContext
  ) {
    return await this.#exclusive(async () => {
      const group = this.#groups.get(input.groupId);
      if (
        group?.state !== "ready" ||
        group.completed.length !== group.plan.blocks.length
      ) {
        throw invalid(
          "A complete ready group is required; uncertain groups cannot be republished"
        );
      }
      try {
        const published = await this.#speech.withNarrationPreviews(
          group.completed.map((c) => c.token),
          async (artifacts, beginPublication) => {
            const { blocks, hashes } = await this.#collectArtifacts(
              artifacts,
              group
            );
            const request = commitNarrationBlocksInputSchema.parse({
              blocks,
              expectedRevision: input.expectedRevision,
              plan: group.plan,
              projectId: input.projectId,
              startMs: input.startMs,
              trackId: input.trackId,
              ...(input.markerPolicy === undefined
                ? {}
                : { markerPolicy: input.markerPolicy }),
              ...(input.caption === undefined
                ? {}
                : { caption: input.caption }),
            });
            assertNarrationJsonBudget(request, 16_777_216);
            if (context.signal.aborted) {
              throw new BridgeError(
                "JOB_CANCELLED",
                "Narration commit was cancelled before publication"
              );
            }
            group.expected = { hashes, input: structuredClone(request) };
            // Publication itself is one noncancellable core transaction. No retry.
            context.markNonCancellable();
            beginPublication();
            return await this.#headless.call(
              { input: request, operation: "commit_narration_blocks" },
              commitNarrationBlocksResultSchema
            );
          }
        );
        this.#groups.delete(input.groupId);
        return {
          ...published.value,
          publicationUnknown: false,
          write: {
            ...published.value.write,
            warnings: [
              ...published.value.write.warnings,
              ...published.warnings,
            ],
          },
        };
      } catch (error) {
        if (error instanceof NarrationPublicationUnknownError) {
          group.state = "quarantined";
        }
        throw error;
      }
    });
  }

  async reconcile(groupId: string) {
    return await this.#exclusive(async () => {
      const group = this.#groups.get(groupId);
      if (group?.state !== "quarantined" || !group.expected) {
        throw invalid(
          "Reconciliation requires a retained uncertain publication"
        );
      }
      const { expected } = group;
      const state = await this.#headless.call(
        { operation: "get_state", projectId: expected.input.projectId },
        projectStateSchema
      );
      const manifest = state.project.narrationManifests.find(
        (m) => m.id === group.plan.id
      );
      let matches =
        !!manifest &&
        manifest.sourceText === group.plan.sourceText &&
        manifest.maxTextCharacters === group.plan.maxTextCharacters &&
        manifest.blocks.length === group.prepared.length &&
        isDeepStrictEqual(
          manifest.markerPolicy,
          expected.input.markerPolicy ?? { type: "none" }
        ) &&
        isDeepStrictEqual(manifest.caption, expected.input.caption);
      let start = expected.input.startMs;
      for (
        let index = 0;
        matches && manifest && index < manifest.blocks.length;
        index += 1
      ) {
        const actual = at(manifest.blocks, index);
        const { binding, spokenSegments, spokenText, ...block } = actual;
        const prepared = at(group.prepared, index);
        const generated = at(expected.input.blocks, index);
        const asset = state.project.assets.find(
          (a) => a.id === binding.assetId
        );
        matches =
          isDeepStrictEqual(block, prepared.block) &&
          isDeepStrictEqual(spokenSegments, prepared.spokenSegments) &&
          spokenText === prepared.spokenText &&
          binding.trackId === expected.input.trackId &&
          binding.startMs === start &&
          binding.durationMs === generated.durationMs &&
          binding.sourceInMs === 0 &&
          asset !== undefined &&
          asset.contentHash.digest === expected.hashes[index] &&
          isDeepStrictEqual(asset.origin, generated.origin);
        start += generated.durationMs;
      }
      if (!matches) {
        return {
          groupId,
          manifestId: group.plan.id,
          publicationUnknown: true,
          reconciled: false,
        };
      }
      const warnings = await this.#speech.acknowledgeNarrationPreviews(
        group.completed.map((c) => c.token)
      );
      this.#groups.delete(groupId);
      return {
        groupId,
        manifestId: group.plan.id,
        publicationUnknown: false,
        reconciled: true,
        revision: state.project.revision,
        warnings,
      };
    });
  }

  async discard(groupId: string, abandon = false) {
    return await this.#exclusive(async () => {
      const group = this.#groups.get(groupId);
      if (!group) {
        throw invalid("Narration group is missing");
      }
      const publicationUnknown = group.state === "quarantined";
      if (publicationUnknown && !abandon) {
        throw invalid(
          "Uncertain groups require explicit reconcile or abandon, never discard/replay"
        );
      }
      const warnings = await this.#speech.abandonNarrationPreviews(
        group.completed.map((c) => c.token)
      );
      this.#groups.delete(groupId);
      return { discarded: true, groupId, publicationUnknown, warnings };
    });
  }

  publicationStatus(groupId: string) {
    const group = this.#groups.get(groupId);
    return {
      groupId,
      ...(group
        ? {
            manifestId: group.plan.id,
            ...(group.expected
              ? { projectId: group.expected.input.projectId }
              : {}),
          }
        : {}),
    };
  }

  async close(): Promise<{
    publicationUnknown: boolean;
    cleanupErrors?: unknown[];
  }> {
    this.#closed = true;
    await Promise.allSettled(this.#active ? [this.#active] : []);
    const publicationUnknown = [...this.#groups.values()].some(
      (group) => group.state === "quarantined"
    );
    try {
      await this.#speech.closeNarration();
    } catch (error) {
      return { cleanupErrors: [error], publicationUnknown };
    }
    this.#groups.clear();
    return { publicationUnknown };
  }
}
