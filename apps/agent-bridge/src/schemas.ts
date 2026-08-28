import { z } from "zod/v4";

const id = z.string().min(1).max(128);
const milliseconds = z.int().nonnegative();
const positiveMilliseconds = z.int().positive();
const finite = z.number().finite();

export const projectRevisionSchema = z
  .object({ expectedRevision: z.int().nonnegative(), projectId: id })
  .strict();

export const timeRangeSchema = z
  .object({ endMs: milliseconds, startMs: milliseconds })
  .strict()
  .refine(
    (range) => range.startMs < range.endMs,
    "startMs must be less than endMs"
  );

export const transformSchema = z
  .object({
    opacity: finite.min(0).max(1),
    positionX: finite,
    positionY: finite,
    scale: finite.positive().max(100),
  })
  .strict();

export const audioSchema = z
  .object({
    fadeInMs: milliseconds,
    fadeOutMs: milliseconds,
    muted: z.boolean(),
    volume: finite.min(0).max(4),
  })
  .strict();

const positionKeyframe = z
  .object({
    easing: z.enum(["hold", "linear", "ease_in", "ease_out", "ease_in_out"]),
    property: z.literal("position"),
    timeMs: milliseconds,
    value: z
      .object({ type: z.literal("position"), x: finite, y: finite })
      .strict(),
  })
  .strict();

const scalarKeyframe = z
  .object({
    easing: z.enum(["hold", "linear", "ease_in", "ease_out", "ease_in_out"]),
    property: z.enum(["scale", "opacity"]),
    timeMs: milliseconds,
    value: z.object({ type: z.literal("scalar"), value: finite }).strict(),
  })
  .strict();

export const keyframeSchema = z.discriminatedUnion("property", [
  positionKeyframe,
  scalarKeyframe,
]);

export const writeResultSchema = z
  .object({
    changedIds: z.array(id),
    projectId: id,
    revision: z.int().nonnegative(),
    summary: z.string(),
    warnings: z.array(z.string()),
  })
  .strict();

export const statusSchema = z
  .object({
    activeProjectId: id.nullable(),
    capabilities: z.array(z.string()),
    ready: z.boolean(),
    subsystems: z
      .object({
        editor: z
          .object({
            capabilities: z.array(z.string()),
            error: z
              .object({
                code: z.string(),
                message: z.string(),
                retryable: z.boolean(),
              })
              .strict()
              .nullable(),
            ready: z.boolean(),
          })
          .strict(),
        rendering: z
          .object({
            capabilities: z.array(z.string()),
            error: z
              .object({
                code: z.string(),
                message: z.string(),
                retryable: z.boolean(),
              })
              .strict()
              .nullable(),
            ready: z.boolean(),
          })
          .strict(),
        speech: z
          .object({
            capabilities: z.array(z.string()),
            error: z
              .object({
                code: z.string(),
                message: z.string(),
                retryable: z.boolean(),
              })
              .strict()
              .nullable(),
            modelId: id.nullable(),
            providerId: id.nullable(),
            queue: z
              .object({
                active: z.int().nonnegative(),
                concurrency: z.literal(1),
                fairness: z.literal("fifo"),
                maxQueued: z.int().positive(),
                queued: z.int().nonnegative(),
              })
              .strict()
              .nullable(),
            ready: z.boolean(),
          })
          .strict(),
        transcription: z
          .object({
            capabilities: z.array(z.string()),
            error: z
              .object({
                code: z.string(),
                message: z.string(),
                retryable: z.boolean(),
              })
              .strict()
              .nullable(),
            modelId: id.nullable(),
            providerId: id.nullable(),
            queue: z
              .object({
                active: z.int().nonnegative(),
                concurrency: z.literal(1),
                fairness: z.literal("fifo"),
                maxQueued: z.int().positive(),
                queued: z.int().nonnegative(),
              })
              .strict()
              .nullable(),
            ready: z.boolean(),
          })
          .strict(),
      })
      .strict(),
    version: z.string(),
  })
  .strict();

