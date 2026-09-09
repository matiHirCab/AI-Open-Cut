import { z } from "zod/v4";

const finite = z.number().finite();
const positionSchema = z
  .strictObject({
    unit: z.enum(["pixels", "normalized"]),
    x: finite,
    y: finite,
  })
  .refine((position) => {
    const limit = position.unit === "pixels" ? 1_000_000 : 100;
    return Math.abs(position.x) <= limit && Math.abs(position.y) <= limit;
  }, "Repeater position exceeds its unit bounds");

export const repeaterTransformOffsetSchema = z.strictObject({
  position: positionSchema,
  rotationDeg: finite.min(-36_000).max(36_000),
  scaleX: finite.gt(0).max(100),
  scaleY: finite.gt(0).max(100),
  skewXDeg: finite.min(-80).max(80),
  skewYDeg: finite.min(-80).max(80),
});

export const repeaterDescriptorSchema = z.strictObject({
  copies: z.int().min(1).max(256),
  opacityOffset: finite.min(-1).max(1),
  source: z.strictObject({
    id: z
      .string()
      .min(1)
      .max(128)
      .regex(/^[A-Za-z0-9_-]+$/),
    scope: z.string().min(1).max(256),
  }),
  transformOffset: repeaterTransformOffsetSchema,
});

export const repeaterEditDescriptorSchema = repeaterDescriptorSchema.extend({
  source: z.strictObject({
    id: z
      .string()
      .min(1)
      .max(128)
      .regex(/^(?:[A-Za-z0-9_-]+|@[A-Za-z][A-Za-z0-9_-]{0,63})$/),
    scope: z.string().min(1).max(256),
  }),
});

export type RepeaterDescriptor = z.infer<typeof repeaterDescriptorSchema>;
