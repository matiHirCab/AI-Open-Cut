import type { z } from "zod/v4";
import type { visualEffectSchema } from "../src/schemas";

// Independently frozen seed1/index0..7/lane0..2 words, never runtime hash output.
const words = [
  [2_336_985_851, 938_826_240, 1_456_866_664],
  [84_929_298, 829_401_672, 2_771_797_279],
  [2_877_292_033, 4_199_349_311, 3_397_877_525],
  [4_067_787_588, 1_783_773_585, 2_445_685_191],
  [3_863_548_237, 661_299_992, 4_101_546_638],
  [2_606_412_158, 1_268_270_209, 2_067_976_603],
  [2_944_314_353, 2_443_517_269, 3_235_942_571],
  [2_958_880_360, 2_302_107_493, 891_655_201],
] as const;
const linear = (v: number) =>
  v <= 0.040_45 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
const encode = (v: number) =>
  Math.round(
    255 * (v <= 0.003_130_8 ? 12.92 * v : 1.055 * v ** (1 / 2.4) - 0.055)
  );

type Effect = z.infer<typeof visualEffectSchema>;
const side = 74;
const eachPixel = (visit: (x: number, y: number, i: number) => void) => {
  for (let y = 0; y < side; y += 1) {
    for (let x = 0; x < side; x += 1) {
      visit(x, y, (y * side + x) * 4);
    }
  }
};
const flashPlane = (
  plane: Float64Array,
  effect: Extract<Effect, { type: "screen_flash" }>,
  time: number
) => {
  const strength =
    time >= effect.startMs && time < effect.startMs + effect.durationMs
      ? effect.intensity *
        effect.color.a *
        (1 - (time - effect.startMs) / effect.durationMs)
      : 0;
  const color = [effect.color.r, effect.color.g, effect.color.b].map(linear);
  eachPixel((_x, _y, i) => {
    for (let c = 0; c < 3; c += 1) {
      plane[i + c] =
        (plane[i + c] ?? 0) +
        ((plane[i + 3] ?? 0) - (plane[i + c] ?? 0)) *
          (color[c] ?? 0) *
          strength;
    }
  });
};
const coverage = (
  x: number,
  y: number,
  cx: number,
  cy: number,
  radius: number
) => {
  let covered = 0;
  for (let row = 0; row < 4; row += 1) {
    for (let col = 0; col < 4; col += 1) {
      const dx = x - 5 + (col + 0.5) / 4 - cx,
        dy = y - 5 + (row + 0.5) / 4 - cy;
      if (dx * dx + dy * dy <= radius * radius) {
        covered += 1;
      }
    }
  }
  return covered / 16;
};
const particlePlane = (
  plane: Float64Array,
  effect: Extract<Effect, { type: "particle_overlay" }>,
  time: number
) => {
  if (effect.seed !== 1 || effect.count !== 8) {
    throw new Error("Independent witness requires frozen seed/count");
  }
  const color = [effect.color.r, effect.color.g, effect.color.b].map(linear);
  for (const [a, b, c] of words) {
    const cx = (64 * a) / 4_294_967_296,
      phase =
        ((time % effect.lifetimeMs) + (c / 4_294_967_296) * effect.lifetimeMs) %
        effect.lifetimeMs;
    const cy =
      ((64 * b) / 4_294_967_296 + (effect.speedPxPerSecond * phase) / 1000) %
      64;
    eachPixel((x, y, i) => {
      const alpha = effect.color.a * coverage(x, y, cx, cy, effect.radiusPx);
      for (let channel = 0; channel < 3; channel += 1) {
        plane[i + channel] =
          (color[channel] ?? 0) * alpha +
          (plane[i + channel] ?? 0) * (1 - alpha);
      }
      plane[i + 3] = alpha + (plane[i + 3] ?? 0) * (1 - alpha);
    });
  }
};
const gaussianPass = (
  input: Float64Array,
  kernel: number[],
  vertical: boolean
) => {
  const output = new Float64Array(input.length);
  eachPixel((x, y, i) => {
    for (let k = 0; k < 7; k += 1) {
      const sx = x + (vertical ? 0 : k - 3),
        sy = y + (vertical ? k - 3 : 0);
      if (sx < 0 || sx >= side || sy < 0 || sy >= side) {
        continue;
      }
      for (let c = 0; c < 4; c += 1) {
        output[i + c] =
          (output[i + c] ?? 0) +
          (input[(sy * side + sx) * 4 + c] ?? 0) * (kernel[k] ?? 0);
      }
    }
  });
  return output;
};
const apply = (plane: Float64Array, effect: Effect, time: number) => {
  switch (effect.type) {
    case "screen_flash":
      flashPlane(plane, effect, time);
      return plane;
    case "particle_overlay":
      particlePlane(plane, effect, time);
      return plane;
    case "gaussian_blur": {
      if (effect.radiusPx !== 1) {
        throw new Error("Independent witness requires positive sigma1");
      }
      const kernel = Array.from({ length: 7 }, (_, i) =>
        Math.exp(-0.5 * (i - 3) ** 2)
      );
      const sum = kernel.reduce((a, b) => a + b, 0),
        normalized = kernel.map((weight) => weight / sum);
      return gaussianPass(
        gaussianPass(plane, normalized, false),
        normalized,
        true
      );
    }
    default:
      throw new Error(`Unsupported independent witness effect ${effect.type}`);
  }
};
/** Normative equations on an independent padded plane, then owner gain once. */
export function independentGroupPlate(stack: Effect[], time: number) {
  let plane: Float64Array = new Float64Array(side * side * 4);
  for (let y = 0; y < 24; y += 1) {
    for (let x = 0; x < 32; x += 1) {
      const i = ((y + 13) * side + x + 9) * 4;
      plane[i + 1] = 0.4;
      plane[i + 3] = 0.4;
    }
  }
  for (const effect of stack) {
    plane = apply(plane, effect, time);
  }
  const result = new Uint8Array(64 * 64 * 4);
  for (let y = 0; y < 64; y += 1) {
    for (let x = 0; x < 64; x += 1) {
      const i = (y * 64 + x) * 4,
        src = ((y + 5) * side + x + 5) * 4;
      for (let c = 0; c < 3; c += 1) {
        result[i + c] = encode((plane[src + c] ?? 0) * 0.65);
      }
      result[i + 3] = 255;
    }
  }
  return result;
}
