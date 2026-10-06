import { describe, expect, it } from "vitest";
import contract from "../../../contracts/track-mattes-v1.json";
import {
  headlessEditSchema,
  schemas,
  timelineItemSchema,
} from "../src/schemas";

const input = { expectedRevision: 0, itemId: "leaf", projectId: "p" };
const leaf = {
  color: "#ffffff",
  durationMs: 1000,
  hidden: false,
  id: "leaf",
  keyframes: [],
  stackOrder: 0,
  startMs: 0,
  transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
  type: "solid_color",
  zIndex: 0,
};
describe("closed track matte public schemas", () => {
  it("preserves canonical references and presence-sensitive edits through every wrapper", () => {
    for (const matte of [...contract.validReferences, null]) {
      const operation = {
        itemId: "leaf",
        matte,
        matteOnly: true,
        operation: "update_item",
      };
      expect(headlessEditSchema.parse(operation)).toEqual(operation);
      expect(
        schemas.timelineUpdateItem.parse({ ...input, matte, matteOnly: true })
      ).toEqual({ ...input, matte, matteOnly: true });
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
        schemas.draftUpdate.parse({ ...wrapped, draftId: "d" }).operations
      ).toEqual([operation]);
    }
    expect(
      headlessEditSchema.parse({ itemId: "leaf", operation: "update_item" })
    ).not.toHaveProperty("matte");
  });
  it("rejects malformed nested references, null visibility and excessive UTF8 IDs", () => {
    for (const matte of contract.invalidReferences.filter(
      (value) => value !== null
    )) {
      expect(
        schemas.timelineUpdateItem.safeParse({ ...input, matte }).success
      ).toBe(false);
    }
    expect(
      schemas.timelineUpdateItem.safeParse({ ...input, matteOnly: null })
        .success
    ).toBe(false);
    expect(
      schemas.timelineUpdateItem.safeParse({
        ...input,
        matte: { channel: "alpha", sourceId: "é".repeat(64) },
      }).success
    ).toBe(true);
    expect(
      schemas.timelineUpdateItem.safeParse({
        ...input,
        matte: { channel: "alpha", sourceId: "é".repeat(65) },
      }).success
    ).toBe(false);
  });
  it("requires nonnull stored references and only defaults on ineligible DTOs", () => {
    const [matte] = contract.validReferences;
    expect(
      timelineItemSchema.parse({ ...leaf, matte, matteOnly: true })
    ).toEqual({ ...leaf, matte, matteOnly: true });
    expect(timelineItemSchema.safeParse({ ...leaf, matte: null }).success).toBe(
      false
    );
    expect(
      timelineItemSchema.safeParse({ ...leaf, matteOnly: null }).success
    ).toBe(false);
    const { color: _color, keyframes: _keyframes, ...group } = leaf;
    expect(
      timelineItemSchema.safeParse({
        ...group,
        matteOnly: false,
        type: "group",
      }).success
    ).toBe(true);
    for (const addition of [{ matte }, { matte: null }, { matteOnly: true }]) {
      expect(
        timelineItemSchema.safeParse({ ...group, type: "group", ...addition })
          .success
      ).toBe(false);
    }
  });
});

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import surface from "../../../contracts/mcp-surface-v1.json";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";
import additions from "./fixtures/track-matte-additions.json";
import {
  projectTrackMatteCatalogPredecessor,
  projectTrackMatteMcpPredecessor,
  TRACK_MATTE_PREDECESSOR_PINS,
} from "./fixtures/track-matte-projection";

const sorted = (value: unknown): unknown => {
  if (Array.isArray(value)) {
    return value.map(sorted);
  }
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, child]) => [key, sorted(child)])
    );
  }
  return value;
};
const semanticDigest = (value: unknown) =>
  createHash("sha256")
    .update(JSON.stringify(sorted(value)), "utf8")
    .digest("hex");
