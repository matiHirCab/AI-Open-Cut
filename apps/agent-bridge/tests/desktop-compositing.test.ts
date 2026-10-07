import "./desktop-compositing-workflow";
import { describe, expect, it } from "vitest";
import catalog from "../../../contracts/desktop-compositing-controls-v1.json";
import surface from "../../../contracts/mcp-surface-v1.json";
import {
  blendModeSchema,
  headlessEditSchema,
  maskSchema,
  schemas,
  visualEffectSchema,
} from "../src/schemas";
import { projectSpeechMarkerMcpPredecessor } from "./fixtures/speech-markers-projection";

describe("governed desktop controls and unchanged API inputs", () => {
  it("accepts exact independent mask/effect defaults in native cross-language inputs", () => {
    expect(maskSchema.parse(catalog.defaultMask)).toEqual(catalog.defaultMask);
    for (const effect of Object.values(catalog.effectDefaults)) {
      expect(visualEffectSchema.parse(effect)).toEqual(effect);
      expect(
        headlessEditSchema.parse({
          effects: [effect],
          itemId: "leaf",
          operation: "update_item",
        })
      ).toEqual({
        effects: [effect],
        itemId: "leaf",
        operation: "update_item",
      });
      expect(
        schemas.timelineUpdateItem.parse({
          effects: [effect],
          expectedRevision: 0,
          itemId: "leaf",
          projectId: "p",
        }).effects
      ).toEqual([effect]);
    }
    expect(catalog.projectSchemaVersion).toBe(38);
    expect(catalog.headlessProtocolVersion).toBe(1);
    expect(catalog.registeredToolCount).toBe(78);
    expect(
      (projectSpeechMarkerMcpPredecessor(surface) as typeof surface).tools
    ).toHaveLength(78);
  });
  it("preserves omission clear exact floating values and maximal seed across public inputs", () => {
    for (const blendMode of catalog.blendModes) {
      expect(blendModeSchema.parse(blendMode)).toBe(blendMode);
    }
    const effect = {
      ...catalog.effectDefaults.particle_overlay,
      color: {
        a: 0.876_543_210_987_654_3,
        b: 0.987_654_321_098_765_4,
        g: 0.333_333_333_333_333_3,
        r: 0.123_456_789_012_345_66,
      },
      seed: 4_294_967_295,
    };
    expect(visualEffectSchema.parse(effect)).toEqual(effect);
    for (const key of ["clip", "matte"] as const) {
      for (const value of [undefined, null]) {
        const request = {
          itemId: "leaf",
          operation: "update_item",
          ...(value === undefined ? {} : { [key]: value }),
        };
        expect(headlessEditSchema.parse(request)).toEqual(request);
      }
    }
    expect(catalog.effectFields.particle_overlay.seed).toBe("u32");
    expect(catalog.effectFields.particle_overlay.count).toBe("u16");
  });
  it("retains core semantic limits and strict representations instead of coercion", () => {
    for (const seed of [-1, 1.5, 4_294_967_296, "4294967295"]) {
      expect(
        visualEffectSchema.safeParse({
          ...catalog.effectDefaults.particle_overlay,
          seed,
        }).success
      ).toBe(false);
    }
    for (const count of [257, 65_535, -1, 1.5]) {
      expect(
        visualEffectSchema.safeParse({
          ...catalog.effectDefaults.particle_overlay,
          count,
        }).success
      ).toBe(false);
    }
    for (const radiusPx of [
      Number.NaN,
      Number.POSITIVE_INFINITY,
      Number.NEGATIVE_INFINITY,
    ]) {
      expect(
        visualEffectSchema.safeParse({
          ...catalog.effectDefaults.gaussian_blur,
          radiusPx,
        }).success
      ).toBe(false);
    }
    // Identity/reference validation stays in core; transport preserves the complete authored collection.
    expect(
      schemas.timelineUpdateItem.parse({
        expectedRevision: 0,
        itemId: "leaf",
        masks: [catalog.defaultMask, catalog.defaultMask],
        projectId: "p",
      }).masks
    ).toEqual([catalog.defaultMask, catalog.defaultMask]);
  });
});
