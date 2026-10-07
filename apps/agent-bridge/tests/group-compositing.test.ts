import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import authority from "../../../contracts/group-compositing-v1.json";
import surface from "../../../contracts/mcp-surface-v1.json";
import {
  compositionClipSchema,
  headlessEditSchema,
  schemas,
  visualEffectSchema,
} from "../src/schemas";
import additions from "./fixtures/group-compositing-additions.json";
import {
  GROUP_PREDECESSOR_PINS,
  projectGroupCatalogPredecessor,
  projectGroupMcpPredecessor,
} from "./fixtures/group-compositing-projection";
import { orderedEffectDigest } from "./fixtures/ordered-effect-projection";

it("matches every approved overlay endpoint integer and closed-field fixture across existing public edits", () => {
  expect(authority.projectSchemaVersion).toBe(38);
  for (const entry of authority.effectCases) {
    expect(visualEffectSchema.safeParse(entry.value).success, entry.id).toBe(
      entry.accepted
    );
    expect(
      headlessEditSchema.safeParse({
        effects: [entry.value],
        itemId: "owner",
        operation: "update_item",
      }).success,
      entry.id
    ).toBe(entry.accepted);
    expect(
      schemas.timelineUpdateItem.safeParse({
        effects: [entry.value],
        expectedRevision: 0,
        itemId: "owner",
        projectId: "p",
      }).success,
      entry.id
    ).toBe(entry.accepted);
  }
  for (const [type, fields] of Object.entries(authority.fields)) {
    const original =
      authority.nativeWitness[type === "screen_flash" ? "flash" : "particles"];
    for (const field of Object.keys(fields)) {
      for (const value of [
        Number.NaN,
        Number.POSITIVE_INFINITY,
        Number.NEGATIVE_INFINITY,
      ]) {
        expect(
          visualEffectSchema.safeParse({ ...original, [field]: value }).success
        ).toBe(false);
      }
    }
  }
});
it("keeps clip stored nonnull and edits nullable with omission preservation", () => {
  for (const entry of authority.clipCases) {
    expect(compositionClipSchema.safeParse(entry.value).success).toBe(
      entry.accepted
    );
  }
  for (const clip of [undefined, null, { type: "composition_bounds" }]) {
    expect(
      headlessEditSchema.safeParse({
        itemId: "owner",
        operation: "update_item",
        ...(clip === undefined ? {} : { clip }),
      }).success
    ).toBe(true);
    expect(
      schemas.timelineUpdateItem.safeParse({
        effects: [],
        expectedRevision: 0,
        itemId: "owner",
        projectId: "p",
        ...(clip === undefined ? {} : { clip }),
      }).success
    ).toBe(true);
  }
  for (const { value: clip } of authority.clipCases.filter(
    (entry) => !entry.accepted && entry.value !== null
  )) {
    expect(
      headlessEditSchema.safeParse({
        clip,
        itemId: "owner",
        operation: "update_item",
      }).success
    ).toBe(false);
    expect(
      schemas.timelineUpdateItem.safeParse({
        clip,
        expectedRevision: 0,
        itemId: "owner",
        projectId: "p",
      }).success
    ).toBe(false);
  }
});
it("pins all eleven actual complete verified36 catalogs through the sole37 marker change", () => {
  expect(Object.keys(GROUP_PREDECESSOR_PINS)).toHaveLength(11);
  for (const [name, pin] of Object.entries(GROUP_PREDECESSOR_PINS)) {
    const source = JSON.parse(
      readFileSync(
        new URL(`../../../contracts/${name}.json`, import.meta.url),
        "utf8"
      )
    );
    const before = JSON.stringify(source);
    expect(
      orderedEffectDigest(projectGroupCatalogPredecessor(name, source)),
      name
    ).toBe(pin);
    expect(JSON.stringify(source)).toBe(before);
    for (const drift of [
      { ...source, projectSchemaVersion: 36 },
      { ...source, unauthorized: true },
    ]) {
      expect(() => projectGroupCatalogPredecessor(name, drift)).toThrow();
    }
  }
});
it("projects only exactly two overlays capabilities literals and29 enumerated clip owners without input mutation", () => {
  const before = JSON.stringify(surface);
  const previous = projectGroupMcpPredecessor(surface);
  expect(JSON.stringify(surface)).toBe(before);
  expect(previous.capabilityIdentifiers).not.toContain("group_compositing_v1");
  expect(previous.capabilityIdentifiers).not.toContain(
    "group_compositing_models_v1"
  );
  expect(additions.paths).toHaveLength(29);
  expect(new Set(additions.paths).size).toBe(29);
  for (const mutation of [
    "missing",
    "duplicate",
    "malformed",
    "capability",
    "literal",
    "clip_missing",
    "clip_unknown",
    "misplaced_clip",
    "unrelated",
  ]) {
    const source = structuredClone(surface);
    const variants = source.$defs.ExtendedVisualEffect.oneOf;
    if (mutation === "missing") {
      variants.pop();
    }
    if (mutation === "duplicate") {
      const [, , , , , fifth] = variants;
      if (!fifth) {
        throw new Error("Missing frozen overlay");
      }
      variants.push(fifth);
    }
    if (mutation === "malformed") {
      const radius = variants[6]?.properties.radiusPx;
      if (!radius) {
        throw new Error("Missing frozen radius");
      }
      radius.maximum = 17;
    }
    if (mutation === "capability") {
      source.capabilityIdentifiers.push("group_compositing_v1");
    }
    if (mutation === "literal") {
      source.toolDefinitions.editor_get_status.outputSchema.properties.projectSchemaVersion.const = 36;
    }
    const owner = source.$defs
      .InputPropertiesTracksItemsPropertiesItemsItemsOneOfGroupProperties as Record<
      string,
      unknown
    >;
    if (mutation === "clip_missing") {
      Reflect.deleteProperty(owner, "clip");
    }
    if (mutation === "clip_unknown") {
      owner.clip = { properties: { path: { type: "string" } }, type: "object" };
    }
    if (mutation === "misplaced_clip") {
      source.$defs.ShapePropertiesFill = {
        ...source.$defs.ShapePropertiesFill,
        clip: owner.clip,
      } as typeof source.$defs.ShapePropertiesFill;
    }
    if (mutation === "unrelated") {
      owner.zIndex = { minimum: -99, type: "integer" };
    }
    const frozen = JSON.stringify(source);
    expect(() => projectGroupMcpPredecessor(source), mutation).toThrow();
    expect(JSON.stringify(source)).toBe(frozen);
  }
});
