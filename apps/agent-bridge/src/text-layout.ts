import { z } from "zod/v4";

export const fontRecordSchema = z
  .object({
    faceIndex: z.literal(0),
    relativePath: z.string(),
    sha256: z.string().regex(/^[0-9a-f]{64}$/),
    sizeBytes: z.int().positive().max(16_777_216),
  })
  .strict();

export const fontBindingSchema = z
  .object({
    bold: z.string().regex(/^[0-9a-f]{64}$/),
    boldItalic: z.string().regex(/^[0-9a-f]{64}$/),
    italic: z.string().regex(/^[0-9a-f]{64}$/),
    profile: z.literal("opencut-text-v2"),
    regular: z.string().regex(/^[0-9a-f]{64}$/),
    warnings: z.array(z.string().max(512)).max(2),
  })
  .strict();

export const fontCatalogSchema = z.record(z.string(), fontRecordSchema);
export const fontStepsSchema = z
  .array(z.record(z.string(), fontBindingSchema))
  .max(100);
