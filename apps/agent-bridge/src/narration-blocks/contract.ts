import { z } from "zod/v4";

// Domain checks remain in core. This factory shares the existing resolved speech
// declaration without a runtime import cycle or a second pronunciation resolver.
export const createNarrationModelSchemas = <
  R extends z.ZodType,
  M extends z.ZodType,
>(
  request: R,
  markerPolicy: M
) => {
  const integer = z.int().nonnegative();
  const planned = z.strictObject({
    displayText: z.string(),
    endScalar: integer,
    id: z.string(),
    paragraph: integer,
    request,
    startScalar: integer,
  });
  const planInput = z.strictObject({
    boundaries: z.array(integer).optional(),
    expectedVersion: integer,
    maxTextCharacters: integer,
    profileId: z.string(),
    sourceText: z.string(),
  });
  const plan = z.strictObject({
    blocks: z.array(planned),
    id: z.string(),
    maxTextCharacters: integer,
    sourceText: z.string(),
  });
  const prepared = z.strictObject({
    block: planned,
    spokenSegments: z.array(z.string()),
    spokenText: z.string(),
  });
  const binding = z.strictObject({
    assetId: z.string(),
    captionRefs: z.array(
      z.strictObject({
        durationMs: integer,
        id: z.string(),
        startMs: integer,
        text: z.string(),
      })
    ),
    cueRefs: z.array(
      z.strictObject({ id: z.string(), name: z.string(), timeMs: integer })
    ),
    durationMs: integer,
    itemId: z.string(),
    sourceInMs: integer,
    startMs: integer,
    trackId: z.string(),
  });
  const stored = planned.extend({
    binding,
    spokenSegments: z.array(z.string()),
    spokenText: z.string(),
  });
  const manifest = plan.extend({
    blocks: z.array(stored),
    caption: z
      .strictObject({
        granularity: z.enum(["sentence", "word"]),
        trackId: z.string(),
      })
      .optional(),
    markerPolicy,
  });
  return { binding, manifest, plan, planInput, planned, prepared, stored };
};
