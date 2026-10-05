import type { z } from "zod/v4";
import type { GridDescriptor } from "./procedural-grids";
import type { RepeaterDescriptor } from "./repeaters";
import type {
  addComponentInstanceSchema,
  addGridSchema,
  addRepeaterSchema,
  addShapeSchema,
  addSvgSchema,
  animationChannelSchema,
  animationPresetParametersSchema,
  componentFieldsSchema,
  componentInstanceDuplicateSchema,
  componentInstanceUpdateSchema,
  generatedAssetOriginSchema,
  maskSchema,
  mediaCropSchema,
  motionBlurSchema,
  richTextDocumentSchema,
  templateSlotSchema,
  textStyleSchema,
  timeExpressionSchema,
  transform2dSchema,
  visualEffectSchema,
} from "./schemas";
import type { ShapeGeometry } from "./shape-items";
import type { Paint, Stroke } from "./vector-primitives";

export const EVALUATED_SCENE_RENDERING_CAPABILITY =
  "evaluated_scene_rendering" as const;
export const LINEAR_LIGHT_COMPOSITING_CAPABILITY =
  "linear_light_compositing_v1" as const;
export const MASK_MODELS_CAPABILITY = "mask_models_v1" as const;
export const MASK_ANIMATION_CAPABILITY = "mask_animation_v1" as const;
export const MASK_RENDERING_CAPABILITY = "mask_rendering_v1" as const;
export type RenderingCapability =
  | "shape_rendering"
  | "svg_rendering"
  | "grid_rendering"
  | "repeater_rendering"
  | "transform2d"
  | "preview"
  | "preview_range"
  | "preview_review_presets_v1"
  | "mp4_export"
  | typeof EVALUATED_SCENE_RENDERING_CAPABILITY
  | typeof LINEAR_LIGHT_COMPOSITING_CAPABILITY
  | typeof MASK_RENDERING_CAPABILITY;

interface Revisioned {
  expectedRevision: number;
  projectId: string;
}

