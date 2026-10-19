import { z } from "zod/v4";
import { closedSlotRecord } from "../schemas";

const closedObject = <T extends z.ZodRawShape>(shape: T) =>
  closedSlotRecord(z.strictObject(shape), Object.keys(shape));

export const PLANNING_LIMITS = {
  candidates: 32,
  chunks: 128,
  inputBytes: 4_194_304,
  outputBytes: 8_388_608,
  segments: 4096,
  sourceDurationMs: 86_400_000,
  words: 32_768,
} as const;

const time = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);
const identifier = z
  .string()
  .min(1)
  .max(128)
  .refine((s) => s.trim().length > 0);
const interval = { endMs: time, startMs: time };
const wordSchema = closedObject({
  word: z.string().min(1).max(256),
  ...interval,
});
const segmentSchema = closedObject({
  id: identifier,
  ...interval,
  text: z
    .string()
    .min(1)
    .max(2000)
    .refine((s) => s.trim().length > 0),
  words: z.array(wordSchema).max(PLANNING_LIMITS.words).optional(),
});
const chunkMetadata = {
  durationMs: time.positive().max(PLANNING_LIMITS.sourceDurationMs),
  id: identifier,
  maxDurationMs: time.positive(),
  modelId: identifier,
  offsetMs: time,
  providerId: identifier,
};
const editSchema = closedObject({
  candidateId: identifier,
  cta: z.string().max(1000).optional(),
  endMs: time.optional(),
  hook: z.string().max(1000).optional(),
  rationale: z.string().max(1000).optional(),
  startMs: time.optional(),
});

export const shortPlanningInputSchema = closedObject({
  approvedTranscript: z.literal(true),
  assetId: identifier,
  chunks: z
    .array(
      closedObject({
        ...chunkMetadata,
        segments: z.array(segmentSchema).min(1).max(PLANNING_LIMITS.segments),
      })
    )
    .min(1)
    .max(PLANNING_LIMITS.chunks),
  edits: z.array(editSchema).max(PLANNING_LIMITS.candidates).optional(),
  expectedRevision: time,
  maxCandidates: time.min(1).max(PLANNING_LIMITS.candidates),
  projectId: identifier,
  selection: closedObject(interval).optional(),
  targetDurationMs: time.min(1000).max(180_000),
  version: z.literal(1),
});

export const PLANNING_WARNINGS = [
  "context_review",
  "editorial_claim_review",
  "cut_boundary",
  "context_excluded",
  "duration_mismatch",
  "chunk_boundary",
  "candidate_overlap",
] as const;
export type PlanningWarning = (typeof PLANNING_WARNINGS)[number];

export const contentHashSchema = closedObject({
  algorithm: z.literal("sha256"),
  digest: z.string().regex(/^[0-9a-f]{64}$/u),
});
const sourceFields = {
  assetId: identifier,
  contentHash: contentHashSchema,
  durationMs: time.positive().max(PLANNING_LIMITS.sourceDurationMs),
  projectId: identifier,
  revision: time,
};
export const shortPlanningSourceSchema = closedObject(sourceFields);
const evidenceSchema = closedObject({
  chunkId: identifier,
  segmentId: identifier,
  ...interval,
  fullyIncluded: z.boolean(),
  text: z.string().min(1).max(2000),
  words: z.array(wordSchema).max(PLANNING_LIMITS.words).optional(),
});
export const shortPlanningSuccessSchema = closedObject({
  candidates: z
    .array(
      closedObject({
        id: identifier,
        ...interval,
        context: closedObject({
          after: evidenceSchema.nullable(),
          before: evidenceSchema.nullable(),
        }),
        durationMs: time.positive(),
        editorial: closedObject({
          cta: z.string().max(1000),
          hook: z.string().max(1000),
          rationale: z.string().max(1000),
        }),
        evidence: z.array(evidenceSchema).min(1).max(PLANNING_LIMITS.segments),
        warnings: z
          .array(z.enum(PLANNING_WARNINGS))
          .min(1)
          .max(PLANNING_WARNINGS.length),
      })
    )
    .min(1)
    .max(PLANNING_LIMITS.candidates),
  ok: z.literal(true),
  selection: closedObject(interval),
  source: closedObject({
    ...sourceFields,
    chunks: z
      .array(closedObject(chunkMetadata))
      .min(1)
      .max(PLANNING_LIMITS.chunks),
    transcriptApproved: z.literal(true),
    transcriptionProvenance: z.literal("supplied"),
  }),
  targetDurationMs: time.min(1000).max(180_000),
  version: z.literal(1),
});

export const PLANNING_MESSAGES = {
  core: "The core could not resolve the approved managed source.",
  input: "Invalid approved source transcript, selection or candidate edit.",
  internal: "Local candidate planning could not complete.",
  limit:
    "Planning capacity exceeded. Supply less transcript data or select a smaller interval.",
  source:
    "The managed source does not match a supported, complete transcript identity.",
  stale:
    "The source revision or identity changed. Read it again and explicitly approve a new input.",
} as const;
export type PlanningKind = keyof typeof PLANNING_MESSAGES;
export class PlanningError extends Error {
  readonly kind: Exclude<PlanningKind, "core">;
  constructor(kind: Exclude<PlanningKind, "core">, options?: ErrorOptions) {
    super(PLANNING_MESSAGES[kind], options);
    this.kind = kind;
  }
}
export const shortPlanningFailureSchema = closedObject({
  error: z.union([
    closedObject({
      kind: z.enum(["input", "limit", "source", "stale", "internal"]),
      message: z.string().min(1).max(256),
    }).refine((error) => error.message === PLANNING_MESSAGES[error.kind]),
    closedObject({
      code: z.string().min(1).max(128),
      kind: z.literal("core"),
      message: z.literal(PLANNING_MESSAGES.core),
      retryable: z.boolean(),
    }),
  ]),
  ok: z.literal(false),
  version: z.literal(1),
});

export type ShortPlanningInput = z.infer<typeof shortPlanningInputSchema>;
export type ShortPlanningSource = z.infer<typeof shortPlanningSourceSchema>;
export type ShortPlanningSuccess = z.infer<typeof shortPlanningSuccessSchema>;
export type PlanningEvidence =
  ShortPlanningSuccess["candidates"][number]["evidence"][number];