export const headlessStatusSchema = statusSchema
  .omit({ activeProjectId: true, subsystems: true })
  .extend({
    subsystems: z
      .object({
        editor: z
          .object({
            capabilities: z.array(z.string()),
            error: z
              .object({
                code: z.string(),
                message: z.string(),
                retryable: z.boolean(),
              })
              .strict()
              .nullable(),
            ready: z.boolean(),
          })
          .strict(),
        rendering: z
          .object({
            capabilities: z.array(z.string()),
            error: z
              .object({
                code: z.string(),
                message: z.string(),
                retryable: z.boolean(),
              })
              .strict()
              .nullable(),
            ready: z.boolean(),
          })
          .strict(),
      })
      .strict(),
  })
  .strict();

export const artifactSchema = z
  .object({
    mimeType: z.string(),
    relativePath: z.string(),
    sizeBytes: z.int().positive(),
  })
  .strict();

export const speechVoiceIdSchema = id;

export const speechVoiceSchema = z
  .object({
    accent: z.string().min(1),
    available: z.boolean(),
    id: speechVoiceIdSchema,
    isDefault: z.boolean(),
    label: z.string().min(1),
    language: z.string().min(1),
    locale: z.string().min(1),
    modelId: id,
    previewSupported: z.boolean(),
    providerId: id,
  })
  .strict();

export const speechTextOptionsSchema = z
  .object({
    chunking: z.enum(["none", "sentence"]).default("sentence"),
    normalization: z.enum(["none", "basic"]).default("basic"),
    pronunciations: z
      .array(
        z
          .object({
            spoken: z.string().trim().min(1).max(256),
            term: z.string().trim().min(1).max(128),
          })
          .strict()
      )
      .max(100)
      .default([])
      .refine(
        (entries) =>
          new Set(entries.map((entry) => entry.term)).size === entries.length,
        "pronunciation terms must be unique"
      ),
    sentencePauseMs: z.int().min(0).max(5000).default(120),
  })
  .strict();

export const speechResourcesSchema = z
  .object({
    execution: z.literal("local"),
    minimumLogicalCpus: z.int().positive(),
    minimumRamBytes: z.int().positive(),
    recommendedLogicalCpus: z.int().positive(),
    recommendedRamBytes: z.int().positive(),
  })
  .strict();

export const speechModelSchema = z
  .object({
    id,
    sampleRateHz: z.int().positive(),
    version: z.string().min(1).nullable(),
  })
  .strict();

export const speechLimitsSchema = z
  .object({
    maxSpeed: finite.positive(),
    maxTextCharacters: z.int().positive(),
    minSpeed: finite.positive(),
  })
  .strict()
  .refine((limits) => limits.minSpeed <= limits.maxSpeed);

export const ttsStatusSchema = z
  .object({
    defaultLanguage: z.string().min(1),
    defaultSpeed: finite.positive(),
    defaultVoiceId: speechVoiceIdSchema,
    device: z.string().min(1),
    devices: z.array(z.string().min(1)).min(1),
    languages: z.array(z.string().min(1)).min(1),
    limits: speechLimitsSchema,
    modelCached: z.boolean(),
    modelId: id,
    modelLoaded: z.boolean(),
    models: z.array(speechModelSchema).min(1),
    modelVersion: z.string().min(1).nullable(),
    providerId: id,
    queue: z
      .object({
        active: z.int().nonnegative(),
        concurrency: z.literal(1),
        fairness: z.literal("fifo"),
        maxQueued: z.int().positive(),
        queued: z.int().nonnegative(),
      })
      .strict()
      .optional(),
    ready: z.boolean(),
    resources: speechResourcesSchema,
    sampleRateHz: z.int().positive(),
    version: z.string(),
    voices: z.array(speechVoiceIdSchema).min(1),
  })
  .strict();

export const speechVoiceListSchema = z.array(speechVoiceSchema);
export const speechVoiceListResultSchema = z
  .object({ voices: speechVoiceListSchema })
  .strict();

export const synthesizedSpeechMetadataSchema = z
  .object({
    durationMs: positiveMilliseconds,
    language: z.string().min(1),
    modelId: id,
    modelVersion: z.string().min(1).nullable(),
    providerId: id,
    sampleRateHz: z.int().positive(),
    voiceId: speechVoiceIdSchema,
  })
  .strict();

