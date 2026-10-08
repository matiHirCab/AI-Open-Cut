import { describe, expect, it } from "vitest";
import PRESETS from "../../../contracts/animation-presets-v1.json";
import audioBuses from "../../../contracts/audio-buses-v1.json";
import PACK from "../../../contracts/initial-motion-preset-pack-v1.json";
import type { HeadlessEdit } from "../src/headless-contract";
import {
  animationPresetParametersSchema,
  animationPresetPropertySchema,
  headlessEditSchema,
  schemas,
} from "../src/schemas";
import { restoreAudioBusCatalogMarker } from "./fixtures/audio-buses-projection";

describe("canonical versioned animation presets", () => {
  it("matches the fixed catalog and materializes only declared defaults", () => {
    expect(animationPresetPropertySchema.options).toEqual(
      PRESETS.presets[0]?.properties
    );
    expect(headlessEditSchema.parse(PRESETS.examples.apply)).toEqual({
      ...PRESETS.examples.apply,
      collisionPolicy: "reject",
      parameters: { ...PRESETS.examples.apply.parameters, curve: "linear" },
    });
    expect(PRESETS.examples.resolvedChannel.keyframes).toEqual([
      { curve: "linear", timeMs: 0, value: { type: "scalar", value: 0 } },
      { curve: "hold", timeMs: 500, value: { type: "scalar", value: 1 } },
    ]);
    expect(PRESETS.compilerVersion).toBe(2);
    expect(PRESETS.projectSchemaVersion).toBe(audioBuses.projectSchemaVersion);
    expect(restoreAudioBusCatalogMarker(PRESETS).projectSchemaVersion).toBe(38);
    expect(PACK.projectSchemaVersion).toBe(audioBuses.projectSchemaVersion);
    expect(restoreAudioBusCatalogMarker(PACK).projectSchemaVersion).toBe(38);
  });

  it("accepts structurally valid unknown identities for canonical core rejection", () => {
    expect(
      schemas.timelineApplyAnimationPreset.safeParse({
        ...PRESETS.examples.apply,
        operation: undefined,
      }).success
    ).toBe(false);
    const { operation: _operation, ...edit } = PRESETS.examples.apply;
    expect(
      schemas.timelineApplyAnimationPreset.safeParse({
        ...edit,
        expectedRevision: 1,
        presetId: "unknown_seed",
        presetVersion: 2,
        projectId: "p",
      }).success
    ).toBe(true);
  });

  it("rejects malformed parameters without interpreting executable input", () => {
    for (const parameters of [
      { ...PRESETS.examples.apply.parameters, from: Number.NaN },
      { ...PRESETS.examples.apply.parameters, to: Number.POSITIVE_INFINITY },
      {
        ...PRESETS.examples.apply.parameters,
        startMs: Number.MAX_SAFE_INTEGER + 1,
      },
      { ...PRESETS.examples.apply.parameters, durationMs: 0 },
      { ...PRESETS.examples.apply.parameters, property: "graphic.path_points" },
      { ...PRESETS.examples.apply.parameters, expression: "t*2" },
      {
        ...PRESETS.examples.apply.parameters,
        curve: {
          damping: 20,
          initialVelocity: 0,
          mass: 1,
          path: "https://example.com",
          stiffness: 100,
          type: "spring",
        },
      },
    ]) {
      expect(
        animationPresetParametersSchema.safeParse(parameters).success
      ).toBe(false);
    }
  });
});

// Compile-time fixtures exercise the exported transport union, not only Zod parsing.
const validEdit: HeadlessEdit = {
  itemId: "item",
  operation: "apply_animation_preset",
  parameters: {
    durationMs: 500,
    from: 0,
    property: "transform.opacity",
    startMs: 0,
    to: 1,
  },
  presetId: "scalar_tween",
  presetVersion: 1,
};
expect(validEdit.presetVersion).toBe(1);
// @ts-expect-error Version is mandatory in the typed preset edit.
const missingVersion: HeadlessEdit = {
  itemId: "item",
  operation: "apply_animation_preset",
  parameters: validEdit.parameters,
  presetId: "scalar_tween",
};
expect(missingVersion.operation).toBe("apply_animation_preset");

// These negatives pin the public caller types without broadening the runtime schemas.
type PresetParameters = Extract<
  HeadlessEdit,
  { operation: "apply_animation_preset" }
>["parameters"];
// @ts-expect-error The seed excludes deferred properties.
const unsupportedProperty: PresetParameters["property"] = "graphic.path_points";
// @ts-expect-error Endpoints are scalar numbers.
const wrongValue: PresetParameters["from"] = "0";
// @ts-expect-error Executable curve tags are excluded.
const wrongCurve: PresetParameters["curve"] = "expression";
expect([unsupportedProperty, wrongValue, wrongCurve]).toHaveLength(3);

describe("initial motion pack typed contract", () => {
  it("accepts all five closed inputs without transport expansion or defaulted loops", () => {
    for (const entry of PACK.presets) {
      expect(animationPresetParametersSchema.parse(entry.parameters)).toEqual(
        entry.parameters
      );
      const request = {
        itemId: "item",
        operation: "apply_animation_preset",
        parameters: entry.parameters,
        presetId: entry.id,
        presetVersion: 1,
      };
      expect(headlessEditSchema.parse(request)).toEqual({
        ...request,
        collisionPolicy: "reject",
      });
      expect(
        schemas.timelineApplyAnimationPreset.parse({
          expectedRevision: 1,
          itemId: "item",
          parameters: entry.parameters,
          presetId: entry.id,
          presetVersion: 1,
          projectId: "project",
        })
      ).toMatchObject({ parameters: entry.parameters });
      expect(
        animationPresetParametersSchema.safeParse({
          ...entry.parameters,
          expression: "time",
        }).success
      ).toBe(false);
      expect(
        animationPresetParametersSchema.safeParse(
          Object.values(entry.parameters)
        ).success
      ).toBe(false);
    }
  });
  it("rejects nested positional blur, unknown fields and missing tagged endpoints", () => {
    const impact = PACK.presets[0]?.parameters;
    expect(
      animationPresetParametersSchema.safeParse({
        ...impact,
        motionBlur: [180, 3],
      }).success
    ).toBe(false);
    expect(
      animationPresetParametersSchema.safeParse({
        ...impact,
        motionBlur: { enabled: true, sampleCount: 3, shutterAngleDeg: 180 },
      }).success
    ).toBe(false);
    expect(
      animationPresetParametersSchema.safeParse({
        durationMs: 100,
        kind: "pulse",
        scaleFrom: 1,
        startMs: 0,
      }).success
    ).toBe(false);
  });
});
