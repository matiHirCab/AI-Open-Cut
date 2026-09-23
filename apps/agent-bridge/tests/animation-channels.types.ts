import type { HeadlessEdit } from "../src/headless-contract";

export const valid: HeadlessEdit = {
  animationChannels: [
    {
      keyframes: [
        { curve: "linear", timeMs: 0, value: { type: "scalar", value: 10 } },
      ],
      property: "transform.position_x",
    },
  ],
  itemId: "item",
  operation: "set_animation_channels",
};

export const unknownChannel: HeadlessEdit = {
  animationChannels: [
    // @ts-expect-error Unknown channel properties are rejected by the typed union.
    { keyframes: [], property: "transform.not_a_channel" },
  ],
  itemId: "item",
  operation: "set_animation_channels",
};

export const unknownValueTag: HeadlessEdit = {
  animationChannels: [
    {
      keyframes: [
        {
          curve: "hold",
          timeMs: 0,
          // @ts-expect-error Unknown value tags are rejected by the closed value union.
          value: { type: "expression", value: "t" },
        },
      ],
      property: "transform.position_x",
    },
  ],
  itemId: "item",
  operation: "set_animation_channels",
};

export const missingTime: HeadlessEdit = {
  animationChannels: [
    {
      keyframes: [
        // @ts-expect-error Keyframe time is mandatory.
        { curve: "hold", value: { type: "scalar", value: 10 } },
      ],
      property: "transform.position_x",
    },
  ],
  itemId: "item",
  operation: "set_animation_channels",
};
