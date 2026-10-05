import { createHash } from "node:crypto";
import { describe, expect, it } from "vitest";
import animation from "../../../contracts/animation-channels-v1.json";
import headless from "../../../contracts/headless-protocol-v1.json";
import models from "../../../contracts/mask-models-v1.json";
import rendering from "../../../contracts/mask-rendering-v1.json";
import mcp from "../../../contracts/mcp-surface-v1.json";
import {
  MASK_ANIMATION_CAPABILITY,
  MASK_RENDERING_CAPABILITY,
} from "../src/headless-contract";
import {
  animationChannelSchema,
  headlessEditSchema,
  schemas,
} from "../src/schemas";
import {
  MASK_PROPERTIES,
  MASK_RENDERING_PREDECESSOR_PINS,
  projectMaskRenderingCatalogPredecessor,
  projectMaskRenderingMcpPredecessor,
} from "./fixtures/mask-rendering-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";

const normalize = (value: unknown): unknown => {
  if (Array.isArray(value)) {
    return value.map(normalize);
  }
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([k, v]) => [k, normalize(v)])
    );
  }
  return value;
};
const digest = (value: unknown) =>
  createHash("sha256")
    .update(JSON.stringify(normalize(value)))
    .digest("hex");
describe("active mask rendering canonical public contracts", () => {
  it("accepts all fifteen authored typed fixtures through direct, batch and draft input schemas", () => {
    expect(Object.keys(rendering.properties)).toEqual(MASK_PROPERTIES);
    expect(Object.keys(rendering.properties)).toHaveLength(15);
    for (const fixture of rendering.channelCases) {
      expect(
        animationChannelSchema.parse(fixture.channel),
        fixture.name
      ).toEqual(fixture.channel);
      const operation = {
        animationChannels: [fixture.channel],
        itemId: "leaf",
        operation: "set_animation_channels",
      };
      expect(headlessEditSchema.parse(operation)).toEqual(operation);
      const wrapped = {
        expectedRevision: 0,
        operations: [operation],
        projectId: "p",
      };
      expect(schemas.timelineBatchEdit.parse(wrapped).operations).toEqual([
        operation,
      ]);
      expect(schemas.draftCreate.parse(wrapped).operations).toEqual([
        operation,
      ]);
      expect(
        schemas.draftUpdate.parse({ ...wrapped, draftId: "draft" }).operations
      ).toEqual([operation]);

      expect(
        schemas.timelineSetAnimationChannels.parse({
          animationChannels: [fixture.channel],
          expectedRevision: 0,
          itemId: "leaf",
          projectId: "p",
        })
      ).toMatchObject({ animationChannels: [fixture.channel] });
      expect(
        animation.active[
          fixture.channel.property as keyof typeof animation.active
        ]
      ).toEqual(
        rendering.properties[
          fixture.channel.property as keyof typeof rendering.properties
        ]
      );
      const { target } = fixture.channel;
      expect(
        animationChannelSchema.safeParse({
          ...fixture.channel,
          target: { ...target, kind: "source_layer" },
        }).success
      ).toBe(false);
      expect(
        animationChannelSchema.safeParse({
          ...fixture.channel,
          target: { ...target, expression: "t" },
        }).success
      ).toBe(false);
    }
  });
  it("preserves literal Unicode and alias-like mask IDs with exact UTF8 byte bounds", () => {
    const channel = rendering.channelCases[0]?.channel;
    if (!channel) {
      throw new Error("canonical channel missing");
    }
    for (const id of ["@paint", "é".repeat(64)]) {
      expect(
        animationChannelSchema.parse({
          ...channel,
          target: { ...channel.target, id },
        }).target?.id
      ).toBe(id);
    }
    expect(
      animationChannelSchema.safeParse({
        ...channel,
        target: { ...channel.target, id: "é".repeat(65) },
      }).success
    ).toBe(false);
  });
  it("rejects wrong or missing approved animation additions before predecessor projection", () => {
    for (const property of MASK_PROPERTIES) {
      const active = {
        ...animation.active,
        [property]: { ...animation.active[property], maximum: 99_999_999 },
      };
      expect(() =>
        projectMaskRenderingCatalogPredecessor("animation-channels-v1", {
          ...animation,
          active,
        })
      ).toThrow("Incorrect approved addition");
      const absent = Object.fromEntries(
        Object.entries(animation.active).filter(([key]) => key !== property)
      );
      expect(() =>
        projectMaskRenderingCatalogPredecessor("animation-channels-v1", {
          ...animation,
          active: absent,
        })
      ).toThrow("Incorrect approved addition");
    }
    for (const maskTarget of [
      undefined,
      { ...animation.maskTarget, maxIdBytes: 129 },
      { ...animation.maskTarget, unrelated: { schemaVersion: 33 } },
    ]) {
      expect(() =>
        projectMaskRenderingCatalogPredecessor("animation-channels-v1", {
          ...animation,
          maskTarget,
        })
      ).toThrow("Incorrect approved addition");
    }
    const drift = {
      ...animation,
      active: {
        ...animation.active,
        "mask.unapproved": { maximum: 1, minimum: 0, valueType: "scalar" },
      },
    };
    expect(
      digest(
        projectMaskRenderingCatalogPredecessor("animation-channels-v1", drift)
      )
    ).not.toBe(MASK_RENDERING_PREDECESSOR_PINS["animation-channels-v1"]);
  });
  it("keeps independent model/editor/render capabilities in canonical catalogs", () => {
    expect(rendering.projectSchemaVersion).toBe(33);
    expect(rendering.capabilities).toEqual({
      editor: MASK_ANIMATION_CAPABILITY,
      renderer: MASK_RENDERING_CAPABILITY,
    });
    expect(headless.status.editorCapabilities).toContain(
      MASK_ANIMATION_CAPABILITY
    );
    expect(headless.status.editorCapabilities).not.toContain(
      MASK_RENDERING_CAPABILITY
    );
    expect(headless.status.renderingCapabilities).toContain(
      MASK_RENDERING_CAPABILITY
    );
    expect(headless.status.renderingCapabilities).not.toContain(
      MASK_ANIMATION_CAPABILITY
    );
    expect(models.rendererStage).toBe("contracts/mask-rendering-v1.json");
  });
  it("pins the complete model predecessor and rejects missing/wrong timing or unrelated drift", () => {
    const previous = projectMaskRenderingCatalogPredecessor(
      "mask-models-v1",
      models
    );
    expect(digest(previous)).toBe(
      MASK_RENDERING_PREDECESSOR_PINS["mask-models-v1"]
    );
    for (const timing of [
      undefined,
      "static_metadata_no_animation_targets",
      "unapproved",
    ]) {
      expect(() =>
        projectMaskRenderingCatalogPredecessor("mask-models-v1", {
          ...models,
          semantics: { ...models.semantics, timing },
        })
      ).toThrow("mask timing");
    }
    const drift = {
      ...models,
      limits: { ...models.limits, maxMasksPerItem: 17 },
      unrelated: { projectSchemaVersion: 33 },
    };
    const projected = projectMaskRenderingCatalogPredecessor(
      "mask-models-v1",
      drift
    );
    expect(projected.unrelated).toEqual({ projectSchemaVersion: 33 });
    expect(digest(projected)).not.toBe(
      MASK_RENDERING_PREDECESSOR_PINS["mask-models-v1"]
    );
  });
  it("rejects malformed exact MCP additions while retaining matching unrelated fields", () => {
    const missing = structuredClone(mcp);
    missing.capabilityIdentifiers = missing.capabilityIdentifiers.filter(
      (x) => x !== MASK_RENDERING_CAPABILITY
    );
    expect(() => projectMaskRenderingMcpPredecessor(missing)).toThrow(
      "capability multiplicity"
    );
    const drift = structuredClone(mcp);
    const input = drift.toolDefinitions.editor_get_status.inputSchema as Record<
      string,
      unknown
    >;
    input.description = "Unrelated mask/property drift";
    input.properties = {
      unrelated: {
        enum: ["mask", "mask.path_points"],
        schemaVersion: { const: 33 },
      },
    };
    const projected = projectMaskRenderingMcpPredecessor(drift);
    const expanded = expandMcpSurfaceCatalog(projected);
    expect(expanded.toolDefinitions).toHaveProperty(
      "editor_get_status.inputSchema.properties",
      input.properties
    );
    expect(
      createHash("sha256").update(JSON.stringify(expanded)).digest("hex")
    ).not.toBe(
      "88b55ff7be147cb4aadc016dd92f92342c3dc366f49ac139b8116513d5854830"
    );
  });
  it("retains independently authored expansion and ordered scalar formula examples", () => {
    expect(rendering.numericCases).toEqual([
      {
        coverage: 0.25,
        expansionGrid: 0.5,
        expected: 0.75,
        name: "partial_below_half_expansion",
      },
      {
        coverage: 0.25,
        expansionGrid: 0,
        expected: 0.25,
        name: "zero_expansion_exact",
      },
      {
        coverage: 0,
        expansionGrid: 8,
        expected: 0,
        insideSeeds: [],
        name: "empty_seeds_stay_empty",
      },
    ]);
    expect(rendering.compositionCases.map((x) => x.expected)).toEqual([
      0.625, 0.125, 0.125, 0.5,
    ]);
    expect(rendering.memory.additiveKernelBytes).toContain("4K");
    expect(rendering.memory.retainedContours).toContain("SUM");
    expect(rendering.memory.additionalRetainedMetadata).toContain("S0_no_grid");
  });
});
