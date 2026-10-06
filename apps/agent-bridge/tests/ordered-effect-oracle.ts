type Pixel = [number, number, number, number];
interface Effect {
  amount?: number;
  color?: { r: number; g: number; b: number; a: number };
  radiusPx?: number;
  type: string;
}
const SIDE = 48,
  ORIGIN = -8;
const at = <T>(values: ArrayLike<T>, position: number): T => {
  const value = values[position];
  if (value === undefined) {
    throw new Error("Independent oracle index outside authored bounds");
  }
  return value;
};
const index = (x: number, y: number) => y * SIDE + x;
const encoded = (linear: number) =>
  Math.round(
    255 *
      (linear <= 0.003_130_8
        ? 12.92 * linear
        : 1.055 * linear ** (1 / 2.4) - 0.055)
  );
const toLinear = (v: number) =>
  v <= 0.040_45 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
const coverage = (x: number, y: number): number => {
  if (x < 0 || x >= 32 || y < 0 || y >= 24 || y - x > 16) {
    return 0;
  }
  return y - x === 16 ? 96 / 255 : 1;
};
const vignette = (source: Pixel[], amount: number) => {
  for (const [i, p] of source.entries()) {
    const u = (2 * ((i % SIDE) + ORIGIN + 0.5)) / 32 - 1;
    const v = (2 * (Math.floor(i / SIDE) + ORIGIN + 0.5)) / 24 - 1;
    const f = 1 - amount * Math.max(0, Math.min(1, (u * u + v * v) / 2));
    for (const c of [0, 1, 2] as const) {
      p[c] *= f;
    }
  }
};
const tint = (source: Pixel[], color: NonNullable<Effect["color"]>) => {
  const { r, g, b, a: mix } = color;
  const target = [r, g, b].map(toLinear);
  for (const p of source) {
    for (const c of [0, 1, 2] as const) {
      p[c] = p[c] * (1 - mix) + p[3] * mix * at(target, c);
    }
  }
};
const convolve = (
  values: ArrayLike<number>,
  kernel: readonly number[],
  horizontal: boolean
) => {
  const output = new Float64Array(SIDE * SIDE);
  for (let y = 0; y < SIDE; y += 1) {
    for (let x = 0; x < SIDE; x += 1) {
      for (const [tap, w] of kernel.entries()) {
        const sx = horizontal ? x + tap - 3 : x;
        const sy = horizontal ? y : y + tap - 3;
        if (sx >= 0 && sx < SIDE && sy >= 0 && sy < SIDE) {
          output[index(x, y)] =
            at(output, index(x, y)) + at(values, index(sx, sy)) * w;
        }
      }
    }
  }
  return output;
};
const glow = (source: Pixel[]) => {
  const weights = Array.from({ length: 7 }, (_, i) =>
    Math.exp(-0.5 * (i - 3) ** 2)
  );
  const sum = weights.reduce((a, b) => a + b, 0),
    kernel = weights.map((w) => w / sum);
  const blurred = convolve(
    convolve(
      source.map((p) => p[3]),
      kernel,
      true
    ),
    kernel,
    false
  );
  for (const [i, p] of source.entries()) {
    const behind = at(blurred, i) * 0.6 * (1 - p[3]);
    p[2] += behind;
    p[3] += behind;
  }
};
const apply = (source: Pixel[], effect: Effect) => {
  if (effect.type === "vignette" && effect.amount !== undefined) {
    vignette(source, effect.amount);
  } else if (effect.type === "color_tint" && effect.color) {
    tint(source, effect.color);
  } else if (effect.type === "glow" && effect.radiusPx === 1) {
    glow(source);
  } else if (effect.type !== "gaussian_blur" || effect.radiusPx !== 0) {
    throw new Error("effect outside independent fixture");
  }
};
/** Independent authored geometry, canonical4x4 boundary rule and linear equations. */
export const independentOrderedEffectPlate = (
  stack: readonly Effect[]
): Uint8Array => {
  const source = Array.from({ length: SIDE * SIDE }, (_, i): Pixel => {
    const a = coverage((i % SIDE) + ORIGIN, Math.floor(i / SIDE) + ORIGIN);
    return [a, 0, 0, a];
  });
  for (const effect of stack) {
    apply(source, effect);
  }
  const plate = new Uint8Array(64 * 64 * 4);
  for (let i = 3; i < plate.length; i += 4) {
    plate[i] = 255;
  }
  for (const [i, p] of source.entries()) {
    const x = 37 - (Math.floor(i / SIDE) + ORIGIN),
      y = 12 + ((i % SIDE) + ORIGIN);
    if (x >= 0 && x < 64 && y >= 0 && y < 64) {
      for (const c of [0, 1, 2] as const) {
        plate[(y * 64 + x) * 4 + c] = encoded(p[c] * 0.5);
      }
    }
  }
  return plate;
};
