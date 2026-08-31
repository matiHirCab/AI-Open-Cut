import type { z } from "zod/v4";
import { runtimePathDiagnostics } from "../diagnostics";
import { errorBody } from "../headless";
import {
  headlessStatusSchema,
  projectStateSchema,
  schemas,
  statusSchema,
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

export const registerProjectTools = (
  server: Server,
  dependencies: ServerDependencies
) => {
  const { config, headless, session, speech, transcription } = dependencies;
  server.registerTool(
    "editor_get_status",
    {
      annotations: READ_ONLY,
      description:
        "Check editor and optional subsystem readiness with sanitized diagnostics for media, project, export, generated-media, FFmpeg, FFprobe, and Kokoro paths.",
      inputSchema: schemas.editorGetStatus,
      outputSchema: statusSchema,
    },
    async ({ protocolVersion }) => {
      try {
        const status = await headless.call(
          {
            operation: "status",
            ...(protocolVersion === undefined ? {} : { protocolVersion }),
          },
          headlessStatusSchema
        );
        let { capabilities } = status;
        let speechSubsystem: z.infer<
          typeof statusSchema
        >["subsystems"]["speech"];
        let transcriptionSubsystem: z.infer<
          typeof statusSchema
        >["subsystems"]["transcription"];
        try {
          const speechStatus = await speech.status();
          if (speechStatus.ready) {
            capabilities = [...capabilities, "tts"];
          }
          speechSubsystem = {
            capabilities: speechStatus.ready ? ["tts"] : [],
            error: speechStatus.startupError ?? null,
            modelId: speechStatus.modelId,
            providerId: speechStatus.providerId,
            queue: speechStatus.queue,
            ready: speechStatus.ready,
          };
        } catch (error) {
          speechSubsystem = {
            capabilities: [],
            error: errorBody(error),
            modelId: null,
            providerId: null,
            queue: null,
            ready: false,
          };
        }
        try {
          const transcriptionStatus = await transcription.status();
          if (transcriptionStatus.ready) {
            capabilities = [...capabilities, "transcription", "captions"];
          }
          transcriptionSubsystem = {
            capabilities: transcriptionStatus.ready
              ? ["transcription", "captions"]
              : [],
            error: null,
            modelId: transcriptionStatus.modelId,
            providerId: transcriptionStatus.providerId,
            queue: transcriptionStatus.queue,
            ready: transcriptionStatus.ready,
          };
        } catch (error) {
          transcriptionSubsystem = {
            capabilities: [],
            error: errorBody(error),
            modelId: null,
            providerId: null,
            queue: null,
            ready: false,
          };
        }
        return success({
          ...status,
          activeProjectId: session.activeProjectId,
          capabilities,
          paths: await runtimePathDiagnostics(config),
          subsystems: {
            ...status.subsystems,
            speech: speechSubsystem,
            transcription: transcriptionSubsystem,
          },
        });
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "project_create",
    {
      annotations: WRITE,
      description: "Create a local OpenCut project.",
      inputSchema: schemas.projectCreate,
      outputSchema: writeResultSchema,
    },
    async ({ name, width, height, fps }) => {
      try {
        const result = await headless.call(
          {
            name,
            operation: "create_project",
            settings: { fps, height, width },
          },
          writeResultSchema
        );
        session.activeProjectId = result.projectId;
        return success(result);
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "project_open",
    {
      annotations: WRITE,
      description: "Open an existing project in this session.",
      inputSchema: schemas.projectOpen,
      outputSchema: projectStateSchema,
    },
    async ({ projectId }) => {
      try {
        const result = await headless.call(
          { operation: "open_project", projectId },
          projectStateSchema
        );
        session.activeProjectId = projectId;
        return success(result);
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "project_get_state",
    {
      annotations: READ_ONLY,
      description: "Read project state and optional time range.",
      inputSchema: schemas.projectGetState,
      outputSchema: projectStateSchema,
    },
    async ({ projectId, timeRange }) => {
      try {
        return await invoke(
          headless,
          {
            ...(timeRange
              ? { endMs: timeRange.endMs, startMs: timeRange.startMs }
              : {}),
            operation: "get_state",
            projectId,
          },
          projectStateSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "asset_import",
    {
      annotations: WRITE,
      description: "Import approved local media into a project.",
      inputSchema: schemas.assetImport,
      outputSchema: writeResultSchema,
    },
    async ({ projectId, expectedRevision, path, mediaType }) => {
      try {
        return await invoke(
          headless,
          {
            expectedRevision,
            mediaType,
            operation: "import_asset",
            path,
            projectId,
          },
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "asset_delete",
    {
      annotations: DESTRUCTIVE,
      description: "Delete an unused logical asset with undo support.",
      inputSchema: schemas.assetDelete,
      outputSchema: writeResultSchema,
    },
    async ({ assetId, expectedRevision, projectId }) => {
      try {
        return await invoke(
          headless,
          { assetId, expectedRevision, operation: "delete_asset", projectId },
          writeResultSchema
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  for (const [name, schema, operation] of [
    ["project_undo", schemas.projectUndo, "undo"],
    ["project_redo", schemas.projectRedo, "redo"],
  ] as const) {
    server.registerTool(
      name,
      {
        annotations: WRITE,
        description: `${operation} the latest project history step.`,
        inputSchema: schema,
        outputSchema: writeResultSchema,
      },
      async ({ projectId, expectedRevision }) => {
        try {
          return await invoke(
            headless,
            { expectedRevision, operation, projectId },
            writeResultSchema
          );
        } catch (error) {
          return failure(error);
        }
      }
    );
  }
};
