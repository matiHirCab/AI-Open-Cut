import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import {
  jobSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";
import { verifyHeroPixels } from "./native-hero-workflow";

type Call = <T>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<T>
) => Promise<T>;
export interface MotifNative {
  evidence: string;
  ffmpeg: string;
  ffprobe: string;
  mode: string;
}
const WIDTH = 1920,
  HEIGHT = 1080,
  BYTES = WIDTH * HEIGHT * 3;
// Independently authored interior/gap samples from reference coordinates and
// existing linear-light source-over with8-bit alpha over the black canvas, never captured output.
const gallery: [number, number, number[]][] = [
  [100, 100, [0, 156, 188]],
  [120, 120, [0, 0, 0]],
  [450, 130, [248, 249, 250]],
  [420, 160, [34, 51, 68]],
  [560, 160, [255, 204, 0]],
  [920, 80, [255, 204, 0]],
  [920, 160, [255, 107, 0]],
  [1180, 190, [255, 107, 0]],
  [1290, 180, [255, 204, 0]],
  [1580, 93, [0, 186, 224]],
  [1580, 105, [0, 0, 0]],
  [100, 420, [188, 188, 188]],
  [560, 480, [0, 255, 255]],
  [450, 450, [255, 255, 255]],
  [420, 480, [0, 0, 0]],
  [920, 400, [255, 255, 255]],
  [920, 480, [0, 0, 0]],
  [1180, 510, [255, 0, 0]],
  [1290, 500, [255, 255, 255]],
  [1580, 413, [224, 224, 224]],
  [1910, 1070, [0, 0, 0]],
];
const around: [number, number, number[]][] = [
  [100, 40, [0, 156, 188]],
  [1010, 960, [255, 204, 0]],
  [870, 960, [34, 51, 68]],
  [900, 930, [248, 249, 250]],
  [1060, 40, [255, 204, 0]],
  [1060, 120, [255, 107, 0]],
  [1660, 990, [255, 107, 0]],
  [1770, 980, [255, 204, 0]],
  [280, 893, [0, 186, 224]],
  [280, 905, [0, 0, 0]],
];
// Fixed pixel-centre interiors come from authored radius4 circles and6px
// rectangles. Only encoded thin-feature checks use unrounded ROI means.
const encodedInteriors: Record<string, [number, number, number, number]> = {
  "100,40": [99, 39, 2, 2],
  "100,100": [99, 99, 2, 2],
  "100,420": [99, 419, 2, 2],
  "280,893": [278, 891, 4, 4],
  "1580,93": [1578, 91, 4, 4],
  "1580,413": [1578, 411, 4, 4],
};
const sampleRgb = (
  pixels: Buffer,
  x: number,
  y: number,
  sampling: "points" | "encodedInteriors"
) => {
  const roi =
    sampling === "encodedInteriors" ? encodedInteriors[`${x},${y}`] : undefined;
  if (!roi) {
    return [...pixels.subarray((y * WIDTH + x) * 3, (y * WIDTH + x) * 3 + 3)];
  }
  const [left, top, width, height] = roi;
  const result = [0, 0, 0];
  for (let dy = 0; dy < height; dy += 1) {
    for (let dx = 0; dx < width; dx += 1) {
      for (let channel = 0; channel < 3; channel += 1) {
        result[channel] =
          (result[channel] ?? 0) +
          (pixels[((top + dy) * WIDTH + left + dx) * 3 + channel] ?? -100) /
            (width * height);
      }
    }
  }
  return result;
};
export const verifyMotifPixels = (
  pixels: Buffer,
  hero: boolean,
  tolerance: number,
  onlyMotif?: number,
  sampling: "points" | "encodedInteriors" = "points"
) => {
  if (pixels.length !== BYTES) {
    throw new Error("Wrong motif frame size");
  }
  const owners = [
    0, 0, 1, 1, 1, 2, 2, 3, 3, 4, 4, 0, 1, 1, 1, 2, 2, 3, 3, 4, -1,
  ];
  for (const [index, sample] of (hero ? around : gallery).entries()) {
    const [x, y, paint] = sample;
    const rgb =
      onlyMotif !== undefined && owners[index] !== onlyMotif
        ? [0, 0, 0]
        : paint;
    const actual = sampleRgb(pixels, x, y, sampling);
    for (let c = 0; c < 3; c += 1) {
      if (Math.abs((actual[c] ?? -100) - (rgb[c] ?? 0)) > tolerance) {
        throw new Error(
          `Independent motif pixel mismatch ${x},${y} actual ${actual.join(",")} expected ${rgb.join(",")}`
        );
      }
    }
  }
  if (hero) {
    verifyHeroPixels(pixels, false, tolerance);
  }
};
const verifyUnchangedHeroPixels = (pixels: Buffer, native: MotifNative) => {
  const before = spawnSync(
    native.ffmpeg,
    [
      "-v",
      "error",
      "-i",
      join(native.evidence, `${native.mode}-native-hero-card.png`),
      "-f",
      "rawvideo",
      "-pix_fmt",
      "rgb24",
      "-",
    ],
    { maxBuffer: BYTES + 1024 * 1024 }
  );
  expect(before.status, before.stderr.toString()).toBe(0);
  expect(before.stdout.length).toBe(BYTES);
  for (const left of [120, 1000]) {
    for (let y = 220; y < 820; y += 1) {
      const start = (y * WIDTH + left) * 3;
      const actual = pixels.subarray(start, start + 800 * 3);
      const original = before.stdout.subarray(start, start + 800 * 3);
      if (!actual.equals(original)) {
        const byte = actual.findIndex(
          (value, index) => value !== original[index]
        );
        const x = left + Math.floor(byte / 3);
        throw new Error(
          `Existing hero pixel changed at ${x},${y}: actual ${pixels.subarray((y * WIDTH + x) * 3, (y * WIDTH + x) * 3 + 3).join(",")}, original ${before.stdout.subarray((y * WIDTH + x) * 3, (y * WIDTH + x) * 3 + 3).join(",")}`
        );
      }
    }
  }
};
const verifyHeroPreservation = (
  pixels: Buffer,
  hero: boolean,
  suffix: string,
  native: MotifNative
) => {
  if (hero && suffix === "png") {
    verifyUnchangedHeroPixels(pixels, native);
  }
};
const verifyDecodedFrames = (
  pixels: Buffer,
  hero: boolean,
  tolerance: number,
  sampling: "points" | "encodedInteriors"
) => {
  for (let i = 0; i < pixels.length; i += BYTES) {
    verifyMotifPixels(
      pixels.subarray(i, i + BYTES),
      hero,
      tolerance,
      undefined,
      sampling
    );
  }
};
const verifyNativePixelsWithFailureArtifact = (
  pixels: Buffer,
  hero: boolean,
  suffix: string,
  native: MotifNative,
  artifact: string
) => {
  try {
    verifyDecodedFrames(
      pixels,
      hero,
      suffix === "png" ? 3 : 8,
      suffix === "png" ? "points" : "encodedInteriors"
    );
    verifyHeroPreservation(pixels, hero, suffix, native);
  } catch (error) {
    writeFileSync(
      join(
        native.evidence,
        `${native.mode}-motif-failed-${hero ? "hero" : "gallery"}-${Date.now()}.${suffix}`
      ),
      readFileSync(artifact)
    );
    throw error;
  }
};
const captureMotifArtifact = async (
  call: Call,
  projects: string,
  projectId: string,
  revision: number,
  name: string,
  input: Record<string, unknown>
) => {
  const queued = await call(
    name,
    { expectedRevision: revision, projectId, ...input },
    jobSchema
  );
  let artifact: string | undefined;
  for (let attempt = 0; attempt < 1200; attempt += 1) {
    // biome-ignore lint/performance/noAwaitInLoops: Poll the same immutable job until terminal.
    const job = await call(
      "job_get_status",
      { jobId: queued.jobId },
      jobSchema
    );
    if (["completed", "failed", "cancelled"].includes(job.status)) {
      expect(job.status, JSON.stringify(job.error)).toBe("completed");
      expect(job.revision).toBe(revision);
      if (!job.artifact) {
        throw new Error("Missing motif native artifact");
      }
      artifact = join(projects, projectId, job.artifact.relativePath);
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  if (!artifact) {
    throw new Error("Motif native request exceeded two minutes");
  }
  return artifact;
};

const verifyMotifEdgePlacement = async (
  call: Call,
  projects: string,
  projectId: string,
  native: MotifNative
) => {
  const initial = await call("project_open", { projectId }, projectStateSchema);
  const items = initial.project.tracks
    .flatMap((t) => t.items)
    .filter((i) => i.type === "component_instance");
  const [dot] = items;
  if (!dot?.transform2d) {
    throw new Error("Missing bounded dot placement source");
  }
  const shifted = structuredClone(dot.transform2d);
  shifted.position = { unit: "pixels", x: -260, y: -160 };
  await call(
    "timeline_batch_edit",
    {
      expectedRevision: initial.project.revision,
      operations: [
        ...items.map((i) => ({
          hidden: i.id !== dot.id,
          itemId: i.id,
          operation: "set_item_visibility",
        })),
        { itemId: dot.id, operation: "update_item", transform2d: shifted },
      ],
      projectId,
    },
    writeResultSchema
  );
  const saved = await call("project_open", { projectId }, projectStateSchema);
  const artifact = await captureMotifArtifact(
    call,
    projects,
    projectId,
    saved.project.revision,
    "preview_render_frame",
    { timeMs: 500 }
  );
  const decoded = spawnSync(
    native.ffmpeg,
    ["-v", "error", "-i", artifact, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"],
    { maxBuffer: BYTES + 1024 * 1024 }
  );
  expect(decoded.status, decoded.stderr.toString()).toBe(0);
  expect(decoded.stdout.length).toBe(BYTES);
  for (const [x, y, paint] of [
    [1, 20, [0, 156, 188]],
    [20, 20, [0, 0, 0]],
    [45, 20, [0, 0, 0]],
  ] as const) {
    const actual = decoded.stdout.subarray(
      (y * WIDTH + x) * 3,
      (y * WIDTH + x) * 3 + 3
    );
    expect(
      [...actual].every((v, c) => Math.abs(v - (paint[c] ?? 0)) <= 3),
      `Independent clipped dot ${x},${y}: ${actual.join(",")}`
    ).toBe(true);
  }
  expect(await call("project_open", { projectId }, projectStateSchema)).toEqual(
    saved
  );
  const bytes = readFileSync(artifact);
  writeFileSync(
    join(native.evidence, `${native.mode}-native-motif-clipped-dot.png`),
    bytes
  );
  await call(
    "timeline_batch_edit",
    {
      expectedRevision: saved.project.revision,
      operations: [
        ...items.map((i) => ({
          hidden: i.hidden,
          itemId: i.id,
          operation: "set_item_visibility",
        })),
        {
          itemId: dot.id,
          operation: "update_item",
          transform2d: dot.transform2d,
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  const restored = await call(
    "project_open",
    { projectId },
    projectStateSchema
  );
  expect(restored.project.components).toEqual(initial.project.components);
  expect(restored.project.tracks).toEqual(initial.project.tracks);
  return {
    placement: shifted,
    revision: saved.project.revision,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  };
};

const verifyStandaloneMotifs = async (
  call: Call,
  projects: string,
  projectId: string,
  native: MotifNative
) => {
  const initial = await call("project_open", { projectId }, projectStateSchema);
  const items = initial.project.tracks
    .flatMap((t) => t.items)
    .filter((i) => i.type === "component_instance");
  const records: unknown[] = [];
  for (let index = 0; index < 5; index += 1) {
    // biome-ignore lint/performance/noAwaitInLoops: Each standalone view is a revision-ordered mutation/render pair.
    const before = await call(
      "project_open",
      { projectId },
      projectStateSchema
    );
    const componentId = initial.project.components[index]?.id;
    if (!componentId) {
      throw new Error("Missing standalone motif definition");
    }
    await call(
      "timeline_batch_edit",
      {
        expectedRevision: before.project.revision,
        operations: items.map((i) => ({
          hidden: i.componentId !== componentId,
          itemId: i.id,
          operation: "set_item_visibility",
        })),
        projectId,
      },
      writeResultSchema
    );
    const saved = await call("project_open", { projectId }, projectStateSchema);
    const artifact = await captureMotifArtifact(
      call,
      projects,
      projectId,
      saved.project.revision,
      "preview_render_frame",
      { timeMs: 500 }
    );
    const decoded = spawnSync(
      native.ffmpeg,
      [
        "-v",
        "error",
        "-i",
        artifact,
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "-",
      ],
      { maxBuffer: BYTES + 1024 * 1024 }
    );
    expect(decoded.status, decoded.stderr.toString()).toBe(0);
    verifyMotifPixels(decoded.stdout, false, 3, index);
    const bytes = readFileSync(artifact);
    writeFileSync(
      join(native.evidence, `${native.mode}-native-motif-alone-${index}.png`),
      bytes
    );
    expect(
      await call("project_open", { projectId }, projectStateSchema)
    ).toEqual(saved);
    records.push({
      index,
      revision: saved.project.revision,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    });
  }
  const before = await call("project_open", { projectId }, projectStateSchema);
  await call(
    "timeline_batch_edit",
    {
      expectedRevision: before.project.revision,
      operations: items.map((i) => ({
        hidden: i.hidden,
        itemId: i.id,
        operation: "set_item_visibility",
      })),
      projectId,
    },
    writeResultSchema
  );
  const restored = await call(
    "project_open",
    { projectId },
    projectStateSchema
  );
  expect(restored.project.components).toEqual(initial.project.components);
  expect(restored.project.tracks).toEqual(initial.project.tracks);
  return records;
};

export const verifyNativeMotifMedia = async (
  call: Call,
  projects: string,
  ids: { galleryProjectId: string; heroProjectId: string },
  native: MotifNative
) => {
  mkdirSync(native.evidence, { recursive: true });
  const records: unknown[] = [];
  for (const [projectId, hero] of [
    [ids.galleryProjectId, false],
    [ids.heroProjectId, true],
  ] as const) {
    // biome-ignore lint/performance/noAwaitInLoops: Verify independent projects sequentially.
    const saved = await call("project_open", { projectId }, projectStateSchema);
    for (const [name, input, suffix] of [
      ["preview_render_frame", { timeMs: 500 }, "png"],
      [
        "preview_review_range",
        {
          endMs: 600,
          includeAudio: false,
          resolution: "project",
          startMs: 500,
        },
        "mp4",
      ],
    ] as const) {
      // biome-ignore lint/performance/noAwaitInLoops: Native requests are bounded and serialized to avoid renderer contention.
      const artifact = await captureMotifArtifact(
        call,
        projects,
        projectId,
        saved.project.revision,
        name,
        input
      );
      const execute = (command: string, args: string[]) => {
        const result = spawnSync(command, args, {
          maxBuffer: BYTES * 4 + 1024 * 1024,
        });
        expect(result.status, result.stderr.toString()).toBe(0);
        return result.stdout;
      };
      const probe = JSON.parse(
        execute(native.ffprobe, [
          "-v",
          "error",
          "-count_frames",
          "-show_streams",
          "-show_format",
          "-of",
          "json",
          artifact,
        ]).toString()
      );
      const streams = probe.streams as Record<string, string | number>[];
      expect(streams.filter((s) => s.codec_type === "audio")).toHaveLength(0);
      const video = streams.find((s) => s.codec_type === "video");
      expect(video?.width).toBe(WIDTH);
      expect(video?.height).toBe(HEIGHT);
      expect(Number(video?.nb_read_frames)).toBe(suffix === "png" ? 1 : 3);
      if (suffix === "mp4") {
        expect(video?.r_frame_rate).toBe("30/1");
        expect(Number(probe.format.duration)).toBeCloseTo(0.1, 3);
      }
      const pixels = execute(native.ffmpeg, [
        "-v",
        "error",
        "-i",
        artifact,
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "-",
      ]);
      expect(pixels.length).toBe(BYTES * (suffix === "png" ? 1 : 3));
      verifyNativePixelsWithFailureArtifact(
        pixels,
        hero,
        suffix,
        native,
        artifact
      );
      const bytes = readFileSync(artifact);
      writeFileSync(
        join(
          native.evidence,
          `${native.mode}-native-motifs-${hero ? "around-hero" : "gallery"}.${suffix}`
        ),
        bytes
      );
      records.push({
        hero,
        input,
        probe,
        projectId,
        revision: saved.project.revision,
        sha256: createHash("sha256").update(bytes).digest("hex"),
      });
      expect(
        await call("project_open", { projectId }, projectStateSchema)
      ).toEqual(saved);
    }
  }
  const standalone = await verifyStandaloneMotifs(
    call,
    projects,
    ids.galleryProjectId,
    native
  );
  const edge = await verifyMotifEdgePlacement(
    call,
    projects,
    ids.galleryProjectId,
    native
  );
  writeFileSync(
    join(native.evidence, `${native.mode}-native-motifs-evidence.json`),
    JSON.stringify(
      {
        edge,
        mode: native.mode,
        recipe: "native-motif-pack-v1",
        records,
        source: {
          canonicalSha256: createHash("sha256")
            .update(
              readFileSync(
                new URL(
                  "../../../contracts/native-motif-pack-v1.json",
                  import.meta.url
                )
              )
            )
            .digest("hex"),
          oracleSha256: createHash("sha256")
            .update(
              readFileSync(
                new URL("./native-motif-pack-native.ts", import.meta.url)
              )
            )
            .digest("hex"),
          promptSha256: createHash("sha256")
            .update(
              readFileSync(new URL("../src/server/context.ts", import.meta.url))
            )
            .digest("hex"),
        },
        standalone,
        tools: { ffmpeg: native.ffmpeg, ffprobe: native.ffprobe },
      },
      null,
      2
    )
  );
};
