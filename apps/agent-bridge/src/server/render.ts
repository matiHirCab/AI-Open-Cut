import { BridgeError } from "../headless";
import { jobSchema, projectStateSchema, schemas } from "../schemas";
import {
  DESTRUCTIVE,
  failure,
  type Server,
  type ServerDependencies,
  success,
  WRITE,
} from "./shared";

export const registerRenderTools = (
  server: Server,
  { headless, jobs }: ServerDependencies
) => {
  server.registerTool(
    "preview_render_frame",
    {
      annotations: WRITE,
      description: "Queue a PNG preview render without changing project state.",
      inputSchema: schemas.previewRenderFrame,
      outputSchema: jobSchema,
    },
    async ({ projectId, expectedRevision, timeMs }) => {
      try {
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
        return success(
          jobs.start("preview", projectId, expectedRevision, {
            expectedRevision,
            operation: "render_preview",
            projectId,
            timeMs,
          })
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "project_export_video",
    {
      annotations: DESTRUCTIVE,
      description:
        "Queue a local MP4 export for an immutable project revision.",
      inputSchema: schemas.projectExportVideo,
      outputSchema: jobSchema,
    },
    async ({
      projectId,
      expectedRevision,
      relativePath,
      resolution,
      overwrite,
    }) => {
      try {
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
        let dimensions = { height: 720, width: 1280 };
        if (resolution === "project") {
          dimensions = state.project.settings;
        } else if (resolution === "1080p") {
          dimensions = { height: 1080, width: 1920 };
        }
        return success(
          jobs.start("export", projectId, expectedRevision, {
            expectedRevision,
            height: dimensions.height,
            operation: "export_video",
            overwrite,
            projectId,
            relativePath,
            width: dimensions.width,
          })
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
};