export const ttsResultSchema = z
  .object({
    assetId: id,
    durationMs: positiveMilliseconds,
    itemId: id,
    language: z.string().min(1),
    modelId: id,
    modelVersion: z.string().min(1).nullable(),
    providerId: id,
    revision: z.int().nonnegative(),
    voice: speechVoiceIdSchema,
    warnings: z.array(z.string()),
  })
  .strict();

export const generatedAssetOriginSchema = z.discriminatedUnion("type", [
  z
    .object({
      generation: z
        .object({
          generatedAtMs: milliseconds,
          modelId: z.string(),
          modelVersion: z.string().nullable(),
          providerId: z.string(),
          request: z
            .object({
              language: z.string(),
              speed: finite.positive(),
              text: z.string(),
              textOptions: speechTextOptionsSchema,
              voiceId: z.string(),
            })
            .strict(),
          sampleRateHz: z.int().positive(),
        })
        .strict(),
      type: z.literal("speech_synthesis"),
    })
    .strict(),
]);

export const commitGeneratedAssetResultSchema = z
  .object({
    assetId: id,
    itemId: id,
    projectId: id,
    revision: z.int().nonnegative(),
    summary: z.string(),
    warnings: z.array(z.string()),
  })
  .strict();

export const replaceGeneratedAssetResultSchema = z
  .object({
    assetId: id,
    itemId: id,
    projectId: id,
    replacedAssetId: id,
    revision: z.int().nonnegative(),
    summary: z.string(),
    warnings: z.array(z.string()),
  })
  .strict();

export const speechPreviewResultSchema = z
  .object({
    durationMs: positiveMilliseconds,
    expiresAtMs: milliseconds,
    language: z.string().min(1),
    modelId: id,
    modelVersion: z.string().min(1).nullable(),
    providerId: id,
    sampleRateHz: z.int().positive(),
    token: id,
    voice: speechVoiceIdSchema,
  })
  .strict();

export const speechRegenerateResultSchema = ttsResultSchema.extend({
  replacedAssetId: id,
});

export const speechEstimateSchema = z
  .object({
    characters: z.int().nonnegative(),
    chunks: z.int().positive(),
    cost: z
      .object({
        amount: z.literal(0),
        billing: z.literal("local"),
        currency: z.null(),
      })
      .strict(),
    estimatedDurationMs: z
      .object({
        expected: positiveMilliseconds,
        maximum: positiveMilliseconds,
        minimum: positiveMilliseconds,
      })
      .strict(),
    language: z.string().min(1),
    modelCached: z.boolean(),
    modelId: id,
    modelLoaded: z.boolean(),
    providerId: id,
    queue: z
      .object({
        active: z.int().nonnegative(),
        concurrency: z.literal(1),
        fairness: z.literal("fifo"),
        maxQueued: z.int().positive(),
        queued: z.int().nonnegative(),
      })
      .strict(),
    resources: speechResourcesSchema,
    voice: speechVoiceIdSchema,
  })
  .strict();

export const transcriptionWordSchema = z
  .object({
    confidence: finite.min(0).max(1).optional(),
    endMs: milliseconds,
    startMs: milliseconds,
    word: z.string().min(1),
  })
  .strict();

export const transcriptionSegmentSchema = z
  .object({
    confidence: finite.min(0).max(1).optional(),
    endMs: positiveMilliseconds,
    startMs: milliseconds,
    text: z.string().trim().min(1).max(4096),
    words: z.array(transcriptionWordSchema).optional(),
  })
  .strict()
  .refine((segment) => segment.startMs < segment.endMs);

export const transcriptionStatusSchema = z
  .object({
    computeType: z.literal("int8"),
    device: z.literal("cpu"),
    limits: z.object({ maxDurationMs: positiveMilliseconds }).strict(),
    modelCached: z.boolean(),
    modelId: id,
    modelLoaded: z.boolean(),
    modelVersion: z.string().min(1).nullable(),
    providerId: id,
    queue: z
      .object({
        active: z.int().nonnegative(),
        concurrency: z.literal(1),
        fairness: z.literal("fifo"),
        maxQueued: z.int().positive(),
        queued: z.int().nonnegative(),
      })
      .strict(),
    ready: z.boolean(),
    version: z.string().min(1),
  })
  .strict();

