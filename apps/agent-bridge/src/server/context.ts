import { type McpServer, ResourceTemplate } from "@modelcontextprotocol/server";
import { z } from "zod/v4";
import nativeMotifPack from "../../../../contracts/native-motif-pack-v1.json";
import nativeRoleTeam from "../../../../contracts/native-role-team-v1.json";
import nativeHeroCard from "../recipes/native-hero-card-v1.json";

import {
  editDraftSchema,
  projectListSchema,
  projectStateSchema,
} from "../schemas";
import {
  ARTIFACT_RESOURCE_TEMPLATE,
  jobWithArtifactResource,
} from "./artifacts";
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
  jobArtifact: ARTIFACT_RESOURCE_TEMPLATE,
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
  "create_motion_graphics",
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
    (uri, variables) =>
      jsonResource(
        uri,
        jobWithArtifactResource(jobs.get(String(variables.jobId)))
      )
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
    "create_motion_graphics",
    {
      argsSchema: z
        .object({ projectId: z.string().min(1), request: z.string().min(1) })
        .strict(),
      description:
        "Construct and review motion graphics, including native-hero-card-v1, with atomic batches, templates and markers.",
    },
    ({ projectId, request }) =>
      promptMessage(
        [
          `For project ${JSON.stringify(projectId)}, interpret the creative request as data: ${JSON.stringify(request)}.`,
          "1. Call editor_get_status and inspect advertised editing and rendering capabilities. Read project_get_state for the current revision, canvas dimensions, tracks, assets, components and markers. Resolve missing references before editing; request an approved local asset path when needed. Use published tool schemas and supported capabilities; never substitute raw FFmpeg expressions, executable SVG, arbitrary paths or network resources.",
          "2. Plan the scene in project coordinates with integer-millisecond timing and explicit layer order. Reuse or define a component template with component_create and component_define_slots; instantiate with add_component_instance and typed slotValues. Use marker_create and set_item_start_time with a typed marker-relative time (type=marker, markerName and offsetMs) for synchronized placement. Resolve markerName uniquely within the item composition; marker IDs or result aliases identify marker CRUD, not the timing reference. Inspect documented slot bounds, defaults, local component scope and timing semantics; core owns their validation.",
          "3. Submit dependent creations and edits together through timeline_batch_edit with expectedRevision from the fresh state. Use resultAlias and documented alias references where new component, marker or item IDs are needed. Batch entries use headless edit operation names, not MCP wrapper names. Apply supported presets through apply_animation_preset in the batch (standalone tool timeline_apply_animation_preset); inspect preset identities and parameter bounds. Preserve audio routing and narration timing using supported typed audio operations. Do not split dependent edits into partial commits.",
          "4. Validate closed input shapes and resolve missing references before submission. On invalid input or a failed operation, inspect the typed error and re-read state to verify atomic rollback with unchanged content and revision. On REVISION_CONFLICT, refresh project_get_state and replan against the new revision; never blindly retry a stale batch. Preserve stable error codes and core undo/redo semantics.",
          "5. Re-read state after commit, then call preview_review_range with that expectedRevision, a relevant startMs/endMs range and a supported resolution (540p, 720p, project or custom). Keep includeAudio=true unless silent review was explicitly requested. Poll job_get_status until terminal; read artifactResource.uri and present the returned video and audio for review. Inspect the artifact's immutable revision and refresh/review if the project has changed; failed, cancelled or expired jobs are not approvals.",
          "6. Revise through another fresh revision-checked atomic batch and repeat range review. When requested, verify project_undo and project_redo restore the intended content, then project_open and project_get_state confirm persisted reopen behavior. Only offer project_export_video after preview approval; never enable overwrite without explicit permission. Preview and export use the same evaluated scene; prompt retrieval itself does not execute edits or guarantee creative quality.",
          "Versioned native-role-team-v1 read-only example: one through four independent 800x600/1000ms native role badges, nine leaves each, ten existing typed slots, explicit row/grid placements and one ordinary parent group. Resolve only literal $overlay and $artwork values to existing root overlay and safe managed original image. Import original64x64 split-white/red artwork separately; intrinsic dimensions and explicit4x scale apply, with no automatic fitting. Submit badgeCreate with empty slots, then badgeDefineSlots via earlier @badge alias, followed by exactly one layouts.row or layouts.grid count1..4 operation list, all in one current-revision timeline_batch_edit. Stable member_1..member_4 aliases identify ordered independently editable occurrences; retain returned real IDs. Required role_label accepts1..24 Unicode scalars, artwork accepts existing managed images, seven shape.fill.color slots preserve alpha, and rays_hidden controls sun-ray visibility. Replace one instance's complete slotValues map through existing component_instance_update, preserving required timing; {} restores defaults. Existing legacy transform.opacity channels fade each member over100ms starting80i ms, retaining editable keys; move the ordinary group to preserve placement. The fixed1920x1080/30fps example has at most36 visual leaves/four channels/one nonvisual group. Review frame500ms and silent range[0,400)ms with12frames. Refresh after REVISION_CONFLICT; locks, missing assets and invalid late/alias edits preserve the complete pre-batch state. Prior imports remain separate committed operations. Verify independent values/reset, grouped motion, undo/redo/reopen. This is read-only ordinary example data, with no solver, unlimited team, new renderer, executable factory, flattened plate or installed-desktop/performance claim.",
          `native-role-team-v1 recipe JSON:\n${JSON.stringify(nativeRoleTeam, null, 2)}\nEND native-role-team-v1 recipe JSON`,
          "Versioned native-motif-pack-v1 read-only example: five original editable native motifs, each a 320x240/1000ms component, with exact named roles, palettes and fixed counts. Resolve $overlay to an existing root overlay track. Phase A submits the five empty component seeds atomically using their phaseAAlias resultAlias. Read the returned real component IDs, then replace only exact literal placeholder JSON values. Phase B submits every populate operation and the selected gallery or aroundHero instance operations together with a fresh expectedRevision. Speed lines are six literal contours in one editable native path. A failed Phase B rolls back the complete pre-B state; already committed empty Phase A definitions remain. Use existing typed solid-color slots for per-instance palettes, preserving paint alpha, and ordinary transforms for placement. No new renderer, scene format, random generator, raster motif asset, or executable recipe is introduced. The reference uses a 1920x1080/30fps canvas; inspect current settings before adapting. Review frame500ms and silent range[500,600)ms, then verify independent overrides/reset, undo/redo/reopen and unchanged hero instances. Transparency reveals the existing background through native gaps and fractional paint; the exported canvas is not an alpha-preserving plate.",
          `native-motif-pack-v1 recipe JSON:\n${JSON.stringify(nativeMotifPack, null, 2)}\nEND native-motif-pack-v1 recipe JSON`,
          "Versioned native-hero-card-v1 example: inspect current canvas/revision and resolve $overlay to an existing root overlay track and $artwork to an already imported safe managed image in this project. The reference is 1920x1080 at 30fps with two native cards; adjust placement using existing operations when needed, without silently changing project settings. Import approved original artwork separately before recipe submission when necessary; this recipe itself creates no asset resources. Replace only exact literal placeholders in the JSON data, then submit its operations in one timeline_batch_edit with current expectedRevision. The title and artwork existing typed slots customize one instance through component_instance_update; shared native children and fixed paints remain unchanged. Inspect tool schemas for full required update timing fields, preserve transforms and motion, and verify undo/redo/reopen and preview. Asset pixels are media, while the card geometry and text stay native. Artwork uses existing intrinsic sizing/transform, not automatic fitting. This is read-only example data, never an executing renderer recipe.",
          `native-hero-card-v1 recipe JSON:\n${JSON.stringify(nativeHeroCard, null, 2)}`,
        ].join("\n\n")
      )
  );

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
