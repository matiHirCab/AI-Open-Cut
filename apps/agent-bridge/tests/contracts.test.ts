import { existsSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";
import { z } from "zod/v4";
import OWNERSHIP from "../../../contracts/contract-ownership-v1.json";
import ERROR_CATALOG from "../../../contracts/error-codes-v1.json";
import HEADLESS_CONTRACT from "../../../contracts/headless-protocol-v1.json";
import MCP_SURFACE from "../../../contracts/mcp-surface-v1.json";
import MOTION_GRAPHICS_CONTRACT from "../../../contracts/motion-graphics-v1.json";
import SPEECH_CONTRACT from "../../../contracts/speech-provider-v1.json";
import STACKING from "../../../contracts/stacking-v1.json";
import TRANSCRIPTION_CONTRACT from "../../../contracts/transcription-provider-v1.json";
import AGENT_BRIDGE_PACKAGE from "../package.json";
import { retryableFor } from "../src/errors";
import type { HeadlessRequest } from "../src/headless-contract";
import { EVALUATED_SCENE_RENDERING_CAPABILITY } from "../src/headless-contract";
import {
  headlessEditSchema,
  headlessStatusSchema,
  schemas,
  ttsStatusSchema,
} from "../src/schemas";
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
import {
  assertMalformedPayloadRegressions,
  validateMotionGraphicsCatalog as validateStrictMotionGraphicsCatalog,
} from "./fixtures/motion-graphics-contract";

const TYPECHECK_GATE_PREFIX = /^bun run typecheck && /;

class ContractHarness {
  readonly prompts = new Set<string>();
  readonly resources = new Set<string>();
  readonly tools = new Set<string>();
  readonly toolDefinitions = new Map<string, ToolDefinition>();

  registerPrompt(name: string) {
    this.prompts.add(name);
  }

  registerResource(name: string) {
    this.resources.add(name);
  }

  registerTool(name: string, definition: ToolDefinition) {
    this.tools.add(name);
    this.toolDefinitions.set(name, definition);
  }
}

interface ToolDefinition {
  annotations: Record<string, unknown>;
  inputSchema: z.ZodType;
  outputSchema: z.ZodType;
}

const normalizeJson = (
  value: unknown,
  omitSchemaDescriptions = false
): unknown => {
  if (Array.isArray(value)) {
    return value.map((child) => normalizeJson(child, omitSchemaDescriptions));
  }
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .filter(([key]) => !(omitSchemaDescriptions && key === "description"))
        .sort(([left], [right]) => left.localeCompare(right))
        .map(([key, child]) => [
          key,
          normalizeJson(child, omitSchemaDescriptions),
        ])
    );
  }
  return value;
};

const normalizeSchemaJson = (value: unknown) => normalizeJson(value, true);

const schemaJson = (schema: z.ZodType, io: "input" | "output") =>
  normalizeSchemaJson(
    z.toJSONSchema(schema, {
      io,
      target: "draft-2020-12",
      unrepresentable: "throw",
    })
  );

const canonicalToolDefinitions = (harness: ContractHarness) =>
  Object.fromEntries(
    [...harness.toolDefinitions.entries()]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([name, definition]) => [
        name,
        {
          annotations: normalizeJson(definition.annotations),
          inputSchema: schemaJson(definition.inputSchema, "input"),
          outputSchema: schemaJson(definition.outputSchema, "output"),
        },
      ])
  );

