import { z } from "zod/v4";
import {
  cornerRadiiSchema,
  paintSchema,
  strokeSchema,
  vectorPathSchema,
  vectorPointSchema,
} from "./vector-primitives";

const dimension = z.number().gt(0).max(16_384);
export const shapeGeometrySchema = z.discriminatedUnion("type", [
  z.strictObject({
    height: dimension,
    type: z.literal("rectangle"),
    width: dimension,
  }),
  z.strictObject({
    height: dimension,
    radii: cornerRadiiSchema,
    type: z.literal("roundedRectangle"),
    width: dimension,
  }),
  z.strictObject({
    height: dimension,
    type: z.literal("ellipse"),
    width: dimension,
  }),
  z
    .strictObject({
      end: vectorPointSchema,
      start: vectorPointSchema,
      type: z.literal("line"),
    })
    .refine(
      (v) => v.start.x !== v.end.x || v.start.y !== v.end.y,
      "Line endpoints must differ"
    ),
  z.strictObject({
    points: z.array(vectorPointSchema).min(3).max(4096),
    type: z.literal("polygon"),
  }),
  z
    .strictObject({
      center: vectorPointSchema,
      innerRadius: dimension,
      outerRadius: dimension,
      pointCount: z.int().min(3).max(2048),
      rotationDeg: z.number().min(-36_000).max(36_000),
      type: z.literal("star"),
    })
    .refine(
      (v) => v.innerRadius < v.outerRadius,
      "Inner radius must be smaller"
    ),
  z.strictObject({ path: vectorPathSchema, type: z.literal("path") }),
]);
export const shapeFields = {
  fill: paintSchema.nullable(),
  geometry: shapeGeometrySchema,
  stroke: strokeSchema.nullable(),
};
export type ShapeGeometry = z.infer<typeof shapeGeometrySchema>;
