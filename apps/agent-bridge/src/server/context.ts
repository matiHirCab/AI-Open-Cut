import { type McpServer, ResourceTemplate } from "@modelcontextprotocol/server";
import { z } from "zod/v4";

import {
  editDraftSchema,
  projectListSchema,
  projectStateSchema,
} from "../schemas";
import type { ServerDependencies } from "./shared";

const jsonResource = (uri: URL, value: unknown) => ({
  contents: [
    {
      mimeType: "application/json",
      text: JSON.stringify(value, null, 2),
      uri: uri.href,
    },
  ],
});

export const MCP_RESOURCE_URIS = {
  jobStatus: "opencut://jobs/{jobId}",
  projectDraft: "opencut://projects/{projectId}/drafts/{draftId}",
  projectState: "opencut://projects/{projectId}/state",
  projects: "opencut://projects",
  projectTimeline: "opencut://projects/{projectId}/timeline",
} as const;

export const WORKFLOW_PROMPT_NAMES = [
  "add_narration",
  "assemble_clips",
  "create_intro_video",
  "transcribe_and_caption",
] as const;

export const registerContextResources = (
  server: McpServer,
  { headless, jobs }: ServerDependencies
) => {
  server.registerResource(
    "projects",
    MCP_RESOURCE_URIS.projects,
    {
      description:
        "Discover local OpenCut projects and their current revisions.",
      mimeType: "application/json",
      title: "OpenCut projects",
    },
    async (uri) =>
      jsonResource(
        uri,
        await headless.call({ operation: "list_projects" }, projectListSchema)
      )
  );

  for (const [name, suffix, description] of [
    [
      "project-state",
      "state",
      "Complete project state including its revision.",
    ],
    [
      "project-timeline",
      "timeline",
      "Timeline tracks and items including the project revision.",
    ],
  ] as const) {
    server.registerResource(
      name,
      new ResourceTemplate(
        suffix === "state"
          ? MCP_RESOURCE_URIS.projectState
          : MCP_RESOURCE_URIS.projectTimeline,
        {
          list: undefined,
        }
      ),
      { description, mimeType: "application/json" },
      async (uri, variables) => {
        const projectId = String(variables.projectId);
        const state = await headless.call(
          { operation: "get_state", projectId },
          projectStateSchema
        );
        return jsonResource(
          uri,
          suffix === "state"
            ? state
            : {
                projectId,
                revision: state.project.revision,
                tracks: state.project.tracks,
              }
        );
      }
    );
  }

  server.registerResource(
    "job-status",
    new ResourceTemplate(MCP_RESOURCE_URIS.jobStatus, { list: undefined }),
    {
      description: "Current process-local OpenCut job status.",
      mimeType: "application/json",
    },
    (uri, variables) => jsonResource(uri, jobs.get(String(variables.jobId)))
  );

  server.registerResource(
    "project-draft",
    new ResourceTemplate(MCP_RESOURCE_URIS.projectDraft, {
      list: undefined,
    }),
    {
      description: "A durable validated edit draft and its base revision.",
      mimeType: "application/json",
    },
    async (uri, variables) => {
      const projectId = String(variables.projectId);
      const [draft, state] = await Promise.all([
        headless.call(
          {
            draftId: String(variables.draftId),
            operation: "get_draft",
            projectId,
          },
          editDraftSchema
        ),
        headless.call(
          { operation: "get_state", projectId },
          projectStateSchema
        ),
      ]);
      return jsonResource(uri, {
        draft,
        projectId,
        revision: state.project.revision,
      });
    }
  );
};

const promptMessage = (text: string) => ({
  messages: [
    { content: { text, type: "text" as const }, role: "user" as const },
  ],
});

export const registerWorkflowPrompts = (server: McpServer) => {
  server.registerPrompt(
    "create_intro_video",
    {
      argsSchema: z
        .object({
          durationMs: z.string().optional(),
          text: z.string().min(1),
        })
        .strict(),
      description: "Create and preview a simple title intro.",
    },
    ({ text, durationMs }) =>
      promptMessage(
        `Call editor_get_status, create a project, then read project_get_state to obtain its overlay track and revision. Add the title ${JSON.stringify(text)} for ${durationMs ?? "10000"} ms with timeline_add_text. Re-read state, render a preview frame, poll the job, and show the preview before offering export. Never enable overwrite without explicit permission.`
      )
  );

  server.registerPrompt(
    "assemble_clips",
    {
      argsSchema: z
        .object({ projectId: z.string().min(1), request: z.string().min(1) })
        .strict(),
      description: "Assemble imported clips into one atomic timeline edit.",
    },
    ({ projectId, request }) =>
      promptMessage(
        `For project ${projectId}, call project_get_state and inspect available assets and tracks. Translate this request into typed operations: ${request}. Use timeline_batch_edit with the returned revision, then re-read state and render a preview. If any asset is missing, ask for an approved local path before calling asset_import.`
      )
  );

  server.registerPrompt(
    "add_narration",
    {
      argsSchema: z
        .object({ projectId: z.string().min(1), text: z.string().min(1) })
        .strict(),
      description: "Preview and commit local speech narration.",
    },
    ({ projectId, text }) =>
      promptMessage(
        `Call speech_list_voices and speech_estimate for ${JSON.stringify(text)}. Queue speech_preview and poll job_get_status. Present the audio preview; only after approval, read project_get_state for project ${projectId} and call speech_commit_preview with its current revision and an audio-track placement. Re-read project state after commit.`
      )
  );

  server.registerPrompt(
    "transcribe_and_caption",
    {
      argsSchema: z
        .object({ assetId: z.string().min(1), projectId: z.string().min(1) })
        .strict(),
      description: "Transcribe project media, review it, and commit captions.",
    },
    ({ assetId, projectId }) =>
      promptMessage(
        `Call transcription_get_status, then transcription_estimate and transcription_preview for asset ${assetId} in project ${projectId}. Poll job_get_status and present the timed transcript for review. After approval, re-read project_get_state and call transcription_commit_preview with the current revision. Render and present a captioned preview before export.`
      )
  );
};
