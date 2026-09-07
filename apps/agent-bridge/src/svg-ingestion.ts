import { z } from "zod/v4";
import { shapeFields } from "./shape-items";
import { vectorPointSchema } from "./vector-primitives";

// Structural contract only. XML and normalized SVG semantics are core-owned.
export const svgDocumentSchema = z.strictObject({
  height: z.number().positive().max(16_384),
  shapes: z
    .array(z.strictObject({ ...shapeFields, offset: vectorPointSchema }))
    .max(4095),
  version: z.literal(1),
  viewBox: z.tuple([
    z.number(),
    z.number(),
    z.number().positive(),
    z.number().positive(),
  ]),
  width: z.number().positive().max(16_384),
});
