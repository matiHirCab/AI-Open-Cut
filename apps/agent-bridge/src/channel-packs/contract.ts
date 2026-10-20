import { z } from "zod/v4";
import {
  closedSlotRecord,
  componentDefinitionSchema,
  componentInstanceSchema,
  narrationProfileSchema,
  parentReferenceSchema,
  resolvedAssetInputSchema,
  transform2dSchema,
  transformSchema,
  writeResultSchema,
} from "../schemas";

export const CHANNEL_CLI_BYTES = 9 * 1024 * 1024;
export const CHANNEL_PACKAGE_BYTES = 8 * 1024 * 1024;

const closedObject = <T extends z.ZodRawShape>(shape: T) =>
  closedSlotRecord(z.strictObject(shape), Object.keys(shape));

const integer = z.int().nonnegative();
const identity = z.string();
const digest = z.string().regex(/^[0-9a-f]{64}$/);
export const channelProfileRefSchema = closedObject({
  expectedVersion: integer,
  profileId: identity,
});
const captionRole = closedObject({
  recipeKey: identity,
  textSlotId: identity,
});

// Existing persisted-native schemas contain defaults for legacy readers. Check
// their structure without transforming the exact saved portable content.
const exactNative = <T extends z.ZodType>(schema: T) =>
  z
    .unknown()
    .refine((value) => schema.safeParse(value).success)
    .meta(z.toJSONSchema(schema)) as z.ZodType<z.output<T>>;
const nativeDefinition = exactNative(componentDefinitionSchema);

export const channelPackageSchema = closedObject({
  captionRole: captionRole.nullable(),
  componentSchemaVersion: z.literal(46),
  fonts: z.array(
    closedObject({
      dataBase64: z.string(),
      faceIndex: integer,
      sha256: digest,
      sizeBytes: integer,
    })
  ),
  formatVersion: z.literal(1),
  images: z.array(
    closedObject({
      contentHash: resolvedAssetInputSchema.shape.contentHash.unwrap(),
      key: identity,
      probe: resolvedAssetInputSchema.shape.probe.unwrap(),
      sizeBytes: integer,
    })
  ),
  name: z.string(),
  narrationProfile: channelProfileRefSchema.nullable(),
  recipes: z.array(
    closedObject({ definition: nativeDefinition, key: identity })
  ),
});

export const channelPackSchema = closedObject({
  id: identity,
  package: channelPackageSchema,
  version: integer,
});
export const channelListSchema = closedObject({
  packs: z.array(
    closedObject({
      fontCount: integer,
      hasCaptionRole: z.boolean(),
      hasNarrationProfile: z.boolean(),
      id: identity,
      imageCount: integer,
      name: z.string(),
      packageBytes: integer,
      recipeCount: integer,
      version: integer,
    })
  ),
});
export const channelCaptureSchema = closedObject({
  captionRole: captionRole.nullable(),
  componentIds: z.array(identity),
  expectedRevision: integer,
  name: z.string(),
  narrationProfile: channelProfileRefSchema.nullable(),
  projectId: identity,
});
export const channelCaptureResultSchema = closedObject({
  pack: channelPackSchema,
  warnings: z.array(z.string()),
});
const channelApplyShape = {
  application: z.union([
    closedObject({
      mode: z.literal("instantiate"),
      placements: z.array(
        closedObject({
          durationMs: integer,
          hidden: z.boolean(),
          parent: exactNative(parentReferenceSchema).nullable(),
          recipeKey: identity,
          slotValues: exactNative(componentInstanceSchema.shape.slotValues),
          staggerMs: integer,
          startMs: integer,
          timeScale: z.number().finite(),
          trackId: identity,
          transform: exactNative(transformSchema),
          transform2d: exactNative(transform2dSchema).nullable(),
          trimStartMs: integer,
          zIndex: z.int().min(-2_147_483_648).max(2_147_483_647),
        })
      ),
    }),
    closedObject({
      components: z.array(
        closedObject({ componentId: identity, recipeKey: identity })
      ),
      mode: z.literal("reapply"),
    }),
  ]),
  expectedRevision: integer,
  expectedVersion: integer,
  images: z.array(closedObject({ assetId: identity, dependencyKey: identity })),
  narration: z.union([
    closedObject({ mode: z.literal("saved") }),
    closedObject({ mode: z.literal("disabled") }),
    closedObject({
      mode: z.literal("replacement"),
      reference: channelProfileRefSchema,
    }),
  ]),
  packId: identity,
  projectId: identity,
};
export const channelApplyInputSchema = closedObject(channelApplyShape);
export const channelApplyResultSchema = closedObject({
  captionRole: closedObject({
    componentId: identity,
    textSlotId: identity,
  }).nullable(),
  components: z.array(
    closedObject({
      afterDigest: digest,
      beforeDigest: digest.nullable(),
      changed: z.boolean(),
      componentId: identity,
      recipeKey: identity,
    })
  ),
  narrationProfile: narrationProfileSchema.nullable(),
  pack: closedObject({ id: identity, name: z.string(), version: integer }),
  write: writeResultSchema,
});

export const channelRequestSchema = z.union([
  closedObject({ operation: z.literal("channel_pack_list") }),
  closedObject({
    operation: z.literal("channel_pack_get"),
    packId: identity,
  }),
  closedObject({
    capture: channelCaptureSchema,
    operation: z.literal("channel_pack_capture"),
  }),
  closedObject({
    capture: channelCaptureSchema,
    expectedVersion: integer,
    operation: z.literal("channel_pack_update"),
    packId: identity,
  }),
  closedObject({
    name: z.string(),
    operation: z.literal("channel_pack_import"),
    packageJson: z.string(),
  }),
  closedObject({
    ...channelApplyShape,
    operation: z.literal("channel_pack_apply"),
  }),
]);

export type ChannelRequest = z.infer<typeof channelRequestSchema>;
