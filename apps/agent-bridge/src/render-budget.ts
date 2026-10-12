import { z } from "zod/v4";

const DECIMAL = /^(0|[1-9][0-9]{0,19})$/;

const decimal = z
  .string()
  .regex(DECIMAL)
  .refine(
    (value) =>
      DECIMAL.test(value) && BigInt(value) <= 18_446_744_073_709_551_615n
  );

export const RENDER_BUDGETS = {
  candidate_analysis_nodes: ["nodes", "charged"],
  composition_bytes: ["bytes", "conservative"],
  composition_pixel_work: ["pixel_samples", "charged"],
  effect_pixel_passes: ["pixel_passes", "conservative"],
  motion_blur_pixel_work: ["pixel_samples", "charged"],
  scene_shape_raster_bytes: ["bytes", "conservative"],
  scene_shape_segments: ["segments", "charged"],
  shape_segments: ["segments", "charged"],
} as const;

export const renderBudgetSchema = z
  .strictObject({
    budget: z.enum([
      "shape_segments",
      "scene_shape_segments",
      "scene_shape_raster_bytes",
      "candidate_analysis_nodes",
      "effect_pixel_passes",
      "motion_blur_pixel_work",
      "composition_pixel_work",
      "composition_bytes",
    ]),
    contributors: z
      .array(
        z.strictObject({
          kind: z.enum(["effect", "grid", "motion_blur", "shape", "svg"]),
          work: decimal,
        })
      )
      .max(8),
    limit: decimal,
    measurement: z.enum(["charged", "conservative"]),
    overflow: z.boolean(),
    units: z.enum([
      "segments",
      "bytes",
      "nodes",
      "pixel_passes",
      "pixel_samples",
    ]),
    work: decimal.nullable(),
  })
  .refine(
    (value) =>
      value.overflow === (value.work === null) &&
      value.units === RENDER_BUDGETS[value.budget][0] &&
      value.measurement === RENDER_BUDGETS[value.budget][1],
    "Inconsistent native budget diagnostic"
  );

export type RenderBudgetDiagnostic = z.infer<typeof renderBudgetSchema>;
