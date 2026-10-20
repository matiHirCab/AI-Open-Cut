import { z } from "zod/v4";
import {
  generatedAssetOriginSchema,
  narrationManifestSchema,
  narrationPlanInputSchema,
  narrationPlanSchema,
  narrationPreparedBlockSchema,
  resolvedAssetInputSchema,
  speechMarkerPolicySchema,
  writeResultSchema,
} from "../schemas";

const integer = z.int().nonnegative();
export const narrationCaptionOptionsSchema = z.strictObject({
  granularity: z.enum(["sentence", "word"]),
  trackId: z.string(),
});
export const commitNarrationBlocksInputSchema = z.strictObject({
  blocks: z.array(
    z.strictObject({
      durationMs: integer,
      origin: generatedAssetOriginSchema,
      path: z.string(),
      prepared: narrationPreparedBlockSchema,
      probe: resolvedAssetInputSchema.shape.probe.unwrap(),
    })
  ),
  caption: narrationCaptionOptionsSchema.optional(),
  expectedRevision: integer,
  markerPolicy: speechMarkerPolicySchema.optional(),
  plan: narrationPlanSchema,
  projectId: z.string(),
  startMs: integer,
  trackId: z.string(),
});
export const commitNarrationBlocksResultSchema = z.strictObject({
  manifest: narrationManifestSchema,
  write: writeResultSchema,
});
export const narrationRequestSchema = z.discriminatedUnion("operation", [
  z.strictObject({
    input: narrationPlanInputSchema,
    operation: z.literal("narration_blocks_plan"),
  }),
  z.strictObject({
    input: commitNarrationBlocksInputSchema,
    operation: z.literal("commit_narration_blocks"),
  }),
  z.strictObject({
    blockId: z.string(),
    expectedRevision: integer,
    manifestId: z.string(),
    operation: z.literal("detach_narration_block"),
    projectId: z.string(),
  }),
]);
export type NarrationRequest = z.infer<typeof narrationRequestSchema>;
