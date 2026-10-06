/** Public DTO choices; editor-core owns eligibility and compositing semantics. */
export const BLEND_MODES = [
  "normal",
  "multiply",
  "screen",
  "overlay",
  "add",
  "darken",
  "lighten",
] as const;
export type BlendMode = (typeof BLEND_MODES)[number];