const mcpDigest = (value: unknown) =>
  createHash("sha256")
    .update(JSON.stringify(expandMcpSurfaceCatalog(value)), "utf8")
    .digest("hex");
const predecessorDigest =
  "803bf5954ebd4cb47be98dd87b4994e6d261eae20693199c0f569f535452f170";
it("pins all eight exact marker-only predecessor catalogs without discarding drift", () => {
  expect(Object.keys(TRACK_MATTE_PREDECESSOR_PINS)).toHaveLength(8);
  for (const [name, pin] of Object.entries(TRACK_MATTE_PREDECESSOR_PINS)) {
    const value = JSON.parse(
      readFileSync(
        new URL(`../../../contracts/${name}.json`, import.meta.url),
        "utf8"
      )
    );
    expect(
      semanticDigest(projectTrackMatteCatalogPredecessor(name, value)),
      name
    ).toBe(pin);
    expect(() =>
      projectTrackMatteCatalogPredecessor(name, {
        ...value,
        projectSchemaVersion: 33,
      })
    ).toThrow("Incorrect approved");
    expect(
      semanticDigest(
        projectTrackMatteCatalogPredecessor(name, {
          ...value,
          unrelated: { projectSchemaVersion: 34 },
        })
      ),
      name
    ).not.toBe(pin);
  }
});
it("requires all58 exact new field shapes and strict reference definition before MCP projection", () => {
  expect(additions.entries).toHaveLength(58);
  expect(mcpDigest(projectTrackMatteMcpPredecessor(surface))).toBe(
    predecessorDigest
  );
  const originalBytes = JSON.stringify(surface);
  const drift = structuredClone(surface) as unknown as Record<string, unknown>;
  const isolatedBytes = JSON.stringify(drift);
  let rejectedCases = 0;
  for (const entry of additions.entries) {
    let parent = drift;
    for (const key of entry.path.slice(0, -1)) {
      parent = parent[key] as Record<string, unknown>;
    }
    const key = entry.path.at(-1);
    if (!key) {
      throw new Error("empty manifest path");
    }
    const original = parent[key];
    const originalEntries = Object.entries(parent);
    for (const missing of [false, true]) {
      try {
        if (missing) {
          Reflect.deleteProperty(parent, key);
        } else {
          parent[key] = { type: "string" };
        }
        expect(
          () => projectTrackMatteMcpPredecessor(drift),
          entry.path.join(".")
        ).toThrow();
        rejectedCases += 1;
      } finally {
        if (missing) {
          // Re-inserting just the deleted field changes JSON key order.
          // Restore the small owning record in its complete original order.
          for (const present of Object.keys(parent)) {
            Reflect.deleteProperty(parent, present);
          }
          for (const [name, value] of originalEntries) {
            parent[name] = value;
          }
        } else {
          parent[key] = original;
        }
      }
    }
  }
  expect(rejectedCases).toBe(116);
  expect(JSON.stringify(drift)).toBe(isolatedBytes);
  expect(JSON.stringify(surface)).toBe(originalBytes);
  expect(
    new Set(additions.entries.map((entry) => entry.path.join("."))).size
  ).toBe(58);
  for (const missing of ["matte_models_v1", "track_mattes_v1"]) {
    const value = structuredClone(surface);
    value.capabilityIdentifiers = value.capabilityIdentifiers.filter(
      (capability) => capability !== missing
    );
    expect(() => projectTrackMatteMcpPredecessor(value)).toThrow(
      "multiplicity"
    );
  }
  const boundDrift = structuredClone(surface);
  boundDrift.$defs.TrackMatteReference.properties.sourceId.maxLength = 129;
  expect(() => projectTrackMatteMcpPredecessor(boundDrift)).toThrow(
    "Incorrect approved"
  );
  const unknownDefinition = structuredClone(surface) as unknown as Record<
    string,
    unknown
  >;
  const definitions = unknownDefinition.$defs as Record<
    string,
    Record<string, unknown>
  >;
  const reference = definitions.TrackMatteReference as Record<string, unknown>;
  expect(reference).toBeDefined();
  reference.unapproved = true;
  expect(() => projectTrackMatteMcpPredecessor(unknownDefinition)).toThrow(
    "Incorrect approved"
  );
  const malformed = structuredClone(surface);
  malformed.$defs.TrackMatteReference.properties.channel.enum = [
    "alpha",
    "red",
  ];
  expect(() => projectTrackMatteMcpPredecessor(malformed)).toThrow(
    "Incorrect approved"
  );
});
it("isolates successful and rejected matte projections from their source", () => {
  const before = JSON.stringify(surface);
  const projected = projectTrackMatteMcpPredecessor(surface);
  const definitions = projected.$defs as Record<string, unknown>;
  definitions.unapproved = { nested: ["changed"] };
  expect(JSON.stringify(surface)).toBe(before);
  const invalid = structuredClone(surface);
  invalid.capabilityIdentifiers.push("track_mattes_v1");
  const invalidBefore = JSON.stringify(invalid);
  expect(() => projectTrackMatteMcpPredecessor(invalid)).toThrow(
    "multiplicity"
  );
  expect(JSON.stringify(invalid)).toBe(invalidBefore);
});

