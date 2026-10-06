import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import type { ResourceTemplate } from "@modelcontextprotocol/server";
import { describe, expect, it } from "vitest";
import { z } from "zod/v4";
import ANIMATION_CHANNELS from "../../../contracts/animation-channels-v1.json";
import PRESETS from "../../../contracts/animation-presets-v1.json";
import ARTIFACT_DELIVERY from "../../../contracts/artifact-delivery-v2.json";
import COMPONENTS from "../../../contracts/component-definitions-v1.json";
import INSTANCE_CATALOG from "../../../contracts/component-evaluation-v1.json";
import type LIFECYCLE_CATALOG from "../../../contracts/component-lifecycle-v1.json";
import OWNERSHIP from "../../../contracts/contract-ownership-v1.json";
import ERROR_CATALOG from "../../../contracts/error-codes-v1.json";
import EXTENDED_VISUAL from "../../../contracts/extended-visual-animation-v1.json";
import GROUPS from "../../../contracts/group-parent-v1.json";
import HEADLESS_CONTRACT from "../../../contracts/headless-protocol-v1.json";
import INHERITED_TIMING from "../../../contracts/inherited-animation-timing-v1.json";
import INITIAL_PRESET_PACK from "../../../contracts/initial-motion-preset-pack-v1.json";
import MASK_MODELS from "../../../contracts/mask-models-v1.json";
import MCP_SURFACE_SOURCE from "../../../contracts/mcp-surface-v1.json";
import MOTION_BLUR from "../../../contracts/motion-blur-sampling-v1.json";
import MOTION_GRAPHICS_CONTRACT from "../../../contracts/motion-graphics-v1.json";
import SPEECH_CONTRACT from "../../../contracts/speech-provider-v1.json";
import STACKING from "../../../contracts/stacking-v1.json";
import type SLOT_CATALOG from "../../../contracts/template-slots-v1.json";
import TRANSCRIPTION_CONTRACT from "../../../contracts/transcription-provider-v1.json";
import AGENT_BRIDGE_PACKAGE from "../package.json";
import { retryableFor } from "../src/errors";
import type { HeadlessRequest } from "../src/headless-contract";
import {
  EVALUATED_SCENE_RENDERING_CAPABILITY,
  LINEAR_LIGHT_COMPOSITING_CAPABILITY,
  MASK_ANIMATION_CAPABILITY,
  MASK_MODELS_CAPABILITY,
  MASK_RENDERING_CAPABILITY,
  MATTE_MODELS_CAPABILITY,
  TRACK_MATTES_CAPABILITY,
} from "../src/headless-contract";
import {
  closedSlotRecord,
  componentDefinitionSchema,
  componentInstanceSchema,
  headlessEditSchema,
  headlessStatusSchema,
  maskSchema,
  schemas,
  slotValueSchema,
  statusSchema,
  synthesizedSpeechMetadataSchema,
  templateSlotSchema,
  timeExpressionSchema,
  ttsStatusSchema,
} from "../src/schemas";
import { ARTIFACT_RESOURCES_CAPABILITY } from "../src/server/artifacts";
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
  projectBlendCatalogPredecessor,
  projectBlendMcpPredecessor,
} from "./fixtures/blend-mode-projection";
import { projectMaskMcpPredecessor } from "./fixtures/mask-mcp-projection";
import {
  MASK_RENDERING_PREDECESSOR_PINS,
  projectMaskRenderingCatalogPredecessor as projectMaskRenderingCatalogPredecessor51,
  projectMaskRenderingMcpPredecessor as projectMaskRenderingMcpPredecessor51,
} from "./fixtures/mask-rendering-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";
import {
  assertMalformedPayloadRegressions,
  validateMotionGraphicsCatalog as validateStrictMotionGraphicsCatalog,
} from "./fixtures/motion-graphics-contract";

import {
  projectTrackMatteCatalogPredecessor,
  projectTrackMatteMcpPredecessor,
} from "./fixtures/track-matte-projection";

// Keep earlier transition checks intact after the exact current34→33 projection.
const projectMaskRenderingCatalogPredecessor = (
  name: string,
  source: unknown
) =>
  projectMaskRenderingCatalogPredecessor51(
    name,
    projectTrackMatteCatalogPredecessor(
      name,
      projectBlendCatalogPredecessor(name, source)
    )
  );
const projectMaskRenderingMcpPredecessor = (source: unknown) =>
  projectMaskRenderingMcpPredecessor51(
    projectTrackMatteMcpPredecessor(projectBlendMcpPredecessor(source))
  );

