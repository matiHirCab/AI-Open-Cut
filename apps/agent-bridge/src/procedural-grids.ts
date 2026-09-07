import { z } from "zod/v4";
import { paintSchema, strokeSchema } from "./vector-primitives";

const dimension = z.number().gt(0).max(16_384);
const gridPatternSchema = z.discriminatedUnion("type", [
  z.strictObject({
    spacingX: dimension,
    spacingY: dimension,
    stroke: strokeSchema,
    type: z.literal("rectangular"),
  }),
  z.strictObject({
    spacing: dimension,
    stroke: strokeSchema,
    type: z.literal("diagonal"),
  }),
  z.strictObject({
    paint: paintSchema,
    radius: dimension,
    spacingX: dimension,
    spacingY: dimension,
    type: z.literal("dot"),
  }),
  z.strictObject({
    spacing: dimension,
    stroke: strokeSchema,
    type: z.literal("isometric"),
  }),
]);

export const gridDescriptorSchema = z
  .strictObject({
    height: dimension,
    pattern: gridPatternSchema,
    width: dimension,
  })
  .refine((grid) => {
    const p = grid.pattern;
    if (p.type === "rectangular" || p.type === "dot") {
      const nx = Math.floor(grid.width / p.spacingX) + 1;
      const ny = Math.floor(grid.height / p.spacingY) + 1;
      return p.type === "dot"
        ? p.radius <= Math.min(p.spacingX, p.spacingY) / 2 && nx * ny <= 4096
        : nx + ny <= 4096;
    }
    const normals: [number, number][] =
      p.type === "diagonal"
        ? [
            [Math.SQRT1_2, Math.SQRT1_2],
            [Math.SQRT1_2, -Math.SQRT1_2],
          ]
        : [
            [1, 0],
            [0.5, 0.866_025_403_784_438_6],
            [0.5, -0.866_025_403_784_438_6],
          ];
    let count = 0;
    for (const [nx, ny] of normals) {
      const lo =
        (Math.min(nx * grid.width, 0) + Math.min(ny * grid.height, 0)) /
        p.spacing;
      const hi =
        (Math.max(nx * grid.width, 0) + Math.max(ny * grid.height, 0)) /
        p.spacing;
      const first = ny === 0 ? Math.ceil(lo) : Math.floor(lo) + 1;
      const last = ny === 0 ? Math.floor(hi) : Math.ceil(hi) - 1;
      if (
        !(Number.isFinite(first) && Number.isFinite(last)) ||
        Math.abs(first) > 4096 ||
        Math.abs(last) > 4096
      ) {
        return false;
      }
      count += Math.max(0, last - first + 1);
    }
    return count <= 4096;
  }, "Invalid grid radius or mark budget");

export type GridDescriptor = z.infer<typeof gridDescriptorSchema>;