export const resolvedAssetInputSchema = z
  .object({
    assetId: id,
    contentHash: z
      .object({
        algorithm: z.literal("sha256"),
        digest: z.string().regex(/^[0-9a-f]{64}$/),
      })
      .strict()
      .nullable(),
    path: z.string().min(1),
    probe: z
      .object({
        audioChannels: z.int().positive().nullable(),
        audioCodec: z.string().nullable(),
        audioSampleRateHz: z.int().positive().nullable(),
        durationMs: milliseconds.nullable(),
        formatName: z.string().nullable(),
        hasAudio: z.boolean(),
        hasVideo: z.boolean(),
        videoCodec: z.string().nullable(),
        videoHeight: z.int().positive().nullable(),
        videoWidth: z.int().positive().nullable(),
      })
      .strict()
      .nullable(),
    projectId: id,
    revision: z.int().nonnegative(),
  })
  .strict();

export const transcriptionPreviewResultSchema = z
  .object({
    assetId: id,
    baseRevision: z.int().nonnegative(),
    durationMs: positiveMilliseconds,
    expiresAtMs: milliseconds,
    language: z.string().min(1),
    modelId: id,
    modelVersion: z.string().min(1).nullable(),
    projectId: id,
    providerId: id,
    segments: z.array(transcriptionSegmentSchema).max(10_000),
    token: id,
  })
  .strict();

export const transcriptionEstimateSchema = z
  .object({
    cost: z
      .object({
        amount: z.literal(0),
        billing: z.literal("local"),
        currency: z.null(),
      })
      .strict(),
    durationMs: positiveMilliseconds,
    language: z.string().min(1).nullable(),
    modelCached: z.boolean(),
    modelId: id,
    providerId: id,
    queue: transcriptionStatusSchema.shape.queue,
  })
  .strict();

const mediaItemSchema = z
  .object({
    assetId: id,
    audio: audioSchema,
    durationMs: positiveMilliseconds,
    hidden: z.boolean(),
    id,
    keyframes: z.array(keyframeSchema),
    sourceInMs: milliseconds,
    startMs: milliseconds,
    transform: transformSchema,
    type: z.literal("media"),
  })
  .strict();

const textItemSchema = z
  .object({
    color: z.string(),
    durationMs: positiveMilliseconds,
    fontFamily: z.string().nullable(),
    fontSize: z.int().positive(),
    hidden: z.boolean(),
    id,
    keyframes: z.array(keyframeSchema),
    startMs: milliseconds,
    text: z.string(),
    transform: transformSchema,
    type: z.literal("text"),
  })
  .strict();

const captionWordSchema = z
  .object({
    confidence: finite.min(0).max(1).nullable(),
    endMs: milliseconds,
    startMs: milliseconds,
    word: z.string(),
  })
  .strict();