const MCP_SURFACE = expandMcpSurfaceCatalog(MCP_SURFACE_SOURCE);
// Approved issue #49 additive capability; tool/schema baseline remains pinned below.
const MCP_BASELINE_DIGEST =
  "88b55ff7be147cb4aadc016dd92f92342c3dc366f49ac139b8116513d5854830";
const MCP_PRE_MASK_RENDERING_DIGEST = MCP_BASELINE_DIGEST;
const MCP_CURRENT_DIGEST =
  "0f413e33c216b52383e201880409d987c705b52d132e1da3b211dd355929737f";
const MCP_PRE_TRACK_MATTES_DIGEST =
  "803bf5954ebd4cb47be98dd87b4994e6d261eae20693199c0f569f535452f170";
const MCP_PRE_MASK_MODELS_DIGEST =
  "2a3fdf15aec5472e1cc2001f47659d4591de02e0f02bdeece3384584ea174503";
const MCP_PRE_LINEAR_COMPOSITION_DIGEST =
  "181c60179f3d9f329417093315b0b6058a9093fd8e1c23e06cee83c4c19b2620";

const LIFECYCLE: typeof LIFECYCLE_CATALOG = JSON.parse(
  readFileSync(
    new URL("../../../contracts/component-lifecycle-v1.json", import.meta.url),
    "utf8"
  )
);

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
  readonly resources = new Map<string, string>();
  readonly tools = new Set<string>();
  readonly toolDefinitions = new Map<string, ToolDefinition>();

  registerPrompt(name: string) {
    this.prompts.add(name);
  }

  registerResource(name: string, uri: string | ResourceTemplate) {
    this.resources.set(
      name,
      typeof uri === "string" ? uri : uri.uriTemplate.toString()
    );
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

// Independently captured from git show b0a9075f before marker edits. Codec:
// SHA-256 UTF-8 JSON.stringify(normalizeJson(catalog)); arrays/order/scalars and
// every description are retained; object keys use existing localeCompare sorting.
const ACTIVE_ANIMATION_CATALOGS = [
  {
    catalog: ANIMATION_CHANNELS,
    name: "animation-channels-v1",
    predecessorDigest:
      "91c032215f105029aa781ae138e0923eab34a803e91a196b02aae800e107c40e",
  },
  {
    catalog: PRESETS,
    name: "animation-presets-v1",
    predecessorDigest:
      "85f0a11e22a1eec5a1f8fab5c51aefec9ea17a89fa52b7ffb164419a1440cdf5",
  },
  {
    catalog: EXTENDED_VISUAL,
    name: "extended-visual-animation-v1",
    predecessorDigest:
      "f3fd62f54e773f3ad1645e7c33278cbf40409ede00a5161a37d7054d08f148f0",
  },
  {
    catalog: INHERITED_TIMING,
    name: "inherited-animation-timing-v1",
    predecessorDigest:
      "908cc19b80987306fada617513583e219088694d1b552da8d7a0d8fbfc85e098",
  },
  {
    catalog: MOTION_BLUR,
    name: "motion-blur-sampling-v1",
    predecessorDigest:
      "e07575ec3955710ab0e964afe95d81bf0bf602a8fab1d1d0c7e4f60a2dfe42d1",
  },
  {
    catalog: INITIAL_PRESET_PACK,
    name: "initial-motion-preset-pack-v1",
    predecessorDigest:
      "31dab37a41930a45e57da6c72ba4450d4730c97a7e21d0cabd0924d6bd825622",
  },
] as const;

const projectActiveAnimationCatalogPredecessor = (source: unknown) => {
  if (source === null || typeof source !== "object" || Array.isArray(source)) {
    throw new Error("Active animation catalog must be an object");
  }
  const previous = structuredClone(source) as Record<string, unknown>;
  if (previous.projectSchemaVersion !== 32) {
    throw new Error("Approved current animation catalog marker must be 32");
  }
  previous.projectSchemaVersion = 31;
  return previous;
};
const animationCatalogDigest = (catalog: unknown) =>
  createHash("sha256")
    .update(JSON.stringify(normalizeJson(catalog)), "utf8")
    .digest("hex");

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

const mismatchedSupportingSurfaces = (
  actual: typeof MCP_SURFACE,
  expected: typeof MCP_SURFACE
) =>
  (["capabilityIdentifiers", "prompts", "resources", "tools"] as const).filter(
    (key) => JSON.stringify(actual[key]) !== JSON.stringify(expected[key])
  );

interface MutableMcpSource {
  $defs: Record<string, Record<string, unknown>>;
  toolDefinitions: Record<
    string,
    {
      annotations: Record<string, unknown>;
      inputSchema: Record<string, unknown>;
      outputSchema: Record<string, unknown>;
    }
  >;
}

const mutableMcpSource = () =>
  structuredClone(MCP_SURFACE_SOURCE) as unknown as MutableMcpSource;

const mutableTool = (source: MutableMcpSource, name: string) => {
  const tool = source.toolDefinitions[name];
  if (!tool) {
    throw new Error(`Missing test tool: ${name}`);
  }
  return tool;
};

const dependencies = {
  config: {},
  headless: {},
  jobs: {},
  session: { activeProjectId: null },
  speech: {},
  transcription: {},
} as unknown as ServerDependencies;

describe("canonical public contracts", () => {
  it("consumes the independent animation preset contract in the canonical gate", () => {
    expect(headlessEditSchema.parse(PRESETS.examples.apply)).toEqual({
      ...PRESETS.examples.apply,
      collisionPolicy: "reject",
      parameters: { ...PRESETS.examples.apply.parameters, curve: "linear" },
    });
    expect(PRESETS.compilerVersion).toBe(2);
    expect(PRESETS.projectSchemaVersion).toBe(35);
    expect(PRESETS.examples.resolvedChannel.keyframes).toEqual([
      { curve: "linear", timeMs: 0, value: { type: "scalar", value: 0 } },
      { curve: "hold", timeMs: 500, value: { type: "scalar", value: 1 } },
    ]);
    expect(PRESETS.examples.retiredProvenance.presetVersion).toBeGreaterThan(0);
  });

  it("synchronizes six current animation markers with composed32 and31 predecessor semantics", () => {
    for (const {
      catalog,
      name,
      predecessorDigest,
    } of ACTIVE_ANIMATION_CATALOGS) {
      expect(catalog.projectSchemaVersion, name).toBe(35);
      const schema32 = projectMaskRenderingCatalogPredecessor(name, catalog);
      expect(animationCatalogDigest(schema32), name).toBe(
        MASK_RENDERING_PREDECESSOR_PINS[name]
      );
      const previous = projectActiveAnimationCatalogPredecessor(schema32);
      expect(previous, name).toEqual({ ...schema32, projectSchemaVersion: 31 });
      expect(animationCatalogDigest(previous), name).toBe(predecessorDigest);
      expect(() => projectActiveAnimationCatalogPredecessor(previous)).toThrow(
        "marker must be 32"
      );
    }
  });

  it("retains unrelated catalog drift and nested version markers in predecessor proofs", () => {
    for (const {
      catalog,
      name,
      predecessorDigest,
    } of ACTIVE_ANIMATION_CATALOGS) {
      const drift = {
        ...catalog,
        unauthorizedMarker: { projectSchemaVersion: 32 },
        version: "unapproved",
      };
      if (name === "extended-visual-animation-v1") {
        expect(() =>
          projectMaskRenderingCatalogPredecessor(name, drift)
        ).toThrow("Unrelated verified predecessor catalog drift");
        continue;
      }
      const previous = projectActiveAnimationCatalogPredecessor(
        projectMaskRenderingCatalogPredecessor(name, drift)
      );
      expect(previous.unauthorizedMarker, name).toEqual({
        projectSchemaVersion: 32,
      });
      expect(previous.version, name).toBe("unapproved");
      expect(animationCatalogDigest(previous), name).not.toBe(
        predecessorDigest
      );
    }
  });

  it("expands the approved additive MCP capability catalog deterministically", () => {
    const first = MCP_SURFACE;
    const second = expandMcpSurfaceCatalog(MCP_SURFACE_SOURCE);
    const firstSerialized = JSON.stringify(first);
    expect(firstSerialized).toBe(JSON.stringify(second));
    expect(Object.keys(first.toolDefinitions)).toHaveLength(78);
    expect(createHash("sha256").update(firstSerialized).digest("hex")).toBe(
      MCP_CURRENT_DIGEST
    );
    const schema34Source = projectBlendMcpPredecessor(MCP_SURFACE_SOURCE);
    expect(
      createHash("sha256")
        .update(JSON.stringify(expandMcpSurfaceCatalog(schema34Source)))
        .digest("hex")
    ).toBe("9d15133960b94806ee92cd9482d10c9481b902f957501fc4742c58b992d73949");
    const schema33Source = projectTrackMatteMcpPredecessor(schema34Source);
    const schema33 = expandMcpSurfaceCatalog(schema33Source);
    expect(
      createHash("sha256").update(JSON.stringify(schema33)).digest("hex")
    ).toBe(MCP_PRE_TRACK_MATTES_DIGEST);
    const schema32Source = projectMaskRenderingMcpPredecessor51(schema33Source);
    const schema32 = expandMcpSurfaceCatalog(schema32Source);
    expect(
      createHash("sha256").update(JSON.stringify(schema32)).digest("hex")
    ).toBe(MCP_PRE_MASK_RENDERING_DIGEST);
    // Issue #50 permits only the explicitly enumerated mask fields, definition,
    // editor capability and two schema-version literals in this projection.
    const previous = expandMcpSurfaceCatalog(
      projectMaskMcpPredecessor(schema32Source)
    );
    expect(
      createHash("sha256").update(JSON.stringify(previous)).digest("hex")
    ).toBe(MCP_PRE_MASK_MODELS_DIGEST);
    expect(
      previous.capabilityIdentifiers.filter(
        (capability) => capability === LINEAR_LIGHT_COMPOSITING_CAPABILITY
      )
    ).toHaveLength(1);
    previous.capabilityIdentifiers = previous.capabilityIdentifiers.filter(
      (capability) => capability !== LINEAR_LIGHT_COMPOSITING_CAPABILITY
    );
    expect(
      createHash("sha256").update(JSON.stringify(previous)).digest("hex")
    ).toBe(MCP_PRE_LINEAR_COMPOSITION_DIGEST);
  });

  it("copies repeated expanded references freshly without losing JSON values or order", () => {
    const source = {
      $defs: {
        Shared: {
          additionalProperties: false,
          metadata: Object.fromEntries([
            ["last", true],
            ["first", 7],
          ]),
          properties: { choices: { default: null, enum: ["first", "second"] } },
          type: "object",
        },
      },
      capabilityIdentifiers: [],
      prompts: [],
      resources: [],
      toolDefinitions: {
        example: {
          annotations: {},
          inputSchema: { $ref: "#/$defs/Shared" },
          outputSchema: { $ref: "#/$defs/Shared" },
        },
      },
      tools: ["example"],
      version: 1,
    };
    const original = JSON.stringify(source);
    const first = expandMcpSurfaceCatalog(source);
    const second = expandMcpSurfaceCatalog(source);
    expect(JSON.stringify(first)).toBe(JSON.stringify(second));
    const definition = first.toolDefinitions.example as NonNullable<
      typeof first.toolDefinitions.example
    >;
    expect(definition).toBeDefined();
    expect(JSON.stringify(definition.inputSchema)).toBe(
      JSON.stringify(source.$defs.Shared)
    );
    expect(definition.inputSchema).not.toBe(definition.outputSchema);
    const properties = definition.inputSchema.properties as Record<
      string,
      { enum: string[] }
    >;
    const choices = properties.choices as { enum: string[] };
    expect(choices).toBeDefined();
    choices.enum[0] = "changed";
    expect(JSON.stringify(definition.outputSchema)).toBe(
      JSON.stringify(source.$defs.Shared)
    );
    expect(JSON.stringify(second)).not.toBe(JSON.stringify(first));
    expect(JSON.stringify(source)).toBe(original);
  });

  it("preserves own keys, scalar identity and independent nested reference copies", () => {
    const shared = Object.fromEntries([
      ["last", { values: [null, false, "value", -0, { leaf: 7 }] }],
      ["__proto__", { data: ["literal"] }],
      ["first", true],
    ]);
    const source = {
      $defs: { Shared: shared },
      capabilityIdentifiers: [],
      prompts: [],
      resources: [],
      toolDefinitions: {
        example: {
          annotations: {},
          inputSchema: { $ref: "#/$defs/Shared" },
          outputSchema: { $ref: "#/$defs/Shared" },
        },
      },
      tools: ["example"],
      version: 1,
    };
    const original = JSON.stringify(source);
    const first = expandMcpSurfaceCatalog(source).toolDefinitions
      .example as NonNullable<typeof MCP_SURFACE.toolDefinitions.example>;
    const second = expandMcpSurfaceCatalog(source).toolDefinitions
      .example as NonNullable<typeof MCP_SURFACE.toolDefinitions.example>;
    expect(first).toBeDefined();
    expect(second).toBeDefined();
    const input = first.inputSchema;
    for (const copy of [input, first.outputSchema, second.inputSchema]) {
      expect(Object.keys(copy)).toEqual(["last", "__proto__", "first"]);
      expect(Object.getPrototypeOf(copy)).toBe(Object.prototype);
      expect(Object.getOwnPropertyDescriptor(copy, "__proto__")).toEqual({
        configurable: true,
        enumerable: true,
        value: { data: ["literal"] },
        writable: true,
      });
      expect(JSON.stringify(copy)).toBe(JSON.stringify(shared));
      const nested = copy.last as { values: unknown[] };
      expect(Object.is(nested.values[3], -0)).toBe(true);
      expect(copy).not.toBe(shared);
      expect(nested).not.toBe(shared.last);
    }
    const nested = input.last as { values: unknown[] };
    const sibling = first.outputSchema.last as { values: unknown[] };
    expect(input).not.toBe(first.outputSchema);
    expect(nested).not.toBe(sibling);
    expect(nested.values).not.toBe(sibling.values);
    expect(nested.values[4]).not.toBe(sibling.values[4]);
    (nested.values[4] as { leaf: number }).leaf = 99;
    nested.values[3] = 0;
    const ownData = Object.getOwnPropertyDescriptor(input, "__proto__")
      ?.value as { data: string[] };
    ownData.data[0] = "changed";
    expect(JSON.stringify(first.outputSchema)).toBe(JSON.stringify(shared));
    expect(JSON.stringify(second.inputSchema)).toBe(JSON.stringify(shared));
    expect(JSON.stringify(source)).toBe(original);
    expect(
      Object.is((shared.last as { values: unknown[] }).values[3], -0)
    ).toBe(true);
  });

  it("governs every mask model fixture in the canonical contract gate", () => {
    expect(MASK_MODELS.capability).toBe(MASK_MODELS_CAPABILITY);
    expect(HEADLESS_CONTRACT.status.editorCapabilities).toContain(
      MASK_MODELS_CAPABILITY
    );
    expect(HEADLESS_CONTRACT.status.renderingCapabilities).not.toContain(
      MASK_MODELS_CAPABILITY
    );
    for (const fixture of MASK_MODELS.cases) {
      expect(maskSchema.safeParse(fixture.value).success, fixture.name).toBe(
        fixture.accepted
      );
    }
  });

  it("preserves unauthorized matching fields and annotations in the exact predecessor projection", () => {
    const drift = mutableMcpSource();
    const input = mutableTool(drift, "editor_get_status").inputSchema;
    (input.properties as Record<string, unknown>).masks = { type: "string" };
    mutableTool(drift, "timeline_update_item").annotations.readOnlyHint = true;
    const projected = expandMcpSurfaceCatalog(
      projectMaskMcpPredecessor(projectMaskRenderingMcpPredecessor(drift))
    );
    expect(projected.toolDefinitions).toHaveProperty(
      "editor_get_status.inputSchema.properties.masks",
      { type: "string" }
    );
    expect(
      createHash("sha256").update(JSON.stringify(projected)).digest("hex")
    ).not.toBe(MCP_PRE_MASK_MODELS_DIGEST);
  });

  it("rejects missing, cyclic, malformed, non-local, and unused MCP definitions", () => {
    const missing = mutableMcpSource();
    mutableTool(missing, "asset_delete").inputSchema = {
      $ref: "#/$defs/MissingDefinition",
    };
    expect(() => expandMcpSurfaceCatalog(missing)).toThrow(
      "Missing MCP definition: MissingDefinition"
    );

    const cyclic = mutableMcpSource();
    cyclic.$defs.LoopA = { $ref: "#/$defs/LoopB" };
    cyclic.$defs.LoopB = { $ref: "#/$defs/LoopA" };
    mutableTool(cyclic, "asset_delete").inputSchema = {
      $ref: "#/$defs/LoopA",
    };
    expect(() => expandMcpSurfaceCatalog(cyclic)).toThrow(
      "Cyclic MCP definition"
    );

    for (const invalid of [
      { $ref: "https://example.com/schema" },
      { $ref: "#/$defs/Unknown/child" },
      { $ref: 42 },
    ]) {
      const source = mutableMcpSource();
      mutableTool(source, "asset_delete").inputSchema = invalid;
      expect(() => expandMcpSurfaceCatalog(source)).toThrow(
        "Malformed or non-local MCP reference"
      );
    }

    const sibling = mutableMcpSource();
    mutableTool(sibling, "asset_delete").inputSchema = {
      ...mutableTool(sibling, "asset_delete").inputSchema,
      $ref: "#/$defs/DraftCreateOutput",
    };
    expect(() => expandMcpSurfaceCatalog(sibling)).toThrow(
      "MCP reference must have no sibling fields"
    );

    const unused = mutableMcpSource();
    unused.$defs.UnusedSchema = { type: "object" };
    expect(() => expandMcpSurfaceCatalog(unused)).toThrow(
      "Unused MCP definitions: UnusedSchema"
    );
  });

  it("detects shared-definition and supporting-surface drift", () => {
    const source = mutableMcpSource();
    source.$defs.DraftCreateOutput = {
      ...source.$defs.DraftCreateOutput,
      title: "drift",
    };
    const drifted = expandMcpSurfaceCatalog(source);
    expect(
      mismatchedToolDefinitions(
        drifted.toolDefinitions,
        MCP_SURFACE.toolDefinitions
      )
    ).toEqual([
      "draft_create",
      "draft_discard",
      "draft_get",
      "draft_rebase",
      "draft_update",
    ]);

    for (const key of [
      "capabilityIdentifiers",
      "prompts",
      "resources",
      "tools",
    ] as const) {
      const changed = { ...MCP_SURFACE };
      changed[key] = [];
      expect(mismatchedSupportingSurfaces(changed, MCP_SURFACE)).toEqual([key]);
    }
    const resourceMappingDrift = { ...MCP_SURFACE };
    resourceMappingDrift.resources = resourceMappingDrift.resources.map(
      (resource, index) => ({
        ...resource,
        uriTemplate: index === 0 ? "opencut://wrong" : resource.uriTemplate,
      })
    );
    expect(
      mismatchedSupportingSurfaces(resourceMappingDrift, MCP_SURFACE)
    ).toEqual(["resources"]);
  });

  it("validates canonical component operations standalone and in batches", () => {
    for (const fixture of COMPONENTS.semanticFixtures) {
      for (const definition of fixture.components) {
        expect(
          componentDefinitionSchema.safeParse({ ...definition, markers: [] })
            .success
        ).toBe(true);
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
      render_review_range: true,
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
      projectSchemaVersion: 35,
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
      textLayoutVersion: 2,
      version: "0.1.0",
    });
    expect(status.protocolVersion).toBe(HEADLESS_CONTRACT.version);
    expect(status.subsystems.rendering.capabilities).toContain(
      EVALUATED_SCENE_RENDERING_CAPABILITY
    );
    expect(MCP_SURFACE.capabilityIdentifiers).toEqual([
      "marker_relative_timing",
      LIFECYCLE.capability,
      EVALUATED_SCENE_RENDERING_CAPABILITY,
      LINEAR_LIGHT_COMPOSITING_CAPABILITY,
      MASK_RENDERING_CAPABILITY,
      "shape_items",
      "shape_rendering",
      "svg_items",
      "svg_rendering",
      "grid_items",
      "grid_rendering",
      "repeater_items",
      "repeater_rendering",
      "rich_text_documents",
      "content_addressed_text_layout_v2",
      "styled_text_layers_v1",
      "advanced_text_layout_v1",
      "typed_animation_channels_v1",
      "deterministic_animation_curves_v1",
      "animation_loops_v1",
      "inherited_animation_timing_v1",
      "extended_visual_animation_v1",
      MASK_MODELS_CAPABILITY,
      MASK_ANIMATION_CAPABILITY,
      "motion_blur_sampling_v1",
      "animation_presets_v1",
      "initial_motion_preset_pack_v1",
      "preview_review_presets_v1",
      ARTIFACT_RESOURCES_CAPABILITY,
      MATTE_MODELS_CAPABILITY,
      TRACK_MATTES_CAPABILITY,
      "blend_models_v1",
      "blend_modes_v1",
    ]);
    expect(Object.keys(status)).toEqual(
      expect.arrayContaining(HEADLESS_CONTRACT.status.requiredFields)
    );
  });

  it.each([true, false])(
    "preserves readiness-gated linear composition in live MCP status (ready=%s)",
    async (ready) => {
      const renderingCapabilities = ready
        ? HEADLESS_CONTRACT.status.renderingCapabilities
        : [];
      const error = ready
        ? null
        : {
            code: "DEPENDENCY_UNAVAILABLE",
            message: "rendering unavailable",
            retryable: false,
          };
      const status = headlessStatusSchema.parse({
        capabilities: [
          ...HEADLESS_CONTRACT.status.editorCapabilities,
          ...renderingCapabilities,
        ],
        projectSchemaVersion: 35,
        protocolVersion: 1,
        ready: true,
        subsystems: {
          editor: {
            capabilities: HEADLESS_CONTRACT.status.editorCapabilities,
            error: null,
            ready: true,
          },
          rendering: { capabilities: renderingCapabilities, error, ready },
        },
        textLayoutVersion: 2,
        version: "0.1.0",
      });
      const handlers = new Map<
        string,
        (
          input: Record<string, unknown>
        ) => Promise<{ structuredContent: unknown }>
      >();
      const server = {
        registerTool: (
          name: string,
          _definition: unknown,
          registeredHandler: (
            input: Record<string, unknown>
          ) => Promise<{ structuredContent: unknown }>
        ) => handlers.set(name, registeredHandler),
      } as unknown as Server;
      const injected = {
        ...dependencies,
        headless: { call: () => Promise.resolve(status) },
      } as unknown as ServerDependencies;
      registerProjectTools(server, injected);
      const handler = handlers.get("editor_get_status");
      if (!handler) {
        throw new Error("Missing MCP status handler");
      }
      const response = await handler({ protocolVersion: 1 });
      const reported = statusSchema.parse(response.structuredContent);
      expect(reported.capabilities).toContain(MASK_MODELS_CAPABILITY);
      expect(reported.subsystems.editor.capabilities).toContain(
        MASK_MODELS_CAPABILITY
      );
      expect(reported.subsystems.editor.capabilities).toContain(
        MATTE_MODELS_CAPABILITY
      );
      expect(reported.subsystems.editor.capabilities).not.toContain(
        TRACK_MATTES_CAPABILITY
      );
      expect(reported.subsystems.rendering.capabilities).not.toContain(
        MATTE_MODELS_CAPABILITY
      );
      expect(
        reported.subsystems.rendering.capabilities.filter(
          (capability) => capability === TRACK_MATTES_CAPABILITY
        )
      ).toHaveLength(ready ? 1 : 0);
      expect(reported.ready).toBe(true);
      expect(reported.protocolVersion).toBe(1);
      expect(reported.subsystems.rendering).toEqual(
        status.subsystems.rendering
      );
      expect(reported.capabilities).toEqual([
        ...status.capabilities,
        ARTIFACT_RESOURCES_CAPABILITY,
      ]);
      for (const capabilities of [
        reported.capabilities,
        reported.subsystems.rendering.capabilities,
      ]) {
        expect(
          capabilities.filter(
            (capability) => capability === LINEAR_LIGHT_COMPOSITING_CAPABILITY
          )
        ).toHaveLength(ready ? 1 : 0);
      }
      expect(MCP_SURFACE.capabilityIdentifiers).toContain(
        LINEAR_LIGHT_COMPOSITING_CAPABILITY
      );
    }
  );

  it("governs the version-2 artifact content policy independently of headless", () => {
    expect(ARTIFACT_DELIVERY.version).toBe(2);
    expect(ARTIFACT_DELIVERY.capability).toBe(ARTIFACT_RESOURCES_CAPABILITY);
    expect(ARTIFACT_DELIVERY.resourceTemplate).toBe(
      MCP_RESOURCE_URIS.jobArtifact
    );
    expect(ARTIFACT_DELIVERY.defaultContentTypes).toEqual([
      "text",
      "resource_link",
    ]);
    expect(
      schemas.jobGetStatus.safeParse({ includeBinary: true, jobId: "job" })
        .success
    ).toBe(true);
    expect(
      schemas.jobGetStatus.safeParse({ includeBinary: "true", jobId: "job" })
        .success
    ).toBe(false);
    expect(ARTIFACT_DELIVERY.headlessProtocolChanged).toBe(false);
    expect(ARTIFACT_DELIVERY.persistedSchemaChanged).toBe(false);
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

  it("governs independent timestamp support and strict legacy compatibility", () => {
    const parsed = ttsStatusSchema.parse(SPEECH_CONTRACT.status);
    expect(parsed.timestampSupport).toEqual(
      SPEECH_CONTRACT.status.timestampSupport
    );
    for (const timestampSupport of SPEECH_CONTRACT.timestampSupportCases
      .valid) {
      expect(
        ttsStatusSchema.parse({ ...SPEECH_CONTRACT.status, timestampSupport })
          .timestampSupport
      ).toEqual(timestampSupport);
    }
    const { timestampSupport: _support, ...legacy } = SPEECH_CONTRACT.status;
    expect(ttsStatusSchema.parse(legacy).timestampSupport).toEqual(
      SPEECH_CONTRACT.timestampSupportCases.unsupported
    );
    for (const timestampSupport of SPEECH_CONTRACT.timestampSupportCases
      .invalid) {
      expect(
        ttsStatusSchema.safeParse({
          ...SPEECH_CONTRACT.status,
          timestampSupport,
        }).success
      ).toBe(false);
    }
    expect(
      synthesizedSpeechMetadataSchema.parse(SPEECH_CONTRACT.synthesis)
    ).toEqual(SPEECH_CONTRACT.synthesis);
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
    expect(
      [...harness.resources]
        .map(([name, uriTemplate]) => ({
          name,
          uriTemplate,
        }))
        .sort((left, right) => left.name.localeCompare(right.name))
    ).toEqual(MCP_SURFACE.resources);
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
    if (!definition) {
      throw new Error(`Missing canonical tool: ${toolName}`);
    }

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
    markers: [],
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

it("matches canonical component instance structural fixtures standalone and in batches", () => {
  for (const operation of INSTANCE_CATALOG.validOperations) {
    expect(headlessEditSchema.safeParse(operation).success).toBe(true);
    expect(
      schemas.timelineBatchEdit.safeParse({
        expectedRevision: 0,
        operations: [operation],
        projectId: "project",
      }).success
    ).toBe(true);
    const { operation: name, ...fields } = operation;
    const schema =
      name === "add_component_instance"
        ? schemas.addComponentInstance
        : schemas.componentInstanceUpdate;
    expect(
      schema.safeParse({ expectedRevision: 0, projectId: "project", ...fields })
        .success
    ).toBe(true);
  }
  for (const operation of INSTANCE_CATALOG.invalidOperations) {
    expect(headlessEditSchema.safeParse(operation).success).toBe(false);
  }
});

it("matches lifecycle duplication fixtures across standalone and batch contracts", () => {
  for (const edit of LIFECYCLE.validOperations) {
    expect(headlessEditSchema.safeParse(edit).success).toBe(true);
    const { operation: _operation, ...fields } = edit;
    expect(
      schemas.componentInstanceDuplicate.safeParse({
        expectedRevision: 0,
        projectId: "project",
        ...fields,
      }).success
    ).toBe(true);
    expect(
      schemas.timelineBatchEdit.safeParse({
        expectedRevision: 0,
        operations: [edit],
        projectId: "project",
      }).success
    ).toBe(true);
  }
  for (const edit of LIFECYCLE.invalidOperations) {
    expect(headlessEditSchema.safeParse(edit).success).toBe(false);
    const { operation: _operation, ...fields } = edit;
    expect(
      schemas.componentInstanceDuplicate.safeParse({
        expectedRevision: 0,
        projectId: "project",
        ...fields,
      }).success
    ).toBe(false);
  }
  for (const fixture of LIFECYCLE.semanticFailures) {
    expect(headlessEditSchema.safeParse(fixture.edit).success).toBe(
      fixture.mcpAccept
    );
  }
  for (const edit of LIFECYCLE.validBatch) {
    expect(headlessEditSchema.safeParse(edit).success).toBe(true);
  }
});

it("matches canonical closed time expressions through the public timing decoder", () => {
  const ids = new Set<string>();
  for (const fixture of MOTION_GRAPHICS_CONTRACT.timeExpressionCases) {
    expect(ids.has(fixture.id)).toBe(false);
    ids.add(fixture.id);
    expect(
      timeExpressionSchema.safeParse(fixture.value).success,
      fixture.id
    ).toBe(fixture.accept);
    for (const operation of [
      {
        itemId: "item",
        operation: "set_item_start_time",
        scope: "root",
        time: fixture.value,
      },
    ]) {
      expect(headlessEditSchema.safeParse(operation).success, fixture.id).toBe(
        fixture.accept
      );
      expect(
        schemas.timelineBatchEdit.safeParse({
          expectedRevision: 0,
          operations: [
            {
              color: "#ffffff",
              durationMs: 100,
              height: 1,
              operation: "add_rectangle",
              resultAlias: "item",
              startMs: 0,
              trackId: "track",
              transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
              width: 1,
            },
            { ...operation, itemId: "@item" },
          ],
          projectId: "project",
        }).success,
        fixture.id
      ).toBe(fixture.accept);
    }
  }
});
