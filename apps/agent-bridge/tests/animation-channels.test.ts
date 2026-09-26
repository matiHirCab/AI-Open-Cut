import { describe, expect, it } from "vitest";
import contract from "../../../contracts/animation-channels-v1.json";
import {
  animationChannelPropertySchema,
  animationChannelSchema,
  headlessEditSchema,
  schemas,
} from "../src/schemas";

const channel = (property: string, value: number) => ({
  keyframes: [{ curve: "linear", timeMs: 0, value: { type: "scalar", value } }],
  property,
});

describe("governed animation channels", () => {
  it("matches every canonical channel name and limit", () => {
    expect(contract.projectSchemaVersion).toBe(22);
    const names = [
      ...Object.keys(contract.active),
      ...Object.keys(contract.inactive),
    ].sort();
    expect(animationChannelPropertySchema.options.slice().sort()).toEqual(
      names
    );
    expect(names).toHaveLength(29);
    expect(contract.limits).toEqual({
      maxChannelsPerItem: 64,
      maxGradientStops: 32,
      maxKeyframesPerChannel: 1000,
      maxPathPoints: 4096,
    });
    expect(contract.active).toEqual({
      "audio.gain_db": {
        activation: "active",
        maximum: 12,
        minimum: -96,
        target: "media_audio",
        valueType: "scalar",
      },
      "transform.opacity": {
        activation: "active",
        maximum: 1,
        minimum: 0,
        target: "visual_legacy",
        valueType: "scalar",
      },
      "transform.position_x": {
        activation: "active",
        maximum: 1_000_000,
        minimum: -1_000_000,
        target: "visual_legacy",
        valueType: "scalar",
      },
      "transform.position_y": {
        activation: "active",
        maximum: 1_000_000,
        minimum: -1_000_000,
        target: "visual_legacy",
        valueType: "scalar",
      },
      "transform.scale_x": {
        activation: "active",
        maximum: 100,
        minimumExclusive: 0,
        target: "visual_legacy",
        valueType: "scalar",
      },
      "transform.scale_y": {
        activation: "active",
        maximum: 100,
        minimumExclusive: 0,
        target: "visual_legacy",
        valueType: "scalar",
      },
    });
    for (const [name, metadata] of Object.entries(contract.inactive)) {
      let valueType = "scalar";
      if (name === "graphic.path_points") {
        valueType = "path_points";
      } else if (name === "graphic.gradient_stops") {
        valueType = "gradient_stops";
      } else if (
        [
          "graphic.fill_color",
          "graphic.stroke_color",
          "effect.tint_color",
        ].includes(name)
      ) {
        valueType = "rgba";
      }
      let target = "media_audio";
      if (name.startsWith("transform.")) {
        target = "visual";
      } else if (
        ["media.source_position_ms", "media.playback_rate"].includes(name)
      ) {
        target = "media_source";
      } else if (name.startsWith("media.")) {
        target = "media_visual";
      } else if (name.startsWith("graphic.")) {
        target = "graphic_scoped";
      } else if (name.startsWith("effect.")) {
        target = "effect_scoped";
      }
      expect(metadata, name).toEqual({
        activation: "inactive",
        bounds: "deferred",
        target,
        valueType,
      });
    }
    expect(
      animationChannelSchema.safeParse(contract.examples.validChannel).success
    ).toBe(true);
    expect(
      animationChannelSchema.safeParse(contract.examples.invalidChannel).success
    ).toBe(false);
    expect(
      animationChannelSchema.safeParse(contract.examples.inactiveChannel)
        .success
    ).toBe(true);
  });

  it("accepts typed standalone and batch payloads and rejects open shapes", () => {
    const value = channel("transform.position_x", 10);
    expect(animationChannelSchema.parse(value)).toEqual(value);
    expect(
      animationChannelSchema.parse(channel("transform.rotation_deg", 10))
    ).toEqual(channel("transform.rotation_deg", 10));
    expect(
      schemas.timelineSetAnimationChannels.safeParse({
        animationChannels: [value],
        expectedRevision: 1,
        itemId: "item",
        projectId: "project",
      }).success
    ).toBe(true);
    expect(
      headlessEditSchema.safeParse({
        animationChannels: [value],
        itemId: "@created",
        operation: contract.operation,
      }).success
    ).toBe(true);
    for (const bad of [
      { ...value, unknown: true },
      { ...value, keyframes: [{ ...value.keyframes[0], curve: "spring" }] },
      { ...value, keyframes: [{ ...value.keyframes[0], timeMs: "@marker" }] },
      {
        ...value,
        keyframes: [{ ...value.keyframes[0], loop: { count: 2 } }],
      },
      {
        ...value,
        keyframes: [
          {
            ...value.keyframes[0],
            value: { type: "scalar", value: Number.POSITIVE_INFINITY },
          },
        ],
      },
      {
        ...value,
        keyframes: Array.from({ length: 1001 }, () => value.keyframes[0]),
      },
    ]) {
      expect(animationChannelSchema.safeParse(bad).success).toBe(false);
    }
  });
});