const mismatchedToolDefinitions = (
  actual: Record<string, unknown>,
  expected: Record<string, unknown>
) =>
  [...new Set([...Object.keys(actual), ...Object.keys(expected)])]
    .sort()
    .filter(
      (name) => JSON.stringify(actual[name]) !== JSON.stringify(expected[name])
    );

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

  it("accepts the complete and safe canonical motion-graphics fixture catalog", () => {
    expect(() =>
      validateStrictMotionGraphicsCatalog(MOTION_GRAPHICS_CONTRACT)
    ).not.toThrow();
  });

  it("rejects malformed payloads, unsafe resource fields, and scope drift", () => {
    expect(() =>
      assertMalformedPayloadRegressions(MOTION_GRAPHICS_CONTRACT)
    ).not.toThrow();
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
    expect(status.subsystems.rendering.capabilities).toContain(
      EVALUATED_SCENE_RENDERING_CAPABILITY
    );
    expect(MCP_SURFACE.capabilityIdentifiers).toEqual([
      EVALUATED_SCENE_RENDERING_CAPABILITY,
    ]);
    expect(Object.keys(status)).toEqual(
      expect.arrayContaining(HEADLESS_CONTRACT.status.requiredFields)
    );
  });

  it("keeps TypeScript checking in the standalone contract gate", () => {
    expect(AGENT_BRIDGE_PACKAGE.scripts["contracts:check"]).toMatch(
      TYPECHECK_GATE_PREFIX
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

  it("registers exactly the canonical MCP definitions and supporting surfaces", () => {
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

    const registeredTools = [...harness.tools].sort();
    const registeredDefinitions = canonicalToolDefinitions(harness);
    expect(registeredTools).toEqual(MCP_SURFACE.tools);
    expect(Object.keys(registeredDefinitions)).toEqual(MCP_SURFACE.tools);
    expect(Object.keys(MCP_SURFACE.toolDefinitions).sort()).toEqual(
      MCP_SURFACE.tools
    );
    expect(
      mismatchedToolDefinitions(
        registeredDefinitions,
        MCP_SURFACE.toolDefinitions
      )
    ).toEqual([]);
    expect(registeredDefinitions).toEqual(MCP_SURFACE.toolDefinitions);
    expect([...harness.resources].sort()).toEqual(
      MCP_SURFACE.resources.map((resource) => resource.name)
    );
    expect([...harness.prompts].sort()).toEqual(MCP_SURFACE.prompts);
    expect(Object.values(MCP_RESOURCE_URIS).sort()).toEqual(
      MCP_SURFACE.resources.map((resource) => resource.uriTemplate).sort()
    );
    expect([...WORKFLOW_PROMPT_NAMES]).toEqual(MCP_SURFACE.prompts);
  });

  it("detects input, output, and annotation drift in MCP definitions", () => {
    const catalog = structuredClone(MCP_SURFACE.toolDefinitions);
    const toolName = "asset_delete";
    const definition = catalog[toolName];

    const inputDrift = {
      ...catalog,
      [toolName]: {
        ...definition,
        inputSchema: { ...definition.inputSchema, title: "drift" },
      },
    };
    const outputDrift = {
      ...catalog,
      [toolName]: {
        ...definition,
        outputSchema: { ...definition.outputSchema, title: "drift" },
      },
    };
    const annotationDrift = {
      ...catalog,
      [toolName]: {
        ...definition,
        annotations: { ...definition.annotations, readOnlyHint: "drift" },
      },
    };

    expect(mismatchedToolDefinitions(inputDrift, catalog)).toEqual([toolName]);
    expect(mismatchedToolDefinitions(outputDrift, catalog)).toEqual([toolName]);
    expect(mismatchedToolDefinitions(annotationDrift, catalog)).toEqual([
      toolName,
    ]);
  });

  it("excludes schema descriptions without hiding structural drift", () => {
    const structuralSchema = z.object({
      nested: z.object({ value: z.string().min(1) }),
    });
    const describedSchema = z
      .object({
        nested: z
          .object({ value: z.string().min(1).describe("value copy") })
          .describe("nested copy"),
      })
      .describe("root copy");
    const changedSchema = z.object({
      nested: z.object({ value: z.string().min(2) }),
    });

    for (const io of ["input", "output"] as const) {
      expect(schemaJson(describedSchema, io)).toEqual(
        schemaJson(structuralSchema, io)
      );
      expect(schemaJson(changedSchema, io)).not.toEqual(
        schemaJson(structuralSchema, io)
      );
    }
    expect(normalizeJson({ description: "annotation data" })).toEqual({
      description: "annotation data",
    });
  });
});

describe("runtime stacking contract", () => {
  it("uses canonical strict payloads for standalone and batch schemas", () => {
    for (const value of STACKING.valid) {
      expect(headlessEditSchema.parse(value)).toEqual(value);
      const { operation, ...input } = value;
      const schema = {
        item_reorder: schemas.itemReorder,
        item_set_z_index: schemas.itemSetZIndex,
        track_reorder: schemas.trackReorder,
      }[operation as "item_set_z_index" | "item_reorder" | "track_reorder"];
      expect(
        schema.safeParse({
          ...input,
          expectedRevision: 0,
          projectId: "project",
        }).success
      ).toBe(true);
    }
    for (const value of STACKING.invalid) {
      expect(
        headlessEditSchema.safeParse(value).success,
        JSON.stringify(value)
      ).toBe(false);
    }
  });
});