const captionItemSchema = z
  .object({
    durationMs: positiveMilliseconds,
    hidden: z.boolean(),
    id,
    source: z
      .object({
        assetId: id,
        confidence: finite.min(0).max(1).nullable(),
        generatedAtMs: positiveMilliseconds,
        language: z.string().min(1),
        modelId: id,
        modelVersion: z.string().min(1).nullable(),
        originalText: z.string(),
        providerId: id,
        words: z.array(captionWordSchema),
      })
      .strict(),
    startMs: milliseconds,
    style: z
      .object({
        backgroundColor: z.string().regex(/^#[0-9a-fA-F]{6}$/),
        bottomMarginPx: z.int().nonnegative(),
        color: z.string().regex(/^#[0-9a-fA-F]{6}$/),
        fontSize: z.int().positive().max(1000),
      })
      .strict(),
    text: z.string(),
    type: z.literal("caption"),
  })
  .strict();

const transitionItemSchema = z
  .object({
    durationMs: positiveMilliseconds,
    fromItemId: id,
    hidden: z.boolean(),
    id,
    startMs: milliseconds,
    toItemId: id.nullable(),
    transitionType: z.enum(["fade", "crossfade"]),
    type: z.literal("transition"),
  })
  .strict();

export const timelineItemSchema = z.discriminatedUnion("type", [
  mediaItemSchema,
  textItemSchema,
  captionItemSchema,
  transitionItemSchema,
]);

export const projectSummarySchema = z
  .object({
    durationMs: milliseconds,
    id,
    name: z.string(),
    revision: z.int().nonnegative(),
    settings: z
      .object({
        fps: z.int().positive(),
        height: z.int().positive(),
        width: z.int().positive(),
      })
      .strict(),
    updatedAtMs: milliseconds,
  })
  .strict();

export const projectListSchema = z.array(projectSummarySchema);

export const projectStateSchema = z
  .object({
    durationMs: milliseconds,
    project: z
      .object({
        assets: z.array(
          z
            .object({
              contentHash: z
                .object({
                  algorithm: z.literal("sha256"),
                  digest: z.string().regex(/^[0-9a-f]{64}$/),
                })
                .strict(),
              durationMs: milliseconds.nullable(),
              fileName: z.string(),
              hasAudio: z.boolean(),
              id,
              mediaType: z.enum(["image", "video", "audio"]),
              origin: generatedAssetOriginSchema.nullable(),
              probe: z
                .object({
                  audioChannels: z.int().positive().nullable(),
                  audioCodec: z.string().nullable(),
                  audioSampleRateHz: z.int().positive().nullable(),
                  durationMs: milliseconds.nullable(),
                  formatName: z.string().nullable(),
                  hasAudio: z.boolean(),
                  hasVideo: z.boolean(),
                  videoCodec: z.string().nullable(),
                  videoHeight: z.int().positive().nullable(),
                  videoWidth: z.int().positive().nullable(),
                })
                .strict(),
              projectRelativePath: z.string(),
              sizeBytes: z.int().nonnegative(),
            })
            .strict()
        ),
        createdAtMs: milliseconds,
        id,
        name: z.string(),
        revision: z.int().nonnegative(),
        schemaVersion: z.int().positive(),
        settings: z
          .object({
            fps: z.int().positive(),
            height: z.int().positive(),
            width: z.int().positive(),
          })
          .strict(),
        tracks: z.array(
          z
            .object({
              hidden: z.boolean(),
              id,
              items: z.array(timelineItemSchema),
              locked: z.boolean(),
              muted: z.boolean(),
              name: z.string(),
              trackType: z.enum(["video", "overlay", "audio", "caption"]),
            })
            .strict()
        ),
        updatedAtMs: milliseconds,
      })
      .strict(),
  })
  .strict();

const speechOverridesSchema = z
  .object({
    language: z.string().min(1).optional(),
    speed: finite.positive().optional(),
    text: z.string().trim().min(1).max(5000).optional(),
    textOptions: speechTextOptionsSchema.optional(),
    voice: speechVoiceIdSchema.optional(),
  })
  .strict();

export const speechSourceSchema = z.discriminatedUnion("type", [
  z
    .object({
      language: z.string().min(1).optional(),
      speed: finite.positive().optional(),
      text: z.string().trim().min(1).max(5000),
      textOptions: speechTextOptionsSchema.optional(),
      type: z.literal("request"),
      voice: speechVoiceIdSchema.optional(),
    })
    .strict(),
  speechOverridesSchema
    .extend({ itemId: id, projectId: id, type: z.literal("item") })
    .strict(),
]);

export const jobSchema = z
  .object({
    artifact: artifactSchema.optional(),
    createdAtMs: milliseconds,
    error: z
      .object({ code: z.string(), message: z.string(), retryable: z.boolean() })
      .strict()
      .optional(),
    expiresAtMs: milliseconds.nullable(),
    generatedArtifact: z
      .object({ expiresAtMs: milliseconds, token: id })
      .strict()
      .optional(),
    jobId: id,
    kind: z.enum([
      "preview",
      "export",
      "tts",
      "speech_preview",
      "speech_regenerate",
      "transcription_preview",
    ]),
    persistence: z.literal("process"),
    progress: finite.min(0).max(1),
    projectId: id,
    result: z.union([ttsResultSchema, speechRegenerateResultSchema]).optional(),
    revision: z.int().nonnegative(),
    speechPreview: speechPreviewResultSchema.optional(),
    status: z.enum(["queued", "running", "completed", "failed", "cancelled"]),
    transcriptionPreview: transcriptionPreviewResultSchema.optional(),
    updatedAtMs: milliseconds,
  })
  .strict();

export const headlessEditSchema = z.discriminatedUnion("operation", [
  z
    .object({
      assetId: id,
      durationMs: positiveMilliseconds,
      operation: z.literal("add_media"),
      sourceInMs: milliseconds,
      startMs: milliseconds,
      trackId: id,
    })
    .strict(),
  z
    .object({
      color: z.string().regex(/^#[0-9a-fA-F]{6}$/),
      durationMs: positiveMilliseconds,
      fontFamily: z.string().min(1).max(200).optional(),
      fontSize: z.int().min(1).max(1000),
      operation: z.literal("add_text"),
      startMs: milliseconds,
      text: z.string().min(1).max(4096),
      trackId: id,
      transform: transformSchema,
    })
    .strict(),
  z
    .object({
      itemId: id,
      operation: z.literal("update_item"),
      text: z.string().min(1).max(4096).optional(),
      transform: transformSchema.optional(),
    })
    .strict(),
  z
    .object({
      itemId: id,
      operation: z.literal("move_item"),
      startMs: milliseconds,
      trackId: id,
    })
    .strict(),
  z
    .object({
      durationMs: positiveMilliseconds,
      itemId: id,
      operation: z.literal("trim_item"),
      sourceInMs: milliseconds.optional(),
      startMs: milliseconds,
    })
    .strict(),
  z.object({ itemId: id, operation: z.literal("delete_item") }).strict(),
  z
    .object({
      itemId: id,
      keyframes: z.array(keyframeSchema).max(1000),
      operation: z.literal("set_keyframes"),
    })
    .strict(),
  z
    .object({
      durationMs: positiveMilliseconds,
      fromItemId: id,
      operation: z.literal("add_transition"),
      startMs: milliseconds,
      toItemId: id.optional(),
      trackId: id,
      transitionType: z.enum(["fade", "crossfade"]),
    })
    .strict(),
  z
    .object({
      audio: audioSchema,
      itemId: id,
      operation: z.literal("set_audio"),
    })
    .strict(),
  z
    .object({
      itemId: id,
      operation: z.literal("split_item"),
      splitMs: milliseconds,
    })
    .strict(),
  z
    .object({
      itemIds: z.array(id).min(1).max(100),
      offsetMs: milliseconds,
      operation: z.literal("duplicate_items"),
    })
    .strict(),
  z
    .object({
      index: z.int().nonnegative().optional(),
      name: z.string().trim().min(1).max(128),
      operation: z.literal("create_track"),
      trackType: z.enum(["video", "overlay", "audio", "caption"]),
    })
    .strict(),
  z
    .object({
      hidden: z.boolean().optional(),
      index: z.int().nonnegative().optional(),
      locked: z.boolean().optional(),
      muted: z.boolean().optional(),
      name: z.string().trim().min(1).max(128).optional(),
      operation: z.literal("update_track"),
      trackId: id,
    })
    .strict(),
  z.object({ operation: z.literal("delete_track"), trackId: id }).strict(),
  z
    .object({
      hidden: z.boolean(),
      itemId: id,
      operation: z.literal("set_item_visibility"),
    })
    .strict(),
]);

export const editDraftSchema = z
  .object({
    baseRevision: z.int().nonnegative(),
    createdAtMs: milliseconds,
    id,
    label: z.string().nullable(),
    operations: z.array(headlessEditSchema).min(1).max(100),
    projectId: id,
    updatedAtMs: milliseconds,
    version: z.literal(1),
  })
  .strict();

export const schemas = {
  assetDelete: projectRevisionSchema.extend({ assetId: id }).strict(),
  assetImport: projectRevisionSchema
    .extend({
      mediaType: z.enum(["image", "video", "audio"]),
      path: z.string().min(1),
    })
    .strict(),
  draftCommit: projectRevisionSchema.extend({ draftId: id }).strict(),
  draftCreate: projectRevisionSchema
    .extend({
      label: z.string().trim().min(1).max(200).optional(),
      operations: z.array(headlessEditSchema).min(1).max(100),
    })
    .strict(),
  draftDiscard: z.object({ draftId: id, projectId: id }).strict(),
  draftGet: z.object({ draftId: id, projectId: id }).strict(),
  draftPreviewFrame: z
    .object({ draftId: id, projectId: id, timeMs: milliseconds })
    .strict(),
  draftRebase: projectRevisionSchema.extend({ draftId: id }).strict(),
  draftUpdate: projectRevisionSchema
    .extend({
      draftId: id,
      label: z.string().trim().min(1).max(200).optional(),
      operations: z.array(headlessEditSchema).min(1).max(100),
    })
    .strict(),
  editorGetStatus: z.object({}).strict(),
  jobCancel: z.object({ jobId: id }).strict(),
  jobGetStatus: z.object({ jobId: id }).strict(),
  previewRenderFrame: projectRevisionSchema
    .extend({ timeMs: milliseconds })
    .strict(),
  projectCreate: z
    .object({
      fps: z.int().min(1).max(120).default(30),
      height: z.int().positive().max(4320).default(1080),
      name: z.string().trim().min(1).max(200),
      width: z.int().positive().max(7680).default(1920),
    })
    .strict(),
  projectExportVideo: projectRevisionSchema
    .extend({
      format: z.literal("mp4"),
      overwrite: z.boolean().default(false),
      relativePath: z
        .string()
        .min(1)
        .regex(/\.mp4$/i),
      resolution: z.enum(["project", "1080p", "720p"]),
    })
    .strict(),
  projectGetState: z
    .object({ projectId: id, timeRange: timeRangeSchema.optional() })
    .strict(),
  projectOpen: z.object({ projectId: id }).strict(),
  projectRedo: projectRevisionSchema,
  projectUndo: projectRevisionSchema,
  speechCommitPreview: projectRevisionSchema
    .extend({
      placement: z.discriminatedUnion("type", [
        z
          .object({
            startMs: milliseconds,
            trackId: id,
            type: z.literal("insert"),
          })
          .strict(),
        z.object({ itemId: id, type: z.literal("replace") }).strict(),
      ]),
      token: id,
    })
    .strict(),
  speechDiscardPreview: z.object({ token: id }).strict(),
  speechEstimate: z.object({ source: speechSourceSchema }).strict(),
  speechGenerateAndInsert: projectRevisionSchema
    .extend({
      language: z.string().min(1).optional(),
      speed: finite.positive().optional(),
      startMs: milliseconds,
      text: z.string().trim().min(1).max(5000),
      textOptions: speechTextOptionsSchema.optional(),
      trackId: id,
      voice: speechVoiceIdSchema.optional(),
    })
    .strict(),
  speechListVoices: z
    .object({ language: z.string().min(1).optional() })
    .strict(),
  speechPreview: z.object({ source: speechSourceSchema }).strict(),
  speechRegenerate: projectRevisionSchema
    .extend({
      itemId: id,
      language: z.string().min(1).optional(),
      speed: finite.positive().optional(),
      text: z.string().trim().min(1).max(5000).optional(),
      textOptions: speechTextOptionsSchema.optional(),
      voice: speechVoiceIdSchema.optional(),
    })
    .strict(),
  timelineAddMedia: projectRevisionSchema
    .extend({
      assetId: id,
      durationMs: positiveMilliseconds,
      sourceInMs: milliseconds.default(0),
      startMs: milliseconds,
      trackId: id,
    })
    .strict(),
  timelineAddText: projectRevisionSchema
    .extend({
      color: z
        .string()
        .regex(/^#[0-9a-fA-F]{6}$/)
        .default("#ffffff"),
      durationMs: positiveMilliseconds,
      fontFamily: z.string().min(1).max(200).optional(),
      fontSize: z.int().min(1).max(1000).default(64),
      startMs: milliseconds,
      text: z.string().min(1).max(4096),
      trackId: id,
      transform: transformSchema.default({
        opacity: 1,
        positionX: 0,
        positionY: 0,
        scale: 1,
      }),
    })
    .strict(),
  timelineAddTransition: projectRevisionSchema
    .extend({
      durationMs: positiveMilliseconds,
      fromItemId: id,
      startMs: milliseconds,
      toItemId: id.optional(),
      trackId: id,
      transitionType: z.enum(["fade", "crossfade"]),
    })
    .strict(),
  timelineBatchEdit: projectRevisionSchema
    .extend({ operations: z.array(headlessEditSchema).min(1).max(100) })
    .strict(),
  timelineDeleteItem: projectRevisionSchema.extend({ itemId: id }).strict(),
  timelineDuplicateItems: projectRevisionSchema
    .extend({
      itemIds: z.array(id).min(1).max(100),
      offsetMs: milliseconds,
    })
    .strict(),
  timelineGetItems: z
    .object({
      itemType: z.enum(["media", "text", "caption", "transition"]).optional(),
      projectId: id,
      timeRange: timeRangeSchema.optional(),
      trackId: id.optional(),
    })
    .strict(),
  timelineMoveItem: projectRevisionSchema
    .extend({ itemId: id, startMs: milliseconds, trackId: id })
    .strict(),
  timelineSetAudio: projectRevisionSchema
    .extend({ audio: audioSchema, itemId: id })
    .strict(),
  timelineSetItemVisibility: projectRevisionSchema
    .extend({ hidden: z.boolean(), itemId: id })
    .strict(),
  timelineSetKeyframes: projectRevisionSchema
    .extend({ itemId: id, keyframes: z.array(keyframeSchema).max(1000) })
    .strict(),
  timelineSplitItem: projectRevisionSchema
    .extend({ itemId: id, splitMs: milliseconds })
    .strict(),
  timelineTrimItem: projectRevisionSchema
    .extend({
      durationMs: positiveMilliseconds,
      itemId: id,
      sourceInMs: milliseconds.optional(),
      startMs: milliseconds,
    })
    .strict(),
  timelineUpdateItem: projectRevisionSchema
    .extend({
      itemId: id,
      text: z.string().min(1).max(4096).optional(),
      transform: transformSchema.optional(),
    })
    .strict()
    .refine(
      (value) => value.transform !== undefined || value.text !== undefined,
      "provide transform or text"
    ),
  trackCreate: projectRevisionSchema
    .extend({
      index: z.int().nonnegative().optional(),
      name: z.string().trim().min(1).max(128),
      trackType: z.enum(["video", "overlay", "audio", "caption"]),
    })
    .strict(),
  trackDelete: projectRevisionSchema.extend({ trackId: id }).strict(),
  trackUpdate: projectRevisionSchema
    .extend({
      hidden: z.boolean().optional(),
      index: z.int().nonnegative().optional(),
      locked: z.boolean().optional(),
      muted: z.boolean().optional(),
      name: z.string().trim().min(1).max(128).optional(),
      trackId: id,
    })
    .strict()
    .refine(
      (value) =>
        value.hidden !== undefined ||
        value.index !== undefined ||
        value.locked !== undefined ||
        value.muted !== undefined ||
        value.name !== undefined,
      "provide at least one track update"
    ),
  transcriptionCommitPreview: projectRevisionSchema
    .extend({
      captionTrackId: id.optional(),
      style: z
        .object({
          backgroundColor: z.string().regex(/^#[0-9a-fA-F]{6}$/),
          bottomMarginPx: z.int().nonnegative().max(4320),
          color: z.string().regex(/^#[0-9a-fA-F]{6}$/),
          fontSize: z.int().min(1).max(1000),
        })
        .strict()
        .optional(),
      token: id,
    })
    .strict(),
  transcriptionDiscardPreview: z.object({ token: id }).strict(),
  transcriptionEstimate: z
    .object({
      assetId: id,
      language: z.string().min(1).optional(),
      projectId: id,
    })
    .strict(),
  transcriptionGetStatus: z.object({}).strict(),
  transcriptionPreview: z
    .object({
      assetId: id,
      language: z.string().min(1).optional(),
      projectId: id,
    })
    .strict(),
  ttsCommitGeneratedArtifact: z
    .object({ artifactToken: id, expectedRevision: z.int().nonnegative() })
    .strict(),
  ttsGenerateAndInsert: projectRevisionSchema
    .extend({
      language: z.string().min(1).optional(),
      speed: finite.positive().optional(),
      startMs: milliseconds,
      text: z.string().trim().min(1),
      textOptions: speechTextOptionsSchema.optional(),
      trackId: id,
      voice: speechVoiceIdSchema.optional(),
    })
    .strict(),
  ttsGetStatus: z.object({}).strict(),
} as const;

export type Job = z.infer<typeof jobSchema>;