export type HeadlessEdit =
  | {
      operation: "marker_create";
      scope: string;
      name: string;
      timeMs: number;
      kind: "cue";
      resultAlias?: string | undefined;
    }
  | {
      operation: "marker_update";
      scope: string;
      markerId: string;
      name: string;
      timeMs: number;
      kind: "cue";
    }
  | { operation: "marker_delete"; scope: string; markerId: string }
  | {
      operation: "set_item_start_time";
      scope: string;
      itemId: string;
      time: z.infer<typeof timeExpressionSchema>;
    }
  | ({ operation: "add_svg"; resultAlias?: string | undefined } & z.infer<
      typeof addSvgSchema
    >)
  | ({ operation: "add_grid"; resultAlias?: string | undefined } & z.infer<
      typeof addGridSchema
    >)
  | ({ operation: "add_repeater"; resultAlias?: string | undefined } & z.infer<
      typeof addRepeaterSchema
    >)
  | ({ operation: "add_shape"; resultAlias?: string | undefined } & z.infer<
      typeof addShapeSchema
    >)
  | ({
      operation: "add_component_instance";
      resultAlias?: string | undefined;
    } & z.infer<typeof addComponentInstanceSchema>)
  | ({
      operation: "component_instance_duplicate";
      resultAlias?: string | undefined;
    } & z.infer<typeof componentInstanceDuplicateSchema>)
  | ({ operation: "component_instance_update" } & z.infer<
      typeof componentInstanceUpdateSchema
    >)
  | {
      operation: "component_define_slots";
      componentId: string;
      slots: z.infer<typeof templateSlotSchema>[];
    }
  | ({
      operation: "component_create";
      resultAlias?: string | undefined;
    } & z.infer<typeof componentFieldsSchema>)
  | ({ operation: "component_update"; componentId: string } & z.infer<
      typeof componentFieldsSchema
    >)
  | { operation: "component_delete"; componentId: string }
  | { operation: "group_ungroup"; groupId: string }
  | {
      operation: "add_group";
      staggerMs?: number | undefined;
      trackId: string;
      startMs: number;
      durationMs: number;
      transform2d?: z.infer<typeof transform2dSchema> | null | undefined;
      parent?: { scope: string; id: string } | null | undefined;
      resultAlias?: string | undefined;
    }
  | {
      operation: "item_set_parent";
      itemId: string;
      parent: { scope: string; id: string } | null;
    }
  | { operation: "item_set_z_index"; itemId: string; zIndex: number }
  | { operation: "item_reorder"; itemId: string; index: number }
  | { operation: "track_reorder"; trackId: string; index: number }
  | {
      operation: "add_media";
      assetId: string;
      durationMs: number;
      sourceInMs: number;
      startMs: number;
      trackId: string;
      resultAlias?: string | undefined;
    }
  | {
      operation: "add_text";
      color: string;
      durationMs: number;
      fontFamily?: string | undefined;
      fontPath?: string | undefined;
      fontSize: number;
      startMs: number;
      text?: string | undefined;
      document?: z.infer<typeof richTextDocumentSchema> | undefined;
      style: z.input<typeof textStyleSchema>;
      trackId: string;
      transform: {
        opacity: number;
        positionX: number;
        positionY: number;
        scale: number;
      };
      resultAlias?: string | undefined;
    }
  | {
      operation: "add_solid_color";
      color: string;
      durationMs: number;
      startMs: number;
      trackId: string;
      transform: {
        opacity: number;
        positionX: number;
        positionY: number;
        scale: number;
      };
      resultAlias?: string | undefined;
    }
  | {
      operation: "add_rectangle";
      color: string;
      durationMs: number;
      height: number;
      startMs: number;
      trackId: string;
      transform: {
        opacity: number;
        positionX: number;
        positionY: number;
        scale: number;
      };
      width: number;
      resultAlias?: string | undefined;
    }
  | {
      operation: "update_item";
      crop?: z.infer<typeof mediaCropSchema> | undefined;
      motionBlur?: z.infer<typeof motionBlurSchema> | undefined;
      effects?: z.infer<typeof visualEffectSchema>[] | undefined;
      masks?: z.infer<typeof maskSchema>[] | undefined;
      staggerMs?: number | undefined;
      geometry?: ShapeGeometry | undefined;
      grid?: GridDescriptor | undefined;
      repeater?: RepeaterDescriptor | undefined;
      fill?: Paint | null | undefined;
      stroke?: Stroke | null | undefined;
      transform2d?: z.infer<typeof transform2dSchema> | null | undefined;
      itemId: string;
      color?: string | undefined;
      fontSize?: number | undefined;
      fontFamily?: string | null | undefined;
      fontPath?: string | null | undefined;
      height?: number | undefined;
      text?: string | undefined;
      document?: z.infer<typeof richTextDocumentSchema> | undefined;
      style?: z.input<typeof textStyleSchema> | undefined;
      transform?:
        | {
            opacity: number;
            positionX: number;
            positionY: number;
            scale: number;
          }
        | undefined;
      width?: number | undefined;
    }
  | { operation: "move_item"; itemId: string; startMs: number; trackId: string }
  | {
      operation: "trim_item";
      durationMs: number;
      itemId: string;
      sourceInMs?: number | undefined;
      startMs: number;
    }
  | { operation: "delete_item"; itemId: string }
  | {
      operation: "apply_animation_preset";
      itemId: string;
      presetId: string;
      presetVersion: number;
      parameters: z.input<typeof animationPresetParametersSchema>;
      collisionPolicy?: "reject" | "replace" | undefined;
    }
  | { operation: "set_keyframes"; itemId: string; keyframes: unknown[] }
  | {
      operation: "set_animation_channels";
      itemId: string;
      animationChannels: z.input<typeof animationChannelSchema>[];
    }
  | {
      operation: "add_transition";
      durationMs: number;
      fromItemId: string;
      startMs: number;
      toItemId?: string | undefined;
      trackId: string;
      transitionType: "fade" | "crossfade";
      resultAlias?: string | undefined;
    }
  | {
      operation: "set_audio";
      audio: {
        fadeInMs: number;
        fadeOutMs: number;
        muted: boolean;
        volume: number;
      };
      itemId: string;
    }
  | { operation: "split_item"; itemId: string; splitMs: number }
  | { operation: "duplicate_items"; itemIds: string[]; offsetMs: number }
  | {
      operation: "create_track";
      index?: number | undefined;
      name: string;
      trackType: "video" | "overlay" | "audio" | "caption";
      audioRole: "unassigned" | "voiceover" | "music" | "sound_effects";
      ducking?:
        | {
            attackMs: number;
            enabled: boolean;
            gain: number;
            releaseMs: number;
          }
        | undefined;
      resultAlias?: string | undefined;
    }
  | {
      operation: "update_track";
      hidden?: boolean | undefined;
      audioRole?:
        | "unassigned"
        | "voiceover"
        | "music"
        | "sound_effects"
        | undefined;
      ducking?:
        | {
            attackMs: number;
            enabled: boolean;
            gain: number;
            releaseMs: number;
          }
        | null
        | undefined;
      index?: number | undefined;
      locked?: boolean | undefined;
      muted?: boolean | undefined;
      name?: string | undefined;
      trackId: string;
    }
  | { operation: "delete_track"; trackId: string }
  | { operation: "set_item_visibility"; hidden: boolean; itemId: string };

