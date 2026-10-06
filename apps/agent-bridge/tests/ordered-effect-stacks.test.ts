import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import catalog from "../../../contracts/extended-visual-animation-v1.json";
import {
  headlessEditSchema,
  schemas,
  visualEffectSchema,
} from "../src/schemas";
import {
  ORDERED_EFFECT_PREDECESSOR_SHA256,
  orderedEffectDigest,
  projectOrderedEffectCatalogPredecessor as projectOrderedEffectCatalogPredecessor35,
} from "./fixtures/ordered-effect-projection";
import { restoreParameterizedCatalogMarker } from "./fixtures/parameterized-effect-projection";

const projectOrderedEffectCatalogPredecessor = (source: unknown) =>
  projectOrderedEffectCatalogPredecessor35(
    restoreParameterizedCatalogMarker(source)
  );
const fixture = catalog.orderedEffectCases;
it("governs exact ordered arrays, closed public edits and independent numeric witnesses", () => {
  for (const stack of Object.values(fixture.orders)) {
    for (const effect of stack) {
      expect(visualEffectSchema.safeParse(effect).success).toBe(true);
    }
    expect(
      headlessEditSchema.safeParse({
        effects: stack,
        itemId: "leaf",
        operation: "update_item",
      }).success
    ).toBe(true);
    expect(
      schemas.timelineUpdateItem.safeParse({
        effects: stack,
        expectedRevision: 0,
        itemId: "leaf",
        projectId: "p",
      }).success
    ).toBe(true);
  }
  for (const { id, value } of fixture.invalidStacks) {
    expect(
      headlessEditSchema.safeParse({
        effects: value,
        itemId: "leaf",
        operation: "update_item",
      }).success
    ).toBe(id === "duplicate");
    expect(
      schemas.timelineUpdateItem.safeParse({
        effects: value,
        expectedRevision: 0,
        itemId: "leaf",
        projectId: "p",
      }).success
    ).toBe(id === "duplicate");
  }
  const [x = Number.NaN, y = Number.NaN] = fixture.primary.localPixelCenter;
  const q = (((2 * x) / 32 - 1) ** 2 + ((2 * y) / 24 - 1) ** 2) / 2;
  expect(q).toBeCloseTo(fixture.primary.radialFactor, 14);
  const f = 1 - 0.8 * q;
  expect(fixture.primary.forwardLinear).toEqual([0.25 * f, 0.25, 0]);
  expect(fixture.primary.reverseLinear).toEqual([0.25 * f, 0.25 * f, 0]);
  const unnormalized = Array.from({ length: 7 }, (_, i) =>
    Math.exp(-0.5 * (i - 3) ** 2)
  );
  const sum = unnormalized.reduce((a, b) => a + b, 0);
  const kernel = unnormalized.map((v) => v / sum);
  for (const [i, value] of kernel.entries()) {
    expect(value).toBeCloseTo(
      fixture.expanded.normalizedKernel[i] ?? Number.NaN,
      14
    );
  }
  expect(kernel.slice(4).reduce((a, b) => a + b, 0)).toBeCloseTo(
    0.300_474_860_173_772_5,
    14
  );
  expect(fixture.encodedGreen.linear).toBeCloseTo(0.214_041_140_482_232_55, 14);
  expect(fixture.primary.sceneAlphaByte).toBe(255);
  expect(fixture.expanded.sceneAlphaByte).toBe(255);
});

it("removes only complete approved orderedEffectCases and reproduces the verified full53 predecessor", () => {
  const source = JSON.parse(
    readFileSync(
      new URL(
        "../../../contracts/extended-visual-animation-v1.json",
        import.meta.url
      ),
      "utf8"
    )
  );
  const before = JSON.stringify(source);
  const predecessor = projectOrderedEffectCatalogPredecessor(source);
  expect(orderedEffectDigest(predecessor)).toBe(
    ORDERED_EFFECT_PREDECESSOR_SHA256
  );
  expect(Object.hasOwn(predecessor, "orderedEffectCases")).toBe(false);
  expect(JSON.stringify(source)).toBe(before);
  for (const malformed of [
    null,
    {},
    [],
    { ...source.orderedEffectCases, oracle: { convertedMaxByteError: 2 } },
  ]) {
    const drift = { ...source, orderedEffectCases: malformed };
    const frozen = JSON.stringify(drift);
    expect(() => projectOrderedEffectCatalogPredecessor(drift)).toThrow(
      "Missing or malformed"
    );
    expect(JSON.stringify(drift)).toBe(frozen);
  }
  const missing = structuredClone(source);
  Reflect.deleteProperty(missing, "orderedEffectCases");
  expect(() => projectOrderedEffectCatalogPredecessor(missing)).toThrow(
    "Missing or malformed"
  );
  for (const drift of [
    { ...source, projectSchemaVersion: 34 },
    { ...source, unapproved: true },
    { ...source, limits: { ...source.limits, maxEffects: 17 } },
  ]) {
    expect(() => projectOrderedEffectCatalogPredecessor(drift)).toThrow(
      "verified predecessor"
    );
  }
});
