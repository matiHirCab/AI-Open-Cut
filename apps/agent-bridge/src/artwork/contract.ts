import { z } from "zod/v4";

export const ARTWORK_LIMITS = {
  manifestBytes: 32_768,
  outlineRadius: 16,
  reviewBytes: 8_388_608,
  sourceBytes: 4_194_304,
  sourcePixels: 262_144,
  sourceSide: 512,
} as const;

const channel = z.int().min(0).max(255);
const hash = z.string().regex(/^[0-9a-f]{64}$/u);
const id = z.string().min(1).max(128);
const side = z.int().min(1).max(ARTWORK_LIMITS.sourceSide);
const offset = z
  .array(z.int().min(0).max(ARTWORK_LIMITS.outlineRadius))
  .length(2);

export const artworkParametersSchema = z.strictObject({
  cleanup: z
    .strictObject({
      color: z.array(channel).length(3),
      tolerance: channel,
    })
    .optional(),
  outline: z.strictObject({
    color: z.array(channel).length(4),
    radius: z.int().min(0).max(ARTWORK_LIMITS.outlineRadius),
  }),
});

export const artworkReviewHashesSchema = z.strictObject({
  dark: hash,
  light: hash,
  mask: hash,
  result: hash,
});

export const artworkDerivationSchema = z.strictObject({
  contentOffset: offset,
  parameters: artworkParametersSchema,
  sourceAssetId: id,
  sourceHash: hash,
  sourceHeight: side,
  sourceWidth: side,
  version: z.literal(1),
});

export const artworkManifestSchema = artworkDerivationSchema.extend({
  expectedRevision: z.int().nonnegative(),
  height: z.int().min(1).max(544),
  projectId: id,
  resultHash: hash,
  reviewHashes: artworkReviewHashesSchema,
  width: z.int().min(1).max(544),
});

const png = z.string().min(1).max(11_184_812);
export const artworkPreviewSchema = z.strictObject({
  files: z.strictObject({ dark: png, light: png, mask: png, result: png }),
  manifest: artworkManifestSchema,
});

export type ArtworkParameters = z.infer<typeof artworkParametersSchema>;
export type ArtworkManifest = z.infer<typeof artworkManifestSchema>;
export type ArtworkDerivation = z.infer<typeof artworkDerivationSchema>;
