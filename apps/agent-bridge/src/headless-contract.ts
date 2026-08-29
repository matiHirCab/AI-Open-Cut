import type { z } from "zod/v4";

import type { generatedAssetOriginSchema } from "./schemas";

interface Revisioned {
  expectedRevision: number;
  projectId: string;
}

export type HeadlessEdit =
  | {
      operation: "add_media";
      assetId: string;
      durationMs: number;
      sourceInMs: number;
      startMs: number;
      trackId: string;
    }
  | {
      operation: "add_text";
      color: string;
      durationMs: number;
      fontFamily?: string | undefined;
      fontSize: number;
      startMs: number;
      text: string;
      trackId: string;
      transform: {
        opacity: number;
        positionX: number;
        positionY: number;
        scale: number;
      };
    }
  | {
      operation: "update_item";
      itemId: string;
      text?: string | undefined;
      transform?:
        | {
            opacity: number;
            positionX: number;
            positionY: number;
            scale: number;
          }
        | undefined;
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
    }
  | {
      operation: "update_track";
      hidden?: boolean | undefined;
      index?: number | undefined;
      locked?: boolean | undefined;
      muted?: boolean | undefined;
      name?: string | undefined;
      trackId: string;
    }
  | { operation: "delete_track"; trackId: string }
  | { operation: "set_item_visibility"; hidden: boolean; itemId: string };

export type HeadlessRequest =
  | { operation: "status" }
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
      height: number;
      operation: "export_video";
      overwrite: boolean;
      relativePath: string;
      width: number;
    });
