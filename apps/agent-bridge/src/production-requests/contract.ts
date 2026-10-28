import { createHash } from "node:crypto";
import { z } from "zod/v4";
import { errorDefinition } from "../errors";
import { BridgeError } from "../headless";
import { closedSlotRecord, jobSchema, schemas } from "../schemas";

export const DURABLE_CAPABILITY = "durable_requests_v1";
export const durableCapabilities = (enabled: boolean) =>
  enabled ? [DURABLE_CAPABILITY] : [];
export const durableHttpHealth = <T extends Record<string, unknown>>(
  enabled: boolean,
  health: T
) => (enabled ? { ...health, capabilities: [DURABLE_CAPABILITY] } : health);
export const DURABLE_LIMITS = Object.freeze({
  catalogBytes: 1_048_576,
  intentBytes: 4096,
  recordBytes: 8192,
  records: 64,
  retentionMs: 604_800_000,
});
export const DURABLE_FILES = Object.freeze({
  catalog: "catalog-v1.json",
  directory: ".opencut-durable-production-v1",
  stage: "catalog-v1.staging",
});

const closed = <T extends z.ZodRawShape>(
  shape: T,
  io: "input" | "output" = "input"
) => {
  const schema = z.strictObject(shape);
  const { $schema: _dialect, ...metadata } = z.toJSONSchema(schema, {
    io,
  });
  return closedSlotRecord(schema, Object.keys(shape)).meta(metadata);
};
const uuid = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/
  );
const digest = z.string().regex(/^[0-9a-f]{64}$/);
const timestamp = z.int().nonnegative();

export const canonicalJson = (value: unknown): string => {
  const normalize = (input: unknown): unknown => {
    if (Array.isArray(input)) {
      return input.map(normalize);
    }
    if (input !== null && typeof input === "object") {
      return Object.fromEntries(
        Object.entries(input)
          .sort(([left], [right]) => {
            if (left === right) {
              return 0;
            }
            return left < right ? -1 : 1;
          })
          .map(([key, item]) => [key, normalize(item)])
      );
    }
    if (
      input === null ||
      typeof input === "string" ||
      typeof input === "boolean" ||
      (typeof input === "number" && Number.isFinite(input))
    ) {
      return input;
    }
    throw new BridgeError("VALIDATION_FAILED", "Invalid durable metadata");
  };
  return JSON.stringify(normalize(value));
};

export const sha256 = (value: string) =>
  createHash("sha256").update(value).digest("hex");

// Reuse the original validators and defaults; only the new envelope rejects raw
// special keys before Zod can discard them, including custom resolution keys.
const boundedRequest = <T extends z.ZodType>(schema: T) => {
  const { $schema: _dialect, ...metadata } = z.toJSONSchema(schema, {
    io: "input",
  });
  const allowed = Object.keys(metadata.properties ?? {});
  const raw = z
    .unknown()
    .check((context) => {
      const input = context.value;
      if (
        input !== null &&
        typeof input === "object" &&
        "resolution" in input
      ) {
        const { resolution } = input;
        if (
          resolution !== null &&
          typeof resolution === "object" &&
          Object.keys(resolution).some(
            (key) => !["width", "height"].includes(key)
          )
        ) {
          context.issues.push({
            code: "custom",
            input: undefined,
            message: "Invalid production resolution fields",
          });
        }
      }
    })
    .pipe(schema);
  return closedSlotRecord(raw, allowed).meta(metadata);
};

const frame = closed({
  kind: z.literal("preview_frame"),
  request: boundedRequest(schemas.previewRenderFrame),
});
const range = closed({
  kind: z.literal("preview_range"),
  request: boundedRequest(schemas.previewRenderRange),
});
const review = closed({
  kind: z.literal("review_range"),
  request: boundedRequest(schemas.previewReviewRange),
});
const exported = closed({
  kind: z.literal("export_video"),
  request: boundedRequest(schemas.projectExportVideo),
});

export const intentSchema = z
  .union([frame, range, review, exported])
  .refine(
    (value) =>
      Buffer.byteLength(canonicalJson(value)) <= DURABLE_LIMITS.intentBytes,
    "Production intent exceeds the storage limit"
  );
export type DurableIntent = z.output<typeof intentSchema>;
export const intentDigest = (intent: DurableIntent) =>
  sha256(canonicalJson(intent));

const stateSchema = z.enum([
  "queued",
  "running",
  "completed",
  "failed",
  "cancelled",
]);
export const outcomeSchema = z.union([
  closed({
    format: z.enum(["png", "mp4"]),
    kind: z.literal("rendered"),
    sizeBytes: z.int().positive(),
  }),
  closed({
    code: z.string().min(1).max(128),
    kind: z.literal("error"),
    retryable: z.boolean(),
  }),
  closed({ kind: z.literal("cancelled") }),
]);
export type DurableOutcome = z.output<typeof outcomeSchema>;

const hasOutcome = (
  state: string,
  intent: DurableIntent,
  outcome: DurableOutcome | null
) => {
  if (state === "queued" || state === "running") {
    return outcome === null;
  }
  if (state === "completed") {
    return (
      outcome?.kind === "rendered" &&
      outcome.format === (intent.kind === "preview_frame" ? "png" : "mp4")
    );
  }
  if (state === "cancelled") {
    return outcome?.kind === "cancelled";
  }
  if (state === "failed" && outcome?.kind === "error") {
    return errorDefinition(outcome.code)?.retryable === outcome.retryable;
  }
  return false;
};

