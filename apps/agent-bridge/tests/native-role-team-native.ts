import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import { jobSchema } from "../src/schemas";

export interface TeamNative {
  evidence: string;
  ffmpeg: string;
  ffprobe: string;
  mode: string;
}
export type TeamCall = <T>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<T>
) => Promise<T>;
export const teamProjectInventory = (
  directory: string
): Record<string, string> => {
  const result: Record<string, string> = {};
  const walk = (relative: string) => {
    const path = join(directory, relative);
    for (const entry of readdirSync(path, { withFileTypes: true }).sort(
      (a, b) => a.name.localeCompare(b.name)
    )) {
      const child = join(relative, entry.name);
      if (entry.isDirectory()) {
        walk(child);
      } else {
        const bytes = readFileSync(join(directory, child));
        result[child.replaceAll("\\", "/")] =
          `${bytes.length}:${createHash("sha256").update(bytes).digest("hex")}`;
      }
    }
  };
  walk("");
  return result;
};
const WIDTH = 1920,
  HEIGHT = 1080,
  BYTES = WIDTH * HEIGHT * 3;
// Fixed independent interiors in half-scale original hero/sun geometry.
const placements = [
  [480, 160],
  [1040, 160],
  [480, 560],
  [1040, 560],
] as const;
const backgrounds = [
  [24, 38, 61],
  [32, 64, 64],
  [64, 32, 64],
  [64, 48, 32],
] as const;
const pixel = (data: Buffer, x: number, y: number) => [
  ...data.subarray((y * WIDTH + x) * 3, (y * WIDTH + x) * 3 + 3),
];
const near = (
  actual: number[],
  expected: readonly number[],
  tolerance: number
) => {
  for (let c = 0; c < 3; c += 1) {
    expect(
      Math.abs((actual[c] ?? -999) - (expected[c] ?? 999))
    ).toBeLessThanOrEqual(tolerance);
  }
};
const opaqueOverBlack = (channel: number, opacity: number) => {
  const s = channel / 255,
    linear = s <= 0.040_45 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  const lit = linear * (Math.round(opacity * 255) / 255);
  return Math.round(
    255 * (lit <= 0.003_130_8 ? 12.92 * lit : 1.055 * lit ** (1 / 2.4) - 0.055)
  );
};
const verifyEncodedYellowPresence = (data: Buffer, x: number, y: number) => {
  // Fixed interiors derive from the authored half-scale accent and top ray.
  for (const [left, right, top, bottom] of [
    [45, 105, 219, 224],
    [317, 323, 14, 30],
  ] as const) {
    let redMinusBlue = 0,
      greenMinusBlue = 0;
    for (let py = top; py < bottom; py += 1) {
      for (let px = left; px < right; px += 1) {
        const [r = 0, g = 0, b = 0] = pixel(data, x + px, y + py);
        redMinusBlue += r - b;
        greenMinusBlue += g - b;
      }
    }
    const area = (right - left) * (bottom - top);
    expect(redMinusBlue / area).toBeGreaterThanOrEqual(100);
    expect(greenMinusBlue / area).toBeGreaterThanOrEqual(80);
  }
};
const verifyOpaqueMember = (
  data: Buffer,
  x: number,
  y: number,
  tolerance: number,
  changed: boolean,
  encoded: boolean
) => {
  for (const [dx, dy, rgb] of [
    [encoded ? 27 : 25, 25, [238, 238, 238]],
    [35, 35, [22, 32, 51]],
    [25, 250, [48, 73, 104]],
    [180, 130, [255, 255, 255]],
    [230, 130, changed ? [0, 0, 255] : [255, 0, 0]],
    [320, 60, [255, 107, 0]],
  ] as const) {
    near(pixel(data, x + dx, y + dy), rgb, tolerance);
  }
  if (encoded) {
    verifyEncodedYellowPresence(data, x, y);
  } else {
    near(pixel(data, x + 40, y + 220), [255, 204, 51], tolerance);
    // y25 stays inside the top ray and five pixels inside the device panel.
    near(
      pixel(data, x + 320, y + 25),
      changed ? [238, 238, 238] : [255, 204, 0],
      tolerance
    );
  }
  let white = 0;
  for (let py = y + 248; py < y + 282; py += 1) {
    for (let px = x + 32; px < x + 367; px += 1) {
      if (pixel(data, px, py).every((c) => c > (encoded ? 200 : 240))) {
        white += 1;
      }
    }
  }
  expect(white).toBeGreaterThan(30);
};
export const verifyTeamPixels = (
  data: Buffer,
  count: number,
  timeMs: number,
  tolerance = 3,
  changed = false,
  shift: readonly [number, number] = [0, 0],
  encoded = false
) => {
  expect(data.length).toBe(BYTES);
  near(pixel(data, 10, 10), [0, 0, 0], tolerance);
  for (let i = 0; i < 4; i += 1) {
    const [left, top] = placements[i] ?? [0, 0];
    const x = left + shift[0],
      y = top + shift[1];
    const opacity =
      i < count ? Math.max(0, Math.min(1, (timeMs - 80 * i) / 100)) : 0;
    const bg = changed && i === 0 ? [0, 255, 0] : (backgrounds[i] ?? [0, 0, 0]);
    near(
      pixel(data, x + 5, y + 5),
      bg.map((c) => opaqueOverBlack(c, opacity)),
      tolerance
    );
    if (opacity === 1) {
      verifyOpaqueMember(data, x, y, tolerance, changed && i === 0, encoded);
    }
  }
};
export const verifyEncodedTeamPixels = (
  data: Buffer,
  count: number,
  timeMs: number
) => verifyTeamPixels(data, count, timeMs, 8, false, [0, 0], true);
const execute = (program: string, args: string[]) => {
  const r = spawnSync(program, args, { maxBuffer: BYTES * 13 });
  expect(r.status, r.error?.message ?? r.stderr.toString()).toBe(0);
  return r.stdout;
};
export const captureTeamArtifact = async (
  call: TeamCall,
  projects: string,
  projectId: string,
  revision: number,
  name: string,
  input: Record<string, unknown>
) => {
  const directory = join(projects, projectId);
  const baseline = teamProjectInventory(directory);
  const q = await call(
    name,
    { expectedRevision: revision, projectId, ...input },
    jobSchema
  );
  for (let attempt = 0; attempt < 1200; attempt += 1) {
    // biome-ignore lint/performance/noAwaitInLoops: Poll one immutable render request to its terminal state.
    const job = await call("job_get_status", { jobId: q.jobId }, jobSchema);
    if (["completed", "failed", "cancelled"].includes(job.status)) {
      expect(job.status, JSON.stringify(job.error)).toBe("completed");
      expect(job.revision).toBe(revision);
      if (!job.artifact) {
        throw new Error("Missing native team artifact");
      }
      const after = teamProjectInventory(directory);
      for (const [path, bytes] of Object.entries(baseline)) {
        expect(after[path], `Native render changed preexisting ${path}`).toBe(
          bytes
        );
      }
      expect(
        Object.keys(after).filter((path) => !Object.hasOwn(baseline, path))
      ).toEqual([job.artifact.relativePath.replaceAll("\\", "/")]);
      return join(projects, projectId, job.artifact.relativePath);
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("Native team request exceeded two minutes");
};
const verifyStaticTeamParity = (frame: Buffer, finalRangeFrame: Buffer) => {
  // Secondary parity comparison follows independent authored-color checks above.
  for (const [x, y] of placements) {
    for (const [dx, dy] of [
      [5, 5],
      [27, 25],
      [35, 35],
      [25, 250],
      [180, 130],
      [230, 130],
      [320, 60],
    ]) {
      near(
        pixel(finalRangeFrame, x + (dx ?? 0), y + (dy ?? 0)),
        pixel(frame, x + (dx ?? 0), y + (dy ?? 0)),
        8
      );
    }
  }
};

export const verifyTeamMedia = (
  native: TeamNative,
  minimum: string,
  maximum: string,
  changed: string,
  range: string,
  samples: string[],
  moved: string
) => {
  mkdirSync(native.evidence, { recursive: true });
  // Keep each incoming native witness before any verification can reject it.
  const incomingAttempt = Date.now();
  for (const [label, path] of [
    ["minimum", minimum],
    ["maximum", maximum],
    ["changed", changed],
    ["range", range],
    ["moved", moved],
    ...samples.map((samplePath, index) => [`sample-${index}`, samplePath]),
  ] as [string, string][]) {
    writeFileSync(
      join(
        native.evidence,
        `${native.mode}-native-role-team-incoming-${incomingAttempt}-${label}.${label === "range" ? "mp4" : "png"}`
      ),
      readFileSync(path)
    );
  }

  const decode = (path: string) =>
    execute(native.ffmpeg, [
      "-v",
      "error",
      "-i",
      path,
      "-f",
      "rawvideo",
      "-pix_fmt",
      "rgb24",
      "pipe:1",
    ]);
  const before = decode(maximum),
    after = decode(changed);
  verifyTeamPixels(decode(minimum), 1, 500);
  verifyTeamPixels(before, 4, 500);
  verifyTeamPixels(after, 4, 500, 3, true);
  for (let member = 1; member < 4; member += 1) {
    const [x, y] = placements[member] ?? [0, 0];
    for (let dy = 0; dy < 300; dy += 1) {
      const start = ((y + dy) * WIDTH + x) * 3;
      expect(
        after
          .subarray(start, start + 400 * 3)
          .equals(before.subarray(start, start + 400 * 3))
      ).toBe(true);
    }
  }
  const [x, y] = placements[0];
  let changedGlyph = false;
  for (let dy = 248; dy < 282; dy += 1) {
    const start = ((y + dy) * WIDTH + x + 32) * 3;
    if (
      !after
        .subarray(start, start + 335 * 3)
        .equals(before.subarray(start, start + 335 * 3))
    ) {
      changedGlyph = true;
    }
  }
  expect(changedGlyph).toBe(true);
  expect(samples).toHaveLength(4);
  const times = [40, 80, 120, 280];
  for (const [index, path] of samples.entries()) {
    verifyTeamPixels(decode(path), 4, times[index] ?? -1);
    writeFileSync(
      join(
        native.evidence,
        `${native.mode}-native-role-team-sample-${times[index]}.png`
      ),
      readFileSync(path)
    );
  }
  verifyTeamPixels(decode(moved), 4, 500, 3, false, [20, 30]);
  writeFileSync(
    join(native.evidence, `${native.mode}-native-role-team-group-moved.png`),
    readFileSync(moved)
  );
  const frames = decode(range);
  expect(frames.length).toBe(12 * BYTES);
  for (let i = 0; i < 12; i += 1) {
    verifyEncodedTeamPixels(
      frames.subarray(i * BYTES, (i + 1) * BYTES),
      4,
      (i * 1000) / 30
    );
  }

  verifyStaticTeamParity(before, frames.subarray(11 * BYTES, 12 * BYTES));
  for (const [label, path, framesCount] of [
    ["minimum", minimum, "1"],
    ["maximum", maximum, "1"],
    ["changed", changed, "1"],
    ["range", range, "12"],
  ] as const) {
    const probe = JSON.parse(
      execute(native.ffprobe, [
        "-v",
        "error",
        "-count_frames",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=width,height,nb_read_frames",
        "-of",
        "json",
        path,
      ]).toString()
    );
    expect(probe.streams[0]).toMatchObject({
      height: HEIGHT,
      nb_read_frames: framesCount,
      width: WIDTH,
    });
    writeFileSync(
      join(
        native.evidence,
        `${native.mode}-native-role-team-${label}.${label === "range" ? "mp4" : "png"}`
      ),
      readFileSync(path)
    );
  }
  writeFileSync(
    join(native.evidence, `${native.mode}-native-role-team-evidence.json`),
    JSON.stringify(
      {
        encodedTolerance: 8,
        exactAdditionalSampleTimesMs: [40, 80, 120, 280],
        ffmpeg: execute(native.ffmpeg, ["-version"]).toString(),
        ffprobe: execute(native.ffprobe, ["-version"]).toString(),
        frames: 12,
        groupShiftPixels: [20, 30],
        height: HEIGHT,
        incomingAttempt,
        members: [1, 4],
        mode: native.mode,
        oracle:
          "Independent authored interiors and linear-light/8bit-alpha clamped ramps",
        pngTolerance: 3,
        rangeMs: [0, 400],
        recipe: "native-role-team-v1",
        width: WIDTH,
      },
      null,
      2
    )
  );
};