export type HeadlessRequest =
  | {
      operation: "status";
      protocolVersion?: 1 | undefined;
      textLayoutVersion?: 2 | undefined;
      styledTextLayersVersion?: 1 | undefined;
    }
  | { operation: "list_projects" }
  | {
      name: string;
      operation: "create_project";
      settings?: { fps: number; height: number; width: number };
    }
  | { operation: "open_project"; projectId: string }
  | {
      endMs?: number;
      operation: "get_state";
      projectId: string;
      startMs?: number;
    }
  | (Revisioned & {
      mediaType: "image" | "video" | "audio";
      operation: "import_asset";
      path: string;
    })
  | (Revisioned & { assetId: string; operation: "delete_asset" })
  | (Revisioned & {
      displayName: string;
      operation: "commit_generated_asset";
      origin: z.infer<typeof generatedAssetOriginSchema>;
      path: string;
      startMs: number;
      trackId: string;
    })
  | (Revisioned & {
      itemId: string;
      operation: "replace_generated_asset";
      origin: z.infer<typeof generatedAssetOriginSchema>;
      path: string;
    })
  | (Revisioned & { edit: HeadlessEdit; operation: "edit" })
  | (Revisioned & { operation: "edit_batch"; operations: HeadlessEdit[] })
  | (Revisioned & {
      label?: string | undefined;
      operation: "create_draft";
      operations: HeadlessEdit[];
    })
  | { draftId: string; operation: "get_draft"; projectId: string }
  | (Revisioned & {
      draftId: string;
      label?: string | undefined;
      operation: "update_draft";
      operations: HeadlessEdit[];
    })
  | (Revisioned & { draftId: string; operation: "rebase_draft" })
  | { draftId: string; operation: "get_draft_state"; projectId: string }
  | (Revisioned & { draftId: string; operation: "commit_draft" })
  | { draftId: string; operation: "discard_draft"; projectId: string }
  | {
      draftId: string;
      operation: "render_draft_preview";
      projectId: string;
      timeMs: number;
    }
  | { assetId: string; operation: "resolve_asset_input"; projectId: string }
  | (Revisioned & {
      assetId: string;
      captionTrackId?: string | undefined;
      generatedAtMs: number;
      language: string;
      modelId: string;
      modelVersion?: string | undefined;
      operation: "commit_transcription";
      providerId: string;
      segments: Array<{
        confidence?: number | undefined;
        endMs: number;
        startMs: number;
        text: string;
        words?:
          | Array<{
              confidence?: number | undefined;
              endMs: number;
              startMs: number;
              word: string;
            }>
          | undefined;
      }>;
      style?:
        | {
            backgroundColor: string;
            bottomMarginPx: number;
            color: string;
            fontSize: number;
          }
        | undefined;
    })
  | (Revisioned & { operation: "undo" | "redo" })
  | (Revisioned & { operation: "render_preview"; timeMs: number })
  | (Revisioned & {
      operation: "render_review_range";
      startMs: number;
      endMs: number;
      resolution?:
        | "540p"
        | "720p"
        | "project"
        | { width: number; height: number };
      fps?: number | undefined;
      includeAudio?: boolean;
    })
  | (Revisioned & {
      operation: "render_preview_range";
      startMs: number;
      endMs: number;
      width: number;
      height: number;
      fps: number;
      includeAudio: boolean;
    })
  | (Revisioned & {
      height: number;
      operation: "export_video";
      overwrite: boolean;
      relativePath: string;
      width: number;
    });
