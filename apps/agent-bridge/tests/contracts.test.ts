import { existsSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import OWNERSHIP from "../../../contracts/contract-ownership-v1.json";
import ERROR_CATALOG from "../../../contracts/error-codes-v1.json";
import HEADLESS_CONTRACT from "../../../contracts/headless-protocol-v1.json";
import MCP_SURFACE from "../../../contracts/mcp-surface-v1.json";
import SPEECH_CONTRACT from "../../../contracts/speech-provider-v1.json";
import TRANSCRIPTION_CONTRACT from "../../../contracts/transcription-provider-v1.json";
import { retryableFor } from "../src/errors";
import type { HeadlessRequest } from "../src/headless-contract";
import { headlessStatusSchema, schemas, ttsStatusSchema } from "../src/schemas";
import {
  MCP_RESOURCE_URIS,
  registerContextResources,
  registerWorkflowPrompts,
  WORKFLOW_PROMPT_NAMES,
} from "../src/server/context";
import { registerDraftTools } from "../src/server/drafts";
import { registerJobTools } from "../src/server/jobs";
import { registerProjectTools } from "../src/server/projects";
import { registerRenderTools } from "../src/server/render";
import type { Server, ServerDependencies } from "../src/server/shared";
import { registerSpeechTools } from "../src/server/speech";
import { registerTimelineTools } from "../src/server/timeline";
import { registerTranscriptionTools } from "../src/server/transcription";

class ContractHarness {
  readonly prompts = new Set<string>();
  readonly resources = new Set<string>();
  readonly tools = new Set<string>();

  registerPrompt(name: string) {
    this.prompts.add(name);
  }

  registerResource(name: string) {
    this.resources.add(name);
  }

  registerTool(name: string) {
    this.tools.add(name);
  }
}

const dependencies = {
  config: {},
  headless: {},
  jobs: {},
  session: { activeProjectId: null },
  speech: {},
  transcription: {},
} as unknown as ServerDependencies;

describe("canonical public contracts", () => {
  it("keeps every canonical owner and governed consumer checked in", () => {
    const repositoryRoot = resolve(import.meta.dirname, "../../..");
    expect(OWNERSHIP.strategy).toBe("fixture-governed-manual-synchronization");
    expect(OWNERSHIP.reviewer).toBe("@matiHirCab");
    for (const [category, ownership] of Object.entries(OWNERSHIP.categories)) {
      expect(
        existsSync(resolve(repositoryRoot, ownership.canonical)),
        `${category} canonical owner is missing: ${ownership.canonical}`
      ).toBe(true);
      for (const consumer of ownership.consumers) {
        expect(
          existsSync(resolve(repositoryRoot, consumer)),
          `${category} consumer is missing: ${consumer}`
        ).toBe(true);
      }
    }
  });

  it("validates canonical status negotiation in TypeScript and Zod", () => {
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
    expect(Object.keys(operations).sort()).toEqual(
      HEADLESS_CONTRACT.operations
    );

    const defaultRequest = HEADLESS_CONTRACT.requests
      .statusDefault as HeadlessRequest;
    const currentRequest = HEADLESS_CONTRACT.requests
      .statusCurrent as HeadlessRequest;
    expect(defaultRequest).toEqual({ operation: "status" });
    expect(currentRequest).toEqual({ operation: "status", protocolVersion: 1 });
    expect(schemas.editorGetStatus.parse({})).toEqual({});
    expect(schemas.editorGetStatus.parse({ protocolVersion: 1 })).toEqual({
      protocolVersion: 1,
    });
    expect(
      schemas.editorGetStatus.safeParse({ protocolVersion: 2 }).success
    ).toBe(false);

    const status = headlessStatusSchema.parse({
      capabilities: HEADLESS_CONTRACT.status.editorCapabilities,
      protocolVersion: HEADLESS_CONTRACT.version,
      ready: true,
      subsystems: {
        editor: {
          capabilities: HEADLESS_CONTRACT.status.editorCapabilities,
          error: null,
          ready: true,
        },
        rendering: {
          capabilities: HEADLESS_CONTRACT.status.renderingCapabilities,
          error: null,
          ready: true,
        },
      },
      version: "0.1.0",
    });
    expect(status.protocolVersion).toBe(HEADLESS_CONTRACT.version);
    expect(Object.keys(status)).toEqual(
      expect.arrayContaining(HEADLESS_CONTRACT.status.requiredFields)
    );
  });

  it("keeps stable errors and provider versions aligned", () => {
    for (const [code, definition] of Object.entries(ERROR_CATALOG.codes)) {
      expect(retryableFor(code), `${code} retryability drifted`).toBe(
        definition.retryable
      );
    }
    expect(ttsStatusSchema.parse(SPEECH_CONTRACT.status).version).toBe("1.0");
    expect(TRANSCRIPTION_CONTRACT.version).toBe("transcription-provider-v1");
  });

  it("registers exactly the canonical MCP tools, resources, and prompts", () => {
    const harness = new ContractHarness();
    const server = harness as unknown as Server;
    registerProjectTools(server, dependencies);
    registerTimelineTools(server, dependencies);
    registerRenderTools(server, dependencies);
    registerSpeechTools(server, dependencies);
    registerJobTools(server, dependencies);
    registerDraftTools(server, dependencies);
    registerTranscriptionTools(server, dependencies);
    registerContextResources(server, dependencies);
    registerWorkflowPrompts(server);

    expect([...harness.tools].sort()).toEqual(MCP_SURFACE.tools);
    expect([...harness.resources].sort()).toEqual(
      MCP_SURFACE.resources.map((resource) => resource.name)
    );
    expect([...harness.prompts].sort()).toEqual(MCP_SURFACE.prompts);
    expect(Object.values(MCP_RESOURCE_URIS).sort()).toEqual(
      MCP_SURFACE.resources.map((resource) => resource.uriTemplate).sort()
    );
    expect([...WORKFLOW_PROMPT_NAMES]).toEqual(MCP_SURFACE.prompts);
  });
});