export const storedRecordSchema = closed({
  createdAtMs: timestamp.max(
    Number.MAX_SAFE_INTEGER - DURABLE_LIMITS.retentionMs
  ),
  intent: intentSchema,
  intentSha256: digest,
  outcome: outcomeSchema.nullable(),
  requestId: uuid,
  runtimeId: uuid.nullable(),
  state: stateSchema,
  updatedAtMs: timestamp,
}).refine(
  (record) =>
    record.updatedAtMs >= record.createdAtMs &&
    record.intentSha256 === intentDigest(record.intent) &&
    (record.state !== "queued" || record.runtimeId === null) &&
    (["queued", "cancelled"].includes(record.state) ||
      record.runtimeId !== null) &&
    hasOutcome(record.state, record.intent, record.outcome) &&
    Buffer.byteLength(canonicalJson(record)) <= DURABLE_LIMITS.recordBytes,
  "Invalid durable record correlations"
);
export type StoredRecord = z.output<typeof storedRecordSchema>;

export const catalogSchema = closed({
  generation: timestamp,
  records: z.array(storedRecordSchema).max(DURABLE_LIMITS.records),
  rootBinding: digest,
  storeId: uuid,
  version: z.literal(1),
}).refine(
  (catalog) =>
    new Set(catalog.records.map((record) => record.requestId)).size ===
      catalog.records.length &&
    Buffer.byteLength(canonicalJson(catalog)) <= DURABLE_LIMITS.catalogBytes,
  "Invalid durable catalog bounds or identities"
);
export type DurableCatalog = z.output<typeof catalogSchema>;

export const publicRecordSchema = closed({
  confirmedState: stateSchema,
  createdAtMs: timestamp.max(
    Number.MAX_SAFE_INTEGER - DURABLE_LIMITS.retentionMs
  ),
  expiresAtMs: timestamp,
  intent: intentSchema,
  outcome: outcomeSchema.nullable(),
  requestId: uuid,
  state: z.enum([
    "queued",
    "running",
    "completed",
    "failed",
    "cancelled",
    "uncertain",
  ]),
  uncertainty: z
    .enum(["previous_runtime", "metadata_unconfirmed", "dispatch_unconfirmed"])
    .nullable(),
  updatedAtMs: timestamp,
  version: z.literal(1),
}).refine((record) => {
  if (
    record.updatedAtMs < record.createdAtMs ||
    record.expiresAtMs !== record.createdAtMs + DURABLE_LIMITS.retentionMs
  ) {
    return false;
  }
  if (record.state === "uncertain") {
    return (
      record.outcome === null &&
      ["queued", "running"].includes(record.confirmedState) &&
      record.uncertainty !== null &&
      (record.uncertainty === "metadata_unconfirmed" ||
        record.confirmedState === "running")
    );
  }
  return (
    record.confirmedState === record.state &&
    record.uncertainty === null &&
    hasOutcome(record.state, record.intent, record.outcome)
  );
}, "Invalid durable public state correlations");
export type PublicRecord = z.output<typeof publicRecordSchema>;

export const storageStatusSchema = closed({
  readable: z.boolean(),
  reason: z
    .enum([
      "guard_occupied",
      "metadata_unconfirmed",
      "unsafe_storage",
      "not_initialized",
    ])
    .nullable(),
  writable: z.boolean(),
}).refine(
  (status) => !status.writable || (status.readable && status.reason === null)
);
export type StorageStatus = z.output<typeof storageStatusSchema>;
export const retentionSchema = closed({
  absenceMeaning: z.literal("unknown"),
  expiredCount: z.int().min(0).max(DURABLE_LIMITS.records),
  maxAgeMs: z.literal(DURABLE_LIMITS.retentionMs),
});

export const identityInputSchema = closed({
  requestId: uuid,
  version: z.literal(1),
});
export const listInputSchema = closed({ version: z.literal(1) });
export const admissionInputSchema = closed({
  intent: intentSchema,
  requestId: uuid,
  version: z.literal(1),
});
export const parseAdmission = (input: unknown) => {
  const result = admissionInputSchema.safeParse(input);
  if (!result.success) {
    throw new BridgeError(
      "VALIDATION_FAILED",
      "Invalid durable production request"
    );
  }
  return result.data;
};

export const durableOutputSchemas = {
  admit: closed({
    created: z.boolean(),
    request: publicRecordSchema,
    version: z.literal(1),
  }),
  cancelQueued: closed({ request: publicRecordSchema, version: z.literal(1) }),
  get: closed({
    request: publicRecordSchema.nullable(),
    retention: retentionSchema,
    storage: storageStatusSchema,
    version: z.literal(1),
  }),
  list: closed({
    requests: z.array(publicRecordSchema).max(DURABLE_LIMITS.records),
    retention: retentionSchema,
    storage: storageStatusSchema,
    version: z.literal(1),
  }),
  start: closed(
    {
      job: jobSchema.nullable(),
      request: publicRecordSchema,
      version: z.literal(1),
    },
    "output"
  ),
};
