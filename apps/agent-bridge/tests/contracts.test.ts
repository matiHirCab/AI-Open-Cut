import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";
import { z } from "zod/v4";
import COMPONENTS from "../../../contracts/component-definitions-v1.json";
import OWNERSHIP from "../../../contracts/contract-ownership-v1.json";
import ERROR_CATALOG from "../../../contracts/error-codes-v1.json";
import GROUPS from "../../../contracts/group-parent-v1.json";
import HEADLESS_CONTRACT from "../../../contracts/headless-protocol-v1.json";
import MCP_SURFACE from "../../../contracts/mcp-surface-v1.json";
import MOTION_GRAPHICS_CONTRACT from "../../../contracts/motion-graphics-v1.json";
import SPEECH_CONTRACT from "../../../contracts/speech-provider-v1.json";
import STACKING from "../../../contracts/stacking-v1.json";
import type SLOT_CATALOG from "../../../contracts/template-slots-v1.json";
import TRANSCRIPTION_CONTRACT from "../../../contracts/transcription-provider-v1.json";
import AGENT_BRIDGE_PACKAGE from "../package.json";
import { retryableFor } from "../src/errors";
import type { HeadlessRequest } from "../src/headless-contract";
import { EVALUATED_SCENE_RENDERING_CAPABILITY } from "../src/headless-contract";
import {
  closedSlotRecord,
  componentDefinitionSchema,
  componentInstanceSchema,
  headlessEditSchema,
  headlessStatusSchema,
  schemas,
  slotValueSchema,
  templateSlotSchema,
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

it("matches canonical component item structural acceptance independently of core semantics", () => {
  for (const fixture of COMPONENTS.itemValidationFixtures) {
    const { operation: _operation, ...fields } = fixture.operation;
    expect(
      headlessEditSchema.safeParse(fixture.operation).success,
      fixture.id
    ).toBe(fixture.mcpAccept);
    expect(
      schemas.componentCreate.safeParse({
        ...fields,
        expectedRevision: 0,
        projectId: "project",
      }).success,
      fixture.id
    ).toBe(fixture.mcpAccept);
    expect(
      schemas.componentUpdate.safeParse({
        ...fields,
        componentId: "component",
        expectedRevision: 0,
        projectId: "project",
      }).success,
      fixture.id
    ).toBe(fixture.mcpAccept);
    expect(
      schemas.timelineBatchEdit.safeParse({
        expectedRevision: 0,
        operations: [fixture.operation],
        projectId: "project",
      }).success,
      fixture.id
    ).toBe(fixture.mcpAccept);
  }
});

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

// Parse the JSON as data: bundlers may emit __proto__ as object-literal syntax.
const SLOTS: typeof SLOT_CATALOG = JSON.parse(
  readFileSync(
    new URL("../../../contracts/template-slots-v1.json", import.meta.url),
    "utf8"
  )
);
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
  it("validates canonical component operations standalone and in batches", () => {
    for (const fixture of COMPONENTS.semanticFixtures) {
      for (const definition of fixture.components) {
        expect(componentDefinitionSchema.safeParse(definition).success).toBe(
          true
        );
      }
    }
    for (const value of COMPONENTS.validOperations) {
      expect(headlessEditSchema.safeParse(value).success).toBe(true);
      expect(
        schemas.timelineBatchEdit.safeParse({
          expectedRevision: 0,
          operations: [value],
          projectId: "project",
        }).success
      ).toBe(true);
    }
    for (const value of COMPONENTS.invalidOperations) {
      expect(headlessEditSchema.safeParse(value).success).toBe(false);
      expect(
        schemas.timelineBatchEdit.safeParse({
          expectedRevision: 0,
          operations: [value],
          projectId: "project",
        }).success
      ).toBe(false);
    }
  });
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

describe("runtime group contract", () => {
  it("leaves canonical graph failures to core semantic validation", () => {
    for (const fixture of GROUPS.graphFailures) {
      const edges =
        "parents" in fixture ? fixture.parents : [["child", "group"]];
      for (const [itemId, id] of edges) {
        expect(
          headlessEditSchema.safeParse({
            itemId,
            operation: "item_set_parent",
            parent: { id, scope: "scope" in fixture ? fixture.scope : "root" },
          }).success,
          fixture.id
        ).toBe(true);
      }
    }
  });
  it("accepts canonical typed standalone and batch inputs and rejects malformed fields", () => {
    for (const fixture of GROUPS.valid) {
      expect(
        headlessEditSchema.safeParse(fixture.value).success,
        fixture.id
      ).toBe(true);
      const { operation, ...input } = fixture.value;
      const schema = {
        add_group: schemas.addGroup,
        group_ungroup: schemas.groupUngroup,
        item_set_parent: schemas.itemSetParent,
      }[operation];
      if (!schema) {
        throw new Error(`Unknown group operation: ${operation}`);
      }
      expect(
        schema.safeParse({
          expectedRevision: 0,
          projectId: "project",
          ...input,
        }).success,
        fixture.id
      ).toBe(true);
    }
    for (const fixture of GROUPS.invalid) {
      if (fixture.value.operation === "group_ungroup") {
        expect(
          schemas.timelineBatchEdit.safeParse({
            expectedRevision: 0,
            operations: [fixture.value],
            projectId: "project",
          }).success,
          fixture.id
        ).toBe(false);
        const { operation: _operation, ...input } = fixture.value;
        expect(
          schemas.groupUngroup.safeParse({
            expectedRevision: 0,
            projectId: "project",
            ...input,
          }).success,
          fixture.id
        ).toBe(false);
      }
      expect(
        headlessEditSchema.safeParse(fixture.value).success,
        fixture.id
      ).toBe(false);
    }
  });
});

it("matches runtime slot fixtures and closed typed values", () => {
  for (const fixture of SLOTS.valid) {
    expect(templateSlotSchema.safeParse(fixture.slot).success, fixture.id).toBe(
      true
    );
    const operation = {
      componentId: "card",
      operation: "component_define_slots",
      slots: [fixture.slot],
    };
    expect(headlessEditSchema.safeParse(operation).success).toBe(true);
    expect(
      schemas.componentDefineSlots.safeParse({
        componentId: "card",
        expectedRevision: 0,
        projectId: "project",
        slots: [fixture.slot],
      }).success
    ).toBe(true);
  }
  for (const fixture of SLOTS.invalid) {
    expect(templateSlotSchema.safeParse(fixture.slot).success, fixture.id).toBe(
      fixture.stage !== "structural"
    );
  }
  for (const value of [
    null,
    { type: "text", value: true },
    { extra: 1, type: "text", value: "x" },
    { type: "duration", value: 0.5 },
    { type: "text", value: "\ud800" },
    {
      type: "rich_text",
      value: { runs: [{ href: "https://example.org", text: "x" }] },
    },
  ]) {
    expect(slotValueSchema.safeParse(value).success).toBe(false);
  }
  expect(
    slotValueSchema.safeParse({ type: "text", value: "😀é" }).success
  ).toBe(true);
});

it("requires slot fields on schema-12 responses while retaining request defaults", () => {
  const definition = {
    ...COMPONENTS.definition,
    tracks: [
      {
        id: "local",
        items: [
          {
            componentId: "leaf",
            durationMs: 1000,
            hidden: false,
            id: "nested",
            stackOrder: 0,
            startMs: 0,
            timeScale: 1,
            transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
            trimStartMs: 0,
            type: "component_instance",
            zIndex: 0,
          },
        ],
        name: "Local",
        trackType: "overlay",
      },
    ],
  };
  expect(componentDefinitionSchema.safeParse(definition).success).toBe(false);
  expect(
    headlessEditSchema.safeParse({
      ...definition,
      id: undefined,
      operation: "component_create",
    }).success
  ).toBe(false);
  const { id: _id, ...fields } = definition;
  expect(
    headlessEditSchema.safeParse({ ...fields, operation: "component_create" })
      .success
  ).toBe(true);
  const complete = {
    ...definition,
    tracks: definition.tracks.map((track) => ({
      ...track,
      items: track.items.map((item) => ({ ...item, slotValues: {} })),
    })),
  };
  expect(componentDefinitionSchema.safeParse(complete).success).toBe(true);
  expect(
    componentDefinitionSchema.safeParse({ ...complete, slots: undefined })
      .success
  ).toBe(false);
});

it("preserves and validates every canonical special override key", () => {
  const schema = componentInstanceSchema.shape.slotValues;
  for (const input of [
    JSON.parse(JSON.stringify(SLOTS.regressions.overrides)),
    Object.assign(Object.create(null), SLOTS.regressions.overrides),
  ]) {
    const parsed = schema.parse(input);
    expect(parsed).toEqual(SLOTS.regressions.overrides);
    expect(Object.getPrototypeOf(parsed)).toBe(Object.prototype);
    expect(JSON.parse(JSON.stringify(parsed))).toEqual(input);
    for (const key of SLOTS.regressions.specialKeys) {
      expect(Object.hasOwn(parsed, key)).toBe(true);
      expect(parsed[key]).not.toBe(input[key]);
      for (const invalid of SLOTS.regressions.invalidValues) {
        const result = schema.safeParse(Object.fromEntries([[key, invalid]]));
        expect(result.success).toBe(false);
        if (!result.success) {
          expect(
            result.error.issues.every((issue) => issue.path[0] === key)
          ).toBe(true);
        }
      }
    }
  }
  const inherited = Object.create({ inherited: { type: "text", value: "x" } });
  expect(schema.parse(inherited)).toEqual({});
  for (const invalid of [null, [], 1, "map", undefined]) {
    expect(schema.safeParse(invalid).success).toBe(false);
  }
  const unknown = {
    [SLOTS.regressions.unknownSlotId]: { type: "text", value: "x" },
  };
  expect(schema.parse(unknown)).toEqual(unknown);
  for (const io of ["input", "output"] as const) {
    expect(z.toJSONSchema(schema, { io })).toEqual(
      z.toJSONSchema(z.record(z.string(), slotValueSchema), { io })
    );
  }
});

it("rejects canonical closed slot records with complete nested paths", () => {
  const locations = [
    "definition",
    "binding",
    "constraints",
    ...SLOTS.slotKinds.map((kind) => `envelope_${kind}`),
    "rich_text_document",
    "rich_text_run",
    "asset_reference",
  ];
  expect(
    SLOTS.regressions.closedRecords.map((fixture) => fixture.id).sort()
  ).toEqual(
    locations
      .flatMap((location) =>
        ["__proto__", "constructor", "toString", "unexpected"].map(
          (key) => `${location}_${key}`
        )
      )
      .sort()
  );
  const assertUnknown = (
    schema: z.ZodType,
    input: unknown,
    path: (string | number)[],
    key: string
  ) => {
    const result = schema.safeParse(input);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues).toContainEqual(
        expect.objectContaining({
          code: "unrecognized_keys",
          keys: [key],
          path,
        })
      );
    }
  };
  for (const fixture of SLOTS.regressions.closedRecords) {
    const record = fixture.recordPath.reduce<unknown>(
      (value, key) => (value as Record<string | number, unknown>)[key],
      fixture.slot
    );
    const bytes = JSON.stringify(fixture.slot);
    expect(Object.hasOwn(record as object, fixture.key), fixture.id).toBe(true);
    assertUnknown(
      templateSlotSchema,
      fixture.slot,
      fixture.recordPath,
      fixture.key
    );
    const request = {
      componentId: "card",
      expectedRevision: 0,
      projectId: "project",
      slots: [fixture.slot],
    };
    assertUnknown(
      schemas.componentDefineSlots,
      request,
      ["slots", 0, ...fixture.recordPath],
      fixture.key
    );
    const operation = {
      componentId: "card",
      operation: "component_define_slots",
      slots: [fixture.slot],
    };
    assertUnknown(
      headlessEditSchema,
      operation,
      ["slots", 0, ...fixture.recordPath],
      fixture.key
    );
    assertUnknown(
      schemas.timelineBatchEdit,
      {
        expectedRevision: 0,
        operations: [operation],
        projectId: "project",
      },
      ["operations", 0, "slots", 0, ...fixture.recordPath],
      fixture.key
    );
    assertUnknown(
      componentDefinitionSchema,
      { ...COMPONENTS.definition, slots: [fixture.slot] },
      ["slots", 0, ...fixture.recordPath],
      fixture.key
    );
    if (fixture.overridePath) {
      assertUnknown(
        slotValueSchema,
        fixture.override,
        fixture.overridePath,
        fixture.key
      );
      for (const id of SLOTS.regressions.specialKeys) {
        const values = Object.fromEntries([[id, fixture.override]]);
        assertUnknown(
          componentInstanceSchema.shape.slotValues,
          values,
          [id, ...fixture.overridePath],
          fixture.key
        );
        const item = {
          componentId: "leaf",
          durationMs: 1000,
          hidden: false,
          id: "nested",
          slotValues: values,
          stackOrder: 0,
          startMs: 0,
          timeScale: 1,
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
          trimStartMs: 0,
          type: "component_instance",
          zIndex: 0,
        };
        assertUnknown(
          componentInstanceSchema,
          item,
          ["slotValues", id, ...fixture.overridePath],
          fixture.key
        );
        const fields = {
          durationMs: 1000,
          height: 240,
          name: "Outer",
          tracks: [
            { id: "local", items: [item], name: "Local", trackType: "overlay" },
          ],
          width: 320,
        };
        assertUnknown(
          schemas.componentCreate,
          { ...fields, expectedRevision: 0, projectId: "project" },
          ["tracks", 0, "items", 0, "slotValues", id, ...fixture.overridePath],
          fixture.key
        );
        assertUnknown(
          schemas.timelineBatchEdit,
          {
            expectedRevision: 0,
            operations: [{ ...fields, operation: "component_create" }],
            projectId: "project",
          },
          [
            "operations",
            0,
            "tracks",
            0,
            "items",
            0,
            "slotValues",
            id,
            ...fixture.overridePath,
          ],
          fixture.key
        );
      }
    }
    expect(JSON.stringify(fixture.slot)).toBe(bytes);
    expect(Object.getPrototypeOf(record)).toBe(Object.prototype);
  }
});

it("delegates closed record parsing without changing types or JSON schemas", () => {
  const original = z
    .object({ count: z.number().optional(), text: z.string() })
    .strict();
  const guarded = closedSlotRecord(original, Object.keys(original.shape));
  const input = { text: "Hello" };
  const parsed: z.infer<typeof original> = guarded.parse(input);
  expect(parsed).toEqual(input);
  expect(parsed).not.toBe(input);
  for (const io of ["input", "output"] as const) {
    expect(z.toJSONSchema(guarded, { io })).toEqual(
      z.toJSONSchema(original, { io })
    );
  }
  for (const value of [null, [], 3, { text: false }]) {
    expect(guarded.safeParse(value).error?.issues).toEqual(
      original.safeParse(value).error?.issues
    );
  }
  const valueError = componentInstanceSchema.shape.slotValues.safeParse(
    JSON.parse(
      '{"__proto__":{"type":"rich_text","value":{"runs":[{"text":false}]}}}'
    )
  );
  expect(valueError.error?.issues[0]?.path).toEqual([
    "__proto__",
    "value",
    "runs",
    0,
    "text",
  ]);
});
