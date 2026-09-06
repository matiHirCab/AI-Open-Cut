import { z } from "zod/v4";

// Core owns runtime semantics. These reusable schemas mirror its version-1 contract.
const coordinate = z.number().min(-1_000_000).max(1_000_000);
const positiveCoordinate = z.number().gt(0).max(1_000_000);
export const vectorColorSchema = z.strictObject({
  a: z.number().min(0).max(1),
  b: z.number().min(0).max(1),
  g: z.number().min(0).max(1),
  r: z.number().min(0).max(1),
});
export const vectorPointSchema = z.strictObject({
  x: coordinate,
  y: coordinate,
});
export const gradientStopSchema = z.strictObject({
  color: vectorColorSchema,
  offset: z.number().min(0).max(1),
});
const stops = z
  .array(gradientStopSchema)
  .min(2)
  .max(64)
  .refine(
    (values) =>
      values[0]?.offset === 0 &&
      values.at(-1)?.offset === 1 &&
      values.every(
        (value, index) =>
          index === 0 || value.offset > (values[index - 1]?.offset ?? 1)
      ),
    "Gradient stops must increase strictly from 0 to 1"
  );
export const paintSchema = z.discriminatedUnion("type", [
  z.strictObject({ color: vectorColorSchema, type: z.literal("solid") }),
  z
    .strictObject({
      end: vectorPointSchema,
      start: vectorPointSchema,
      stops,
      type: z.literal("linearGradient"),
    })
    .refine(
      (value) => value.start.x !== value.end.x || value.start.y !== value.end.y,
      "Linear gradient endpoints must differ"
    ),
  z.strictObject({
    center: vectorPointSchema,
    radius: positiveCoordinate,
    stops,
    type: z.literal("radialGradient"),
  }),
]);
export const strokeSchema = z.strictObject({
  dash: z
    .array(positiveCoordinate)
    .max(64)
    .refine(
      (values) => values.length % 2 === 0,
      "Dash entries must have even length"
    ),
  dashOffset: coordinate,
  lineCap: z.enum(["butt", "round", "square"]),
  lineJoin: z.enum(["miter", "round", "bevel"]),
  miterLimit: z.number().min(1).max(1000),
  paint: paintSchema,
  width: z.number().gt(0).max(16_384),
});
const radius = z.number().min(0).max(16_384);
export const cornerRadiiSchema = z.strictObject({
  bottomLeft: radius,
  bottomRight: radius,
  topLeft: radius,
  topRight: radius,
});
export const pathCommandSchema = z.discriminatedUnion("type", [
  z.strictObject({ to: vectorPointSchema, type: z.literal("moveTo") }),
  z.strictObject({ to: vectorPointSchema, type: z.literal("lineTo") }),
  z.strictObject({
    control: vectorPointSchema,
    to: vectorPointSchema,
    type: z.literal("quadraticTo"),
  }),
  z.strictObject({
    control1: vectorPointSchema,
    control2: vectorPointSchema,
    to: vectorPointSchema,
    type: z.literal("cubicTo"),
  }),
  z.strictObject({ type: z.literal("close") }),
]);
export const vectorPathSchema = z
  .strictObject({
    commands: z.array(pathCommandSchema).min(1).max(4096),
    fillRule: z.enum(["nonzero", "evenodd"]),
  })
  .refine((value) => {
    if (value.commands.length > 4096) {
      return false;
    }
    let open = false;
    let drawn = false;
    for (const command of value.commands) {
      if (command.type === "moveTo") {
        open = true;
        drawn = false;
      } else if (command.type === "close") {
        if (!(open && drawn)) {
          return false;
        }
        open = false;
        drawn = false;
      } else {
        if (!open) {
          return false;
        }
        drawn = true;
      }
    }
    return true;
  }, "Invalid vector subpath grammar");

export type VectorColor = z.infer<typeof vectorColorSchema>;
export type VectorPoint = z.infer<typeof vectorPointSchema>;
export type GradientStop = z.infer<typeof gradientStopSchema>;
export type Paint = z.infer<typeof paintSchema>;
export type Stroke = z.infer<typeof strokeSchema>;
export type CornerRadii = z.infer<typeof cornerRadiiSchema>;
export type PathCommand = z.infer<typeof pathCommandSchema>;
export type VectorPath = z.infer<typeof vectorPathSchema>;
