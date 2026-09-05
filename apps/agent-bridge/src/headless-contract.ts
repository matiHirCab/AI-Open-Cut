import type { z } from "zod/v4";

import type { generatedAssetOriginSchema, transform2dSchema } from "./schemas";

export const EVALUATED_SCENE_RENDERING_CAPABILITY =
  "evaluated_scene_rendering" as const;
export type RenderingCapability =
  | "transform2d"
  | "preview"
  | "preview_range"
  | "mp4_export"
  | typeof EVALUATED_SCENE_RENDERING_CAPABILITY;

interface Revisioned {
  expectedRevision: number;
  projectId: string;
}

export type HeadlessEdit =
  | { operation: "group_ungroup"; groupId: string }
  | {
      operation: "add_group";
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
      text: string;
      style: Record<string, unknown>;
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
      transform2d?: z.infer<typeof transform2dSchema> | null | undefined;
      itemId: string;
      color?: string | undefined;
      fontFamily?: string | null | undefined;
      fontPath?: string | null | undefined;
      height?: number | undefined;
      text?: string | undefined;
      style?: Record<string, unknown> | undefined;
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
  | { operation: "set_keyframes"; itemId: string; keyframes: unknown[] }
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
  | { operation: "status"; protocolVersion?: 1 | undefined }
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
