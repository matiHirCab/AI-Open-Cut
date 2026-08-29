import { z } from "zod/v4";

import type { HeadlessEdit, HeadlessRequest } from "../headless-contract";
import {
  projectStateSchema,
  schemas,
  timelineItemSchema,
  writeResultSchema,
} from "../schemas";
import {
  DESTRUCTIVE,
  failure,
  invoke,
  READ_ONLY,
  type Server,
  type ServerDependencies,
  success,
  WRITE,
} from "./shared";

const itemListSchema = z
  .object({
    items: z.array(timelineItemSchema),
    projectId: z.string(),
    revision: z.int().nonnegative(),
  })
  .strict();
const editRequest = (
  projectId: string,
  expectedRevision: number,
  edit: HeadlessEdit
): Extract<HeadlessRequest, { operation: "edit" }> => ({
  edit,
  expectedRevision,
  operation: "edit",
  projectId,
});

export const registerTimelineTools = (
  server: Server,
  { headless }: ServerDependencies
) => {
  server.registerTool(
    "timeline_get_items",
    {
      annotations: READ_ONLY,
      description: "List filtered timeline items.",
      inputSchema: schemas.timelineGetItems,
      outputSchema: itemListSchema,
    },
    async ({ projectId, trackId, itemType, timeRange }) => {
      try {
        const state = await headless.call(
          {
            ...(timeRange
              ? { endMs: timeRange.endMs, startMs: timeRange.startMs }
              : {}),
            operation: "get_state",
            projectId,
          },
          projectStateSchema
        );
        const items = state.project.tracks
          .filter((track) => trackId === undefined || track.id === trackId)
          .flatMap((track) => track.items)
          .filter((item) => itemType === undefined || item.type === itemType);
        return success({ items, projectId, revision: state.project.revision });
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "timeline_add_media",
    {
      annotations: WRITE,
      description: "Add media to a timeline track.",
      inputSchema: schemas.timelineAddMedia,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, ...edit }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            operation: "add_media",
            ...edit,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "timeline_add_text",
    {
      annotations: WRITE,
      description: "Add text to an overlay track.",
      inputSchema: schemas.timelineAddText,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, ...edit }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            operation: "add_text",
            ...edit,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "timeline_update_item",
    {
      annotations: WRITE,
      description: "Update a timeline item.",
      inputSchema: schemas.timelineUpdateItem,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, ...edit }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            operation: "update_item",
            ...edit,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "timeline_move_item",
    {
      annotations: WRITE,
      description: "Move a timeline item.",
      inputSchema: schemas.timelineMoveItem,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, ...edit }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            operation: "move_item",
            ...edit,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "timeline_trim_item",
    {
      annotations: WRITE,
      description: "Trim a timeline item.",
      inputSchema: schemas.timelineTrimItem,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, ...edit }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            operation: "trim_item",
            ...edit,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "timeline_delete_item",
    {
      annotations: DESTRUCTIVE,
      description: "Reversibly delete a timeline item.",
      inputSchema: schemas.timelineDeleteItem,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, ...edit }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            operation: "delete_item",
            ...edit,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "timeline_set_keyframes",
    {
      annotations: WRITE,
      description: "Replace timeline keyframes.",
      inputSchema: schemas.timelineSetKeyframes,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, ...edit }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            operation: "set_keyframes",
            ...edit,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "timeline_add_transition",
    {
      annotations: WRITE,
      description: "Add a timeline transition.",
      inputSchema: schemas.timelineAddTransition,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, ...edit }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            operation: "add_transition",
            ...edit,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "timeline_set_audio",
    {
      annotations: WRITE,
      description: "Update timeline audio controls.",
      inputSchema: schemas.timelineSetAudio,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, ...edit }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            operation: "set_audio",
            ...edit,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "timeline_split_item",
    {
      annotations: WRITE,
      description:
        "Split a media, text, or caption item at an absolute timeline time.",
      inputSchema: schemas.timelineSplitItem,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, itemId, splitMs }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            itemId,
            operation: "split_item",
            splitMs,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "timeline_duplicate_items",
    {
      annotations: WRITE,
      description:
        "Duplicate media, text, or caption items on their current tracks.",
      inputSchema: schemas.timelineDuplicateItems,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, itemIds, offsetMs }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            itemIds,
            offsetMs,
            operation: "duplicate_items",
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "timeline_batch_edit",
    {
      annotations: WRITE,
      description:
        "Apply up to 100 typed edits atomically as one revision and undo step.",
      inputSchema: schemas.timelineBatchEdit,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, operations }) => {
      try {
        return await invoke(
          headless,
          { expectedRevision, operation: "edit_batch", operations, projectId },
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "track_create",
    {
      annotations: WRITE,
      description: "Create a typed timeline track.",
      inputSchema: schemas.trackCreate,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, ...edit }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            operation: "create_track",
            ...edit,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "track_update",
    {
      annotations: WRITE,
      description: "Rename, reorder, lock, hide, or mute a timeline track.",
      inputSchema: schemas.trackUpdate,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, ...edit }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            operation: "update_track",
            ...edit,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "track_delete",
    {
      annotations: DESTRUCTIVE,
      description: "Delete an empty, unlocked timeline track.",
      inputSchema: schemas.trackDelete,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, trackId }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            operation: "delete_track",
            trackId,
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "timeline_set_item_visibility",
    {
      annotations: WRITE,
      description: "Show or hide a timeline item without deleting it.",
      inputSchema: schemas.timelineSetItemVisibility,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, itemId, hidden }) => {
      try {
        return await invoke(
          headless,
          editRequest(projectId, expectedRevision, {
            hidden,
            itemId,
            operation: "set_item_visibility",
          }),
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
};
