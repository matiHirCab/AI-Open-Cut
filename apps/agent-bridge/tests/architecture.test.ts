import { describe, expect, it } from "vitest";

import type { HeadlessRequest } from "../src/headless-contract";
import { registerJobTools } from "../src/server/jobs";
import { registerProjectTools } from "../src/server/projects";
import { registerRenderTools } from "../src/server/render";
import type { Server, ServerDependencies } from "../src/server/shared";
import { registerSpeechTools } from "../src/server/speech";
import { registerTimelineTools } from "../src/server/timeline";

type Handler = (input: Record<string, unknown>) => Promise<unknown> | unknown;

class RegistrationHarness {
  readonly handlers = new Map<string, Handler>();

  registerTool(name: string, _definition: unknown, handler: Handler) {
    this.handlers.set(name, handler);
  }
}

const subsystemStatus = {
  capabilities: ["projects", "timeline"],
  ready: true,
  subsystems: {
    editor: {
      capabilities: ["projects", "timeline"],
      error: null,
      ready: true,
    },
    rendering: {
      capabilities: [],
      error: {
        code: "DEPENDENCY_UNAVAILABLE",
        message: "rendering unavailable",
        retryable: false,
      },
      ready: false,
    },
  },
  version: "0.1.0",
};

const dependencies = () =>
  ({
    config: { projectsDirectory: "C:/projects" },
    headless: { call: () => Promise.resolve(subsystemStatus) },
    jobs: { cancel: () => ({}), get: () => ({}) },
    session: { activeProjectId: null },
    speech: {
      status: () => Promise.reject(new Error("C:/private/provider.log")),
    },
  }) as unknown as ServerDependencies;

describe("capability registrar architecture", () => {
  it("registers every capability independently with injected dependencies", () => {
    const harness = new RegistrationHarness();
    const server = harness as unknown as Server;
    const injected = dependencies();
    registerProjectTools(server, injected);
    registerTimelineTools(server, injected);
    registerRenderTools(server, injected);
    registerSpeechTools(server, injected);
    registerJobTools(server, injected);

    expect([...harness.handlers.keys()].sort()).toEqual(
      expect.arrayContaining([
        "asset_delete",
        "editor_get_status",
        "job_cancel",
        "preview_render_frame",
        "project_create",
        "project_export_video",
        "timeline_add_media",
        "tts_generate_and_insert",
      ])
    );
  });

  it("keeps overall editor health ready when optional subsystems are degraded", async () => {
    const harness = new RegistrationHarness();
    registerProjectTools(harness as unknown as Server, dependencies());
    const response = await harness.handlers.get("editor_get_status")?.({});
    const structured = (
      response as { structuredContent: Record<string, unknown> }
    ).structuredContent;
    expect(structured.ready).toBe(true);
    expect(structured.capabilities).toEqual(["projects", "timeline"]);
    expect(structured.subsystems).toMatchObject({
      rendering: { ready: false },
      speech: { ready: false },
    });
  });
});

it("enumerates every Rust headless request discriminant", () => {
  const operations = {
    commit_draft: true,
    commit_generated_asset: true,
    commit_transcription: true,
    create_draft: true,
    create_project: true,
    delete_asset: true,
    discard_draft: true,
    edit: true,
    edit_batch: true,
    export_video: true,
    get_draft: true,
    get_draft_state: true,
    get_state: true,
    import_asset: true,
    list_projects: true,
    open_project: true,
    rebase_draft: true,
    redo: true,
    render_draft_preview: true,
    render_preview: true,
    render_preview_range: true,
    replace_generated_asset: true,
    resolve_asset_input: true,
    status: true,
    undo: true,
    update_draft: true,
  } satisfies Record<HeadlessRequest["operation"], true>;
  expect(Object.keys(operations)).toHaveLength(26);
});
