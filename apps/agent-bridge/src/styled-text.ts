import { z } from "zod/v4";

const color = z.string().regex(/^#[0-9a-fA-F]{6}$/);
const opacity = z.number().min(0).max(1);
export const textPaintLayerSchema = z.discriminatedUnion("kind", [
  z.object({ color, kind: z.literal("fill"), opacity }).strict(),
  z
    .object({
      color,
      kind: z.literal("stroke"),
      opacity,
      widthPx: z.number().positive().max(200),
    })
    .strict(),
  z
    .object({
      blurSigmaPx: z.number().min(0).max(64),
      color,
      kind: z.literal("shadow"),
      offsetXPx: z.number().min(-4096).max(4096),
      offsetYPx: z.number().min(-4096).max(4096),
      opacity,
    })
    .strict(),
]);
export const textPaintLayersSchema = z.array(textPaintLayerSchema).max(16);
export const textSpanSchema = z
  .object({
    end: z.int().nonnegative().max(0xff_ff_ff_ff),
    start: z.int().nonnegative().max(0xff_ff_ff_ff),
    style: z
      .object({
        bold: z.boolean().optional(),
        color: color.optional(),
        italic: z.boolean().optional(),
        paintLayers: textPaintLayersSchema.optional(),
      })
      .strict(),
  })
  .strict();
