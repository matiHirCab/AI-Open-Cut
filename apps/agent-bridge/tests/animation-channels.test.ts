import { describe, expect, it } from "vitest";
import contract from "../../../contracts/animation-channels-v1.json";
import audioEvents from "../../../contracts/timeline-audio-events-v1.json";
import {
  animationChannelPropertySchema,
  animationChannelSchema,
  headlessEditSchema,
  schemas,
  timelineItemSchema,
} from "../src/schemas";
import { restoreAudioBusCatalogMarker } from "./fixtures/audio-buses-projection";

const channel = (property: string, value: number) => ({
  keyframes: [{ curve: "linear", timeMs: 0, value: { type: "scalar", value } }],
  property,
});

describe("governed animation channels", () => {
  it("matches every canonical channel name and limit", () => {
    expect(contract.projectSchemaVersion).toBe(
      audioEvents.projectSchemaVersion
    );
    expect(restoreAudioBusCatalogMarker(contract).projectSchemaVersion).toBe(
      38
    );
    const names = [
      ...Object.keys(contract.active),
      ...Object.keys(contract.inactive),
    ].sort();
    expect(animationChannelPropertySchema.options.slice().sort()).toEqual(
      names
    );
    expect(names).toHaveLength(44);
    expect(contract.limits).toEqual({
      maxChannelsPerItem: 64,
      maxGradientStops: 32,
      maxKeyframesPerChannel: 1000,
      maxLoopIterations: 10_000,
      maxPathPoints: 4096,
    });
    expect(contract.active).toMatchObject({
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
    expect(contract.curves).toEqual([
      "hold",
      "linear",
      "cubic_bezier",
      "spring",
    ]);
    expect(contract.curveParameters).toEqual({
      cubic_bezier: {
        iterations: 40,
        x1: [0, 1],
        x1AtMostX2: true,
        x2: [0, 1],
        y1: [0, 1],
        y2: [0, 1],
      },
      spring: {
        damping: [0.01, 1000],
        initialVelocity: [-100, 100],
        mass: [0.01, 100],
        stiffness: [0.01, 10_000],
      },
    });
    for (const testCase of contract.curveCases) {
      const value = {
        keyframes: [
          {
            curve: testCase.curve,
            timeMs: 0,
            value: { type: "scalar", value: 0 },
          },
          { curve: "hold", timeMs: 500, value: { type: "scalar", value: 100 } },
        ],
        property: "transform.position_x",
      };
      expect(
        animationChannelSchema.safeParse(value).success,
        testCase.name
      ).toBe(testCase.accepted);
    }
    expect(
      animationChannelSchema.safeParse(contract.examples.validBezier).success
    ).toBe(true);
    expect(
      animationChannelSchema.safeParse(contract.examples.validSpring).success
    ).toBe(true);
    expect(contract.loop.modes).toEqual(["repeat", "ping_pong"]);
    expect(
      animationChannelSchema.safeParse(contract.examples.validRepeatLoop)
        .success
    ).toBe(true);
    expect(
      animationChannelSchema.safeParse(contract.examples.validInfinitePingPong)
        .success
    ).toBe(true);
    expect(
      animationChannelSchema.safeParse(contract.examples.validMaxRepeatLoop)
        .success
    ).toBe(true);
    expect(
      animationChannelSchema.safeParse(
        contract.examples.invalidLoopUnknownField
      ).success
    ).toBe(false);
    expect(
      animationChannelSchema.safeParse(contract.examples.invalidLoopCount)
        .success
    ).toBe(false);
    // Endpoint equality is an editor-core semantic check, not a Zod shape rule.
    expect(
      animationChannelSchema.safeParse(contract.examples.invalidLoopEndpoint)
        .success
    ).toBe(true);
    expect(
      animationChannelSchema.safeParse(contract.examples.invalidSpring).success
    ).toBe(false);
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
      { ...value, loop: { iterations: 0, mode: "repeat" } },
      { ...value, loop: { iterations: 10_001, mode: "repeat" } },
      { ...value, loop: { iterations: 1.5, mode: "repeat" } },
      { ...value, loop: { iterations: 2, mode: "reverse" } },
      { ...value, loop: { expression: "t", iterations: 2, mode: "repeat" } },
      { ...value, keyframes: [{ ...value.keyframes[0], curve: "spring" }] },
      {
        ...value,
        keyframes: [
          {
            ...value.keyframes[0],
            curve: {
              damping: 26,
              initialVelocity: 0,
              mass: 0,
              stiffness: 170,
              type: "spring",
            },
          },
        ],
      },
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

it("matches retained source clock structural fixtures without duplicating core semantics", () => {
  for (const fixture of contract.clockCases) {
    const payload = { ...contract.examples.validBezier, clock: fixture.clock };
    expect(
      animationChannelSchema.safeParse(payload).success,
      fixture.name
    ).toBe(fixture.accepted);
    expect(
      schemas.timelineSetAnimationChannels.safeParse({
        animationChannels: [payload],
        expectedRevision: 0,
        itemId: "i",
        projectId: "p",
      }).success,
      fixture.name
    ).toBe(fixture.accepted);
    expect(
      headlessEditSchema.safeParse({
        animationChannels: [payload],
        itemId: "@seed",
        operation: "set_animation_channels",
      }).success,
      fixture.name
    ).toBe(fixture.accepted);
  }
  expect(
    animationChannelSchema.parse(contract.examples.validRetainedClock)
  ).toEqual(contract.examples.validRetainedClock);
  expect(
    animationChannelSchema.parse(contract.examples.validNegativeRetainedClock)
  ).toEqual(contract.examples.validNegativeRetainedClock);
});

it("preserves legacy visual clocks in strict response shapes", () => {
  const item = {
    color: "#ffffff",
    durationMs: 500,
    height: 32,
    hidden: false,
    id: "box",
    keyframes: [
      {
        easing: "linear",
        property: "position",
        timeMs: 0,
        value: { type: "position", x: 0, y: 0 },
      },
    ],
    stackOrder: 0,
    startMs: 125,
    transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
    type: "rectangle",
    width: 32,
    zIndex: 0,
  };
  for (const fixture of contract.clockCases) {
    const payload = { ...item, legacyAnimationClock: fixture.clock };
    expect(timelineItemSchema.safeParse(payload).success, fixture.name).toBe(
      fixture.accepted
    );
  }
});
