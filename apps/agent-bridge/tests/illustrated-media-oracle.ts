import type { TargetId } from "./fixtures/illustrated-targets";

const WIDTH = 1920,
  HEIGHT = 1080;
export const FRAME_BYTES = WIDTH * HEIGHT * 3;
const rgb = (pixels: Buffer, x: number, y: number) => [
  ...pixels.subarray((y * WIDTH + x) * 3, (y * WIDTH + x) * 3 + 3),
];
const near = (
  pixels: Buffer,
  x: number,
  y: number,
  expected: number[],
  tolerance: number
) => {
  const actual = rgb(pixels, x, y);
  if (
    actual.length !== 3 ||
    actual.some((v, i) => Math.abs(v - (expected[i] ?? -999)) > tolerance)
  ) {
    throw new Error(
      `Independent illustrated pixel ${x},${y}: ${actual} != ${expected}`
    );
  }
};
const glyphs = (pixels: Buffer, box: number[]) => {
  let count = 0;
  for (let y = box[1] ?? 0; y < (box[3] ?? 0); y += 1) {
    for (let x = box[0] ?? 0; x < (box[2] ?? 0); x += 1) {
      if (rgb(pixels, x, y).every((channel) => channel >= 240)) {
        count += 1;
      }
    }
  }
  if (count < 100) {
    throw new Error("Independent illustrated label glyphs missing");
  }
};
export const verifyIllustratedPixels = (
  id: TargetId,
  pixels: Buffer,
  timeMs: number,
  tolerance = 3
) => {
  if (pixels.length !== FRAME_BYTES) {
    throw new Error("Wrong independent illustrated frame size");
  }
  near(pixels, 100, 100, [24, 38, 61], tolerance);
  if (id === "illustrated_device_intro") {
    for (const [x, y, color] of [
      [280, 200, [238, 238, 238]],
      [350, 250, [22, 32, 51]],
      [1250, 570, [48, 73, 104]],
      [450, 785, [255, 204, 51]],
      [650, 400, [255, 255, 255]],
      [840, 400, [255, 0, 0]],
      [1145, 500, [24, 38, 61]],
    ] as const) {
      near(pixels, x, y, [...color], tolerance);
    }
    // Independent moving-accent control at exact final sample times.
    near(
      pixels,
      310,
      785,
      timeMs === 1500 ? [238, 238, 238] : [255, 204, 51],
      tolerance
    );
    glyphs(pixels, [1230, 330, 1670, 430]);
  } else if (id === "sticker_motif_intro") {
    for (const [x, y, color] of [
      [120, 120, [24, 38, 61]],
      [320, 320, [0, 212, 255]],
      [960, 110, [255, 204, 51]],
      [200, 396, [255, 107, 0]],
      [600, 500, [48, 73, 104]],
      [605, 185, [48, 73, 104]],
      [585, 165, [24, 38, 61]],
      [850, 450, [255, 255, 255]],
      [1040, 450, [255, 0, 0]],
      [1450, 500, [24, 38, 61]],
    ] as const) {
      near(pixels, x, y, [...color], tolerance);
    }
    glyphs(pixels, [650, 940, 1350, 1020]);
  } else {
    for (const left of [160, 720, 1280]) {
      for (const [dx, y, color] of [
        [30, 433, [238, 238, 238]],
        [40, 450, [22, 32, 51]],
        [30, 695, [48, 73, 104]],
        [80, 668, [255, 204, 51]],
        [180, 530, [255, 255, 255]],
        [280, 530, [255, 0, 0]],
        [384, 475, [255, 107, 0]],
        [384, 430, [255, 204, 0]],
      ] as const) {
        near(pixels, left + dx, y, [...color], tolerance);
      }
      glyphs(pixels, [left + 38, 701, left + 441, 742]);
    }
    near(pixels, 680, 600, [24, 38, 61], tolerance);
    glyphs(pixels, [600, 100, 1500, 200]);
  }
};
export const syntheticPcm = () => {
  const data = new Float32Array(144_000);
  for (let sample = 5760; sample < 136_800; sample += 1) {
    const t = sample / 48_000;
    // Chirp plus coded amplitude envelope prevents a periodic tone from hiding
    // wrong-clock alignment. The onset is independently fixed at 120ms.
    const gain =
      [0.12, 0.21, 0.16, 0.09, 0.19, 0.14, 0.23][Math.floor(t * 10) % 7] ?? 0;
    const ramp = Math.min(1, (sample - 5760) / 240, (136_800 - sample) / 240);
    data[sample] =
      gain *
      ramp *
      (Math.sin(2 * Math.PI * (437 * t + 75 * t * t)) +
        0.25 * Math.sin(2 * Math.PI * 1103 * t));
  }
  return data;
};
export const syntheticWav = (data: Float32Array) => {
  const bytes = Buffer.alloc(44 + data.length * 4);
  bytes.write("RIFF", 0);
  bytes.writeUInt32LE(bytes.length - 8, 4);
  bytes.write("WAVEfmt ", 8);
  bytes.writeUInt32LE(16, 16);
  bytes.writeUInt16LE(1, 20);
  bytes.writeUInt16LE(2, 22);
  bytes.writeUInt32LE(48_000, 24);
  bytes.writeUInt32LE(192_000, 28);
  bytes.writeUInt16LE(4, 32);
  bytes.writeUInt16LE(16, 34);
  bytes.write("data", 36);
  bytes.writeUInt32LE(data.length * 4, 40);
  for (let i = 0; i < data.length; i += 1) {
    const value = Math.round((data[i] ?? 0) * 32_767);
    bytes.writeInt16LE(value, 44 + i * 4);
    bytes.writeInt16LE(value, 46 + i * 4);
  }
  return bytes;
};
export const verifyOriginalAudio = (decoded: Buffer, source: Float32Array) => {
  const length = decoded.length / 8;
  if (!Number.isInteger(length) || length < source.length - 1600) {
    throw new Error("Original audio omitted or truncated");
  }
  const verifyChannel = (channel: number) => {
    const correlation = (lag: number) => {
      let dot = 0,
        observedEnergy = 0,
        sourceEnergy = 0;
      for (
        let i = 2416;
        i < Math.min(source.length - 2400, length - 2416);
        i += 16
      ) {
        const observed = decoded.readFloatLE((i + lag) * 8 + channel * 4),
          expected = source[i] ?? 0;
        if (!Number.isFinite(observed)) {
          throw new Error("Nonfinite original audio");
        }
        dot += observed * expected;
        observedEnergy += observed * observed;
        sourceEnergy += expected * expected;
      }
      return dot / Math.sqrt(observedEnergy * sourceEnergy);
    };
    let best = -1,
      alignmentSamples = 0;
    for (let lag = -2400; lag <= 2400; lag += 16) {
      const rho = correlation(lag);
      if (rho > best) {
        best = rho;
        alignmentSamples = lag;
      }
    }
    const coarse = alignmentSamples;
    for (let lag = coarse - 15; lag <= coarse + 15; lag += 1) {
      const rho = correlation(lag);
      if (rho > best) {
        best = rho;
        alignmentSamples = lag;
      }
    }
    if (!(best >= 0.99)) {
      throw new Error("Original PCM waveform correlation failed");
    }
    if (Math.abs(alignmentSamples) > 1600) {
      throw new Error("Original audio clock exceeds one output frame");
    }
    return {
      alignmentMs: alignmentSamples / 48,
      alignmentSamples,
      correlation: best,
    };
  };
  const left = verifyChannel(0),
    right = verifyChannel(1);
  const alignmentSamples =
    Math.abs(left.alignmentSamples) >= Math.abs(right.alignmentSamples)
      ? left.alignmentSamples
      : right.alignmentSamples;
  return {
    alignmentMs: alignmentSamples / 48,
    alignmentSamples,
    correlation: Math.min(left.correlation, right.correlation),
  };
};
