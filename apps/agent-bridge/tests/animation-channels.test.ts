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
  });

  it("accepts typed standalone and batch payloads and rejects open shapes", () => {
    const value = channel("transform.position_x", 10);
    expect(animationChannelSchema.parse(value)).toEqual(value);
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
