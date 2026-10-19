import type { z } from "zod/v4";
import { BridgeError, type HeadlessClient } from "../headless";
import type { HeadlessRequest } from "../headless-contract";
import { projectStateSchema, type schemas } from "../schemas";

// No durable storage bound applies here: the existing tools retain their inputs.
export type ProductionIntent =
  | {
      kind: "preview_frame";
      request: z.output<typeof schemas.previewRenderFrame>;
    }
  | {
      kind: "preview_range";
      request: z.output<typeof schemas.previewRenderRange>;
    }
  | {
      kind: "review_range";
      request: z.output<typeof schemas.previewReviewRange>;
    }
  | {
      kind: "export_video";
      request: z.output<typeof schemas.projectExportVideo>;
    };

export interface PreparedProduction {
  kind: "preview" | "preview_range" | "export";
  request: Extract<
    HeadlessRequest,
    {
      operation:
        | "render_preview"
        | "render_preview_range"
        | "render_review_range"
        | "export_video";
    }
  >;
}

/** Shared old-tool preparation, with editor-core retaining final domain authority. */
export const prepareProduction = async (
  headless: HeadlessClient,
  intent: ProductionIntent
): Promise<PreparedProduction> => {
  const { projectId, expectedRevision } = intent.request;
  const state = await headless.call(
    { operation: "get_state", projectId },
    projectStateSchema
  );
  if (state.project.revision !== expectedRevision) {
    throw new BridgeError(
      "REVISION_CONFLICT",
      `Expected revision ${expectedRevision}, current revision is ${state.project.revision}`
    );
  }
  const base = { expectedRevision, projectId };
  switch (intent.kind) {
    case "preview_frame":
      return {
        kind: "preview",
        request: {
          ...base,
          operation: "render_preview",
          timeMs: intent.request.timeMs,
        },
      };
    case "preview_range": {
      const { startMs, endMs, resolution, fps, includeAudio } = intent.request;
      return {
        kind: "preview_range",
        request: {
          ...base,
          endMs,
          fps,
          height: resolution.height,
          includeAudio,
          operation: "render_preview_range",
          startMs,
          width: resolution.width,
        },
      };
    }
    case "review_range": {
      const { startMs, endMs, resolution, fps, includeAudio } = intent.request;
      return {
        kind: "preview_range",
        request: {
          ...base,
          endMs,
          fps,
          includeAudio,
          operation: "render_review_range",
          resolution,
          startMs,
        },
      };
    }
    case "export_video": {
      const { relativePath, resolution, overwrite } = intent.request;
      let dimensions = { height: 720, width: 1280 };
      if (resolution === "project") {
        dimensions = state.project.settings;
      } else if (resolution === "1080p") {
        dimensions = { height: 1080, width: 1920 };
      }
      return {
        kind: "export",
        request: {
          ...base,
          height: dimensions.height,
          operation: "export_video",
          overwrite,
          relativePath,
          width: dimensions.width,
        },
      };
    }
    default: {
      const exhaustive: never = intent;
      return exhaustive;
    }
  }
};
