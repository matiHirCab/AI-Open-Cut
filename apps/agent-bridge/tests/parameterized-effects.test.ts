import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import surface from "../../../contracts/mcp-surface-v1.json";
import contract from "../../../contracts/parameterized-effects-v1.json";
import {
  headlessEditSchema,
  schemas,
  visualEffectSchema,
} from "../src/schemas";
import { orderedEffectDigest } from "./fixtures/ordered-effect-projection";
import {
  PARAMETERIZED_PREDECESSOR_PINS,
  projectParameterizedCatalogPredecessor,
  projectParameterizedMcpPredecessor,
} from "./fixtures/parameterized-effect-projection";

it("validates every canonical static control endpoint and malformed value through public edits", () => {
  expect(contract.projectSchemaVersion).toBe(38);
  for (const entry of contract.effectCases) {
    expect(visualEffectSchema.safeParse(entry.value).success, entry.id).toBe(
      entry.accepted
    );
    expect(
      headlessEditSchema.safeParse({
        effects: [entry.value],
        itemId: "leaf",
        operation: "update_item",
      }).success,
      entry.id
    ).toBe(entry.accepted);
    expect(
      schemas.timelineUpdateItem.safeParse({
        effects: [entry.value],
        expectedRevision: 0,
        itemId: "leaf",
        projectId: "p",
      }).success,
      entry.id
    ).toBe(entry.accepted);
  }
  for (const field of ["exposureStops", "contrast", "saturation"]) {
    for (const value of [
      Number.NaN,
      Number.POSITIVE_INFINITY,
      Number.NEGATIVE_INFINITY,
    ]) {
      expect(
        visualEffectSchema.safeParse({
          ...contract.nativeWitness.identity,
          [field]: value,
        }).success
      ).toBe(false);
    }
  }
});
it("pins all ten complete schema35 catalogs and rejects unrelated drift without mutating source", () => {
  expect(Object.keys(PARAMETERIZED_PREDECESSOR_PINS)).toHaveLength(10);
  for (const [name, pin] of Object.entries(PARAMETERIZED_PREDECESSOR_PINS)) {
    const source = JSON.parse(
      readFileSync(
        new URL(`../../../contracts/${name}.json`, import.meta.url),
        "utf8"
      )
    );
    const before = JSON.stringify(source);
    expect(
      orderedEffectDigest(projectParameterizedCatalogPredecessor(name, source)),
      name
    ).toBe(pin);
    expect(JSON.stringify(source)).toBe(before);
    for (const drift of [
      { ...source, projectSchemaVersion: 35 },
      { ...source, unauthorized: true },
    ]) {
      expect(() =>
        projectParameterizedCatalogPredecessor(name, drift)
      ).toThrow();
    }
  }
});
it("removes only the exact fifth MCP variant, two capabilities and two current literals", () => {
  const before = JSON.stringify(surface);
  const previous = projectParameterizedMcpPredecessor(surface);
  expect(JSON.stringify(surface)).toBe(before);
  expect(previous.capabilityIdentifiers).not.toContain(
    "parameterized_effect_models_v1"
  );
  expect(previous.capabilityIdentifiers).not.toContain(
    "parameterized_effects_v1"
  );
  for (const mutation of [
    "missing",
    "malformed",
    "duplicate",
    "capability",
    "literal",
  ]) {
    const source = structuredClone(surface);
    const [, , , , fifth] = source.$defs.ExtendedVisualEffect.oneOf;
    if (!fifth?.properties.contrast) {
      throw new Error("canonical fifth color variant missing");
    }
    if (mutation === "missing") {
      source.$defs.ExtendedVisualEffect.oneOf.pop();
    }
    if (mutation === "malformed") {
      fifth.properties.contrast.maximum = 3;
    }
    if (mutation === "duplicate") {
      source.$defs.ExtendedVisualEffect.oneOf.push(fifth);
    }
    if (mutation === "capability") {
      source.capabilityIdentifiers.push("parameterized_effects_v1");
    }
    if (mutation === "literal") {
      source.toolDefinitions.editor_get_status.outputSchema.properties.projectSchemaVersion.const = 35;
    }
    const frozen = JSON.stringify(source);
    expect(
      () => projectParameterizedMcpPredecessor(source),
      mutation
    ).toThrow();
    expect(JSON.stringify(source)).toBe(frozen);
  }
});
