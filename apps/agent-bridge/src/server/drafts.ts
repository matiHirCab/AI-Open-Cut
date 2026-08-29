import {
  editDraftSchema,
  jobSchema,
  schemas,
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

export const registerDraftTools = (
  server: Server,
  { headless, jobs }: ServerDependencies
) => {
  server.registerTool(
    "draft_create",
    {
      annotations: WRITE,
      description:
        "Create a durable validated edit draft without changing the project revision.",
      inputSchema: schemas.draftCreate,
      outputSchema: editDraftSchema,
    },
    async ({ projectId, expectedRevision, operations, label }) => {
      try {
        return await invoke(
          headless,
          {
            expectedRevision,
            label,
            operation: "create_draft",
            operations,
            projectId,
          },
          editDraftSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "draft_get",
    {
      annotations: READ_ONLY,
      description: "Read a durable edit draft.",
      inputSchema: schemas.draftGet,
      outputSchema: editDraftSchema,
    },
    async ({ projectId, draftId }) => {
      try {
        return await invoke(
          headless,
          { draftId, operation: "get_draft", projectId },
          editDraftSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "draft_update",
    {
      annotations: WRITE,
      description:
        "Replace the operations and optional label of a current-revision draft.",
      inputSchema: schemas.draftUpdate,
      outputSchema: editDraftSchema,
    },
    async ({ projectId, expectedRevision, draftId, operations, label }) => {
      try {
        return await invoke(
          headless,
          {
            draftId,
            expectedRevision,
            label,
            operation: "update_draft",
            operations,
            projectId,
          },
          editDraftSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "draft_rebase",
    {
      annotations: WRITE,
      description:
        "Revalidate a retained draft against a newly read project revision.",
      inputSchema: schemas.draftRebase,
      outputSchema: editDraftSchema,
    },
    async ({ projectId, expectedRevision, draftId }) => {
      try {
        return await invoke(
          headless,
          { draftId, expectedRevision, operation: "rebase_draft", projectId },
          editDraftSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "draft_preview_frame",
    {
      annotations: WRITE,
      description:
        "Queue a PNG render of a draft without mutating project state.",
      inputSchema: schemas.draftPreviewFrame,
      outputSchema: jobSchema,
    },
    async ({ projectId, draftId, timeMs }) => {
      try {
        const draft = await headless.call(
          { draftId, operation: "get_draft", projectId },
          editDraftSchema
        );
        return success(
          jobs.start("preview", projectId, draft.baseRevision, {
            draftId,
            operation: "render_draft_preview",
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
    "draft_commit",
    {
      annotations: WRITE,
      description:
        "Commit a draft atomically as one project revision and undo step.",
      inputSchema: schemas.draftCommit,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, draftId }) => {
      try {
        return await invoke(
          headless,
          { draftId, expectedRevision, operation: "commit_draft", projectId },
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "draft_discard",
    {
      annotations: DESTRUCTIVE,
      description:
        "Permanently discard a retained edit draft without changing the project.",
      inputSchema: schemas.draftDiscard,
      outputSchema: editDraftSchema,
    },
    async ({ projectId, draftId }) => {
      try {
        return await invoke(
          headless,
          { draftId, operation: "discard_draft", projectId },
          editDraftSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
};