it("rejects duplicated capabilities and preserves unapproved matching fields and annotations", () => {
  const duplicate = structuredClone(surface);
  duplicate.capabilityIdentifiers.push("track_mattes_v1");
  expect(() => projectTrackMatteMcpPredecessor(duplicate)).toThrow(
    "multiplicity"
  );
  const drift = structuredClone(surface) as unknown as Record<string, unknown>;
  const tools = drift.toolDefinitions as Record<
    string,
    Record<string, unknown>
  >;
  expect(tools).toHaveProperty("editor_get_status");
  const status = tools.editor_get_status as Record<string, unknown>;
  const inputSchema = status.inputSchema as Record<string, unknown>;
  inputSchema.properties = { matte: { type: "string" } };
  expect(mcpDigest(projectTrackMatteMcpPredecessor(drift))).not.toBe(
    predecessorDigest
  );
  for (const path of additions.literalPaths) {
    const value = structuredClone(surface) as unknown as Record<
      string,
      unknown
    >;
    let parent = value;
    for (const key of path.slice(0, -1)) {
      parent = parent[key] as Record<string, unknown>;
    }
    const key = path.at(-1);
    if (!key) {
      throw new Error("empty literal path");
    }
    parent[key] = 33;
    expect(() => projectTrackMatteMcpPredecessor(value)).toThrow(
      "Incorrect approved"
    );
  }
});
it("uses independent premultiplied alpha/luma and copy-average oracles", () => {
  for (const fixture of contract.numericCases) {
    const [red = 0, green = 0, blue = 0, alpha = 0] =
      fixture.providerPremultipliedLinear;
    const coverage =
      fixture.channel === "alpha"
        ? alpha
        : 0.2126 * red + 0.7152 * green + 0.0722 * blue;
    expect(coverage, fixture.name).toBeCloseTo(fixture.expectedCoverage, 12);
    fixture.recipientPremultipliedLinear.forEach((component, index) => {
      expect(component * coverage, fixture.name).toBeCloseTo(
        fixture.expectedRecipient[index] ?? Number.NaN,
        12
      );
    });
  }
  const [copies] = contract.copyAverageCases;
  if (!copies) {
    throw new Error("missing copy witness");
  }
  const average = copies.copyAlphaSamples.map(([a = 0, b = 0]) => (a + b) / 2);
  expect(
    1 - average.reduce((remaining, alpha) => remaining * (1 - alpha), 1)
  ).toBe(0.4375);
  expect(copies.forbiddenAggregateBeforeAverageAlpha).toBe(0.5);
});
