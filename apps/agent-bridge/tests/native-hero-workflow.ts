import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  mkdirSync,
  readdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import type RECIPE from "../src/recipes/native-hero-card-v1.json";
import {
  jobSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <T>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<T>
) => Promise<T>;
interface Native {
  evidence: string;
  ffmpeg: string;
  ffprobe: string;
  mode: string;
}
const required = <T>(value: T | undefined): T => {
  if (value === undefined) {
    throw new Error("Missing required hero evidence");
  }
  return value;
};
const WIDTH = 1920,
  HEIGHT = 1080,
  FRAME_BYTES = WIDTH * HEIGHT * 3;
const samples: [number, number, [number, number, number]][] = [
  [50, 50, [0, 0, 0]],
  [175, 275, [238, 238, 238]],
  [230, 300, [22, 32, 51]],
  [200, 775, [48, 73, 104]],
  [250, 662, [255, 204, 51]],
  [440, 400, [255, 255, 255]],
  [615, 400, [0, 0, 255]],
  [1495, 400, [255, 0, 0]],
  [1130, 662, [255, 204, 51]],
];
const heroGlyphPixels = (pixels: Buffer) => {
  let white = 0;
  for (let y = 717; y < 785; y += 1) {
    for (let x = 184; x < 854; x += 1) {
      const i = (y * WIDTH + x) * 3;
      if (
        (pixels[i] ?? 0) > 240 &&
        (pixels[i + 1] ?? 0) > 240 &&
        (pixels[i + 2] ?? 0) > 240
      ) {
        white += 1;
      }
    }
  }
  return white;
};
export const verifyHeroPixels = (
  pixels: Buffer,
  hidden: boolean,
  tolerance: number
) => {
  if (pixels.length !== FRAME_BYTES) {
    throw new Error("Wrong hero frame size");
  }
  for (const [x, y, rgb] of samples) {
    const expected = hidden && x < 920 && x > 100 ? [0, 0, 0] : rgb;
    for (let c = 0; c < 3; c += 1) {
      if (
        Math.abs(
          (pixels[(y * WIDTH + x) * 3 + c] ?? -100) - (expected[c] ?? 0)
        ) > tolerance
      ) {
        throw new Error(`Independent hero pixel mismatch at ${x},${y}`);
      }
    }
  }
  if (!hidden) {
    const white = heroGlyphPixels(pixels);
    if (white < 100) {
      throw new Error("Native hero title glyphs absent");
    }
  }
};

const snapshot = (root: string) => {
  const result: Record<string, string> = {};
  const visit = (directory: string) => {
    for (const name of readdirSync(directory).sort()) {
      if (name === ".lock") {
        continue;
      }
      const path = join(directory, name);
      if (statSync(path).isDirectory()) {
        visit(path);
      } else {
        result[path] = createHash("sha256")
          .update(readFileSync(path))
          .digest("hex");
      }
    }
  };
  visit(root);
  return result;
};
const content = (project: object) =>
  Object.fromEntries(
    Object.entries(project).filter(
      ([key]) => !["revision", "updatedAtMs"].includes(key)
    )
  );

const originalHeroBmp = (right: readonly number[]) => {
  const bmp = Buffer.alloc(54 + 64 * 64 * 3);
  bmp.write("BM");
  bmp.writeUInt32LE(bmp.length, 2);
  bmp.writeUInt32LE(54, 10);
  bmp.writeUInt32LE(40, 14);
  bmp.writeUInt32LE(64, 18);
  bmp.writeUInt32LE(64, 22);
  bmp.writeUInt16LE(1, 26);
  bmp.writeUInt16LE(24, 28);
  for (let y = 0; y < 64; y += 1) {
    for (let x = 0; x < 64; x += 1) {
      const rgb = x < 32 ? [255, 255, 255] : right;
      const i = 54 + (y * 64 + x) * 3;
      bmp[i] = required(rgb[2]);
      bmp[i + 1] = required(rgb[1]);
      bmp[i + 2] = required(rgb[0]);
    }
  }
  return bmp;
};

const captureHeroArtifact = async (
  call: Call,
  projectId: string,
  revision: number,
  directory: string,
  name: string,
  input: Record<string, unknown>
) => {
  const queued = await call(
    name,
    { expectedRevision: revision, projectId, ...input },
    jobSchema
  );
  for (let attempt = 0; attempt < 1200; attempt += 1) {
    // biome-ignore lint/performance/noAwaitInLoops: Poll one immutable renderer request until terminal.
    const job = await call(
      "job_get_status",
      { jobId: queued.jobId },
      jobSchema
    );
    if (["completed", "failed", "cancelled"].includes(job.status)) {
      expect(job.status, JSON.stringify(job.error)).toBe("completed");
      expect(job.revision).toBe(revision);
      return join(directory, required(job.artifact).relativePath);
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("Hero native request exceeded two minutes");
};

const verifyCustomizationPixels = (before: Buffer, after: Buffer) => {
  expect(before.length).toBe(FRAME_BYTES);
  expect(after.length).toBe(FRAME_BYTES);
  let changedTitlePixels = 0;
  for (let y = 717; y < 785; y += 1) {
    for (let x = 184; x < 854; x += 1) {
      const p = (y * WIDTH + x) * 3;
      if (!before.subarray(p, p + 3).equals(after.subarray(p, p + 3))) {
        changedTitlePixels += 1;
      }
    }
  }
  expect(changedTitlePixels).toBeGreaterThan(100);
  for (let y = 220; y < 820; y += 1) {
    const start = (y * WIDTH + 1000) * 3;
    expect(before.subarray(start, start + 800 * 3)).toEqual(
      after.subarray(start, start + 800 * 3)
    );
  }
};

const verifyNativeHeroArtifacts = (
  native: Native,
  artifacts: string[],
  recipe: typeof RECIPE,
  revision: number,
  beforeFrame: string
) => {
  mkdirSync(native.evidence, { recursive: true });
  for (const [index, path] of artifacts.entries()) {
    writeFileSync(
      join(
        native.evidence,
        `${native.mode}-native-hero-card.${index === 0 ? "png" : "mp4"}`
      ),
      readFileSync(path)
    );
  }
  const execute = (command: string, args: string[]) => {
    const result = spawnSync(command, args, {
      maxBuffer: FRAME_BYTES * 6 + 1024 * 1024,
    });
    expect(
      result.status,
      result.error?.message ?? result.stderr.toString()
    ).toBe(0);
    return result.stdout;
  };
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
  const before = decode(beforeFrame),
    after = decode(required(artifacts[0]));
  verifyCustomizationPixels(before, after);
  writeFileSync(
    join(native.evidence, `${native.mode}-native-hero-card-before.png`),
    readFileSync(beforeFrame)
  );
  for (const [i, path] of artifacts.entries()) {
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
      nb_read_frames: i === 0 ? "1" : "6",
      width: WIDTH,
    });
    const pixels = execute(native.ffmpeg, [
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
    if (i === 0) {
      verifyHeroPixels(pixels, false, 2);
    } else {
      expect(pixels.length).toBe(FRAME_BYTES * 6);
      verifyHeroPixels(pixels.subarray(0, FRAME_BYTES), true, 8);
      verifyHeroPixels(pixels.subarray(FRAME_BYTES * 5), false, 8);
      const final = pixels.subarray(FRAME_BYTES * 5);
      const frame = execute(native.ffmpeg, [
        "-v",
        "error",
        "-i",
        required(artifacts[0]),
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "pipe:1",
      ]);
      for (const [x, y] of samples) {
        for (let c = 0; c < 3; c += 1) {
          const p = (y * WIDTH + x) * 3 + c;
          expect(
            Math.abs(required(final[p]) - required(frame[p]))
          ).toBeLessThanOrEqual(8);
        }
      }
    }
  }
  mkdirSync(native.evidence, { recursive: true });
  writeFileSync(
    join(native.evidence, `${native.mode}-native-hero-card.json`),
    JSON.stringify(
      {
        artifactSha256: artifacts.map((p) =>
          createHash("sha256").update(readFileSync(p)).digest("hex")
        ),
        ffmpeg: execute(native.ffmpeg, ["-version"]).toString(),
        frameCount: 6,
        immutableRevision: revision,
        mode: native.mode,
        platform: process.platform,
        recipeId: recipe.recipeId,
        recipeSha256: createHash("sha256")
          .update(JSON.stringify(recipe))
          .digest("hex"),
        sourceRevision: execute("git", [
          "-C",
          join(import.meta.dirname, "../../.."),
          "rev-parse",
          "HEAD",
        ])
          .toString()
          .trim(),
      },
      null,
      2
    )
  );
};

export const verifyNativeHeroCardWorkflow = async (
  client: Client,
  call: Call,
  media: string,
  projects: string,
  native?: Native
) => {
  const created = await call(
    "project_create",
    { fps: 30, height: HEIGHT, name: "Native hero card v1", width: WIDTH },
    writeResultSchema
  );
  const { projectId } = created;
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const initial = await read();
  const listed = (await client.listPrompts()).prompts.find(
    (p) => p.name === "create_motion_graphics"
  );
  expect(listed?.description).toContain("native-hero-card-v1");
  const prompt = await client.getPrompt({
    arguments: { projectId, request: "Create native-hero-card-v1" },
    name: "create_motion_graphics",
  });
  const message = prompt.messages[0]?.content;
  if (message?.type !== "text") {
    throw new Error("Missing hero recipe guidance");
  }
  const recipe: typeof RECIPE = JSON.parse(
    message.text.split("native-hero-card-v1 recipe JSON:\n")[1] ?? "null"
  );
  expect(recipe.version).toBe(1);
  expect(await read()).toEqual(initial);
  const directory = join(projects, projectId);
  const assets: string[] = [];
  for (const [name, right] of [
    ["hero-red.bmp", [255, 0, 0]],
    ["hero-blue.bmp", [0, 0, 255]],
  ] as const) {
    const bmp = originalHeroBmp(right);
    const path = join(media, name);
    writeFileSync(path, bmp);
    // biome-ignore lint/performance/noAwaitInLoops: Imports share one optimistic project revision.
    const imported = await call(
      "asset_import",
      {
        expectedRevision: (await read()).project.revision,
        mediaType: "image",
        path,
        projectId,
      },
      writeResultSchema
    );
    assets.push(required(imported.changedIds[0]));
  }
  const before = await read();
  const overlay = before.project.tracks.find(
    (t) => t.trackType === "overlay"
  )?.id;
  if (!(overlay && assets[0] && assets[1])) {
    throw new Error("Missing hero inputs");
  }
  const replacements: Record<string, string> = {
    $artwork: assets[0],
    $overlay: overlay,
  };
  const operations = JSON.parse(
    JSON.stringify(recipe.operations),
    (_key, value) =>
      typeof value === "string" && Object.hasOwn(replacements, value)
        ? replacements[value]
        : value
  );
  const retained = snapshot(directory);
  for (const [candidate, code] of [
    [
      [...operations, { itemId: "missing", operation: "delete_item" }],
      "ITEM_NOT_FOUND",
    ],
    [
      JSON.parse(JSON.stringify(operations), (_key, value) =>
        value === assets[0] ? "missing-safe-asset" : value
      ),
      "ITEM_NOT_FOUND",
    ],
  ] as const) {
    // biome-ignore lint/performance/noAwaitInLoops: Every failed candidate is compared against one unchanged generation.
    const failed = await client.callTool({
      arguments: {
        expectedRevision: before.project.revision,
        operations: candidate,
        projectId,
      },
      name: "timeline_batch_edit",
    });
    expect(failed.isError).toBe(true);
    expect(
      failed.structuredContent,
      JSON.stringify(failed.structuredContent)
    ).toMatchObject({
      error: { code, retryable: false },
    });
    expect(snapshot(directory)).toEqual(retained);
    expect(await read()).toEqual(before);
  }
  const batch = await call(
    "timeline_batch_edit",
    { expectedRevision: before.project.revision, operations, projectId },
    writeResultSchema
  );
  expect(batch.revision).toBe(before.project.revision + 1);
  const original = await read();
  expect(original.project.components).toHaveLength(1);
  expect(
    original.project.components[0]?.tracks[0]?.items.map((i) => i.id)
  ).toEqual([
    "background",
    "device_frame",
    "screen_panel",
    "label_panel",
    "accent",
    "artwork",
    "title",
  ]);
  const instances = original.project.tracks
    .flatMap((t) => t.items)
    .filter((i) => i.type === "component_instance");
  expect(instances).toHaveLength(2);
  const left = required(instances.find((i) => i.id === batch.aliases.left));
  const right = required(instances.find((i) => i.id === batch.aliases.right));
  if (left.type !== "component_instance") {
    throw new Error("Missing native hero instance");
  }
  expect(left.animationChannels?.[0]?.property).toBe("transform.opacity");
  const update = {
    componentId: left.componentId,
    durationMs: left.durationMs,
    itemId: left.id,
    projectId,
    slotValues: {
      artwork: {
        type: "asset",
        value: { id: assets[1], kind: "asset", scope: "project" },
      },
      title: { type: "text", value: "EDITED HERO" },
    },
    startMs: left.startMs,
    timeScale: left.timeScale,
    trimStartMs: left.trimStartMs,
  };
  const beforeFrame = native
    ? await captureHeroArtifact(
        call,
        projectId,
        original.project.revision,
        directory,
        "preview_render_frame",
        { timeMs: 500 }
      )
    : undefined;
  expect(await read()).toEqual(original);
  const unchanged = snapshot(directory);
  const stale = await client.callTool({
    arguments: { ...update, expectedRevision: before.project.revision },
    name: "component_instance_update",
  });
  expect(stale.structuredContent).toMatchObject({
    error: { code: "REVISION_CONFLICT", retryable: true },
  });
  expect(snapshot(directory)).toEqual(unchanged);
  const changed = await call(
    "component_instance_update",
    { ...update, expectedRevision: batch.revision },
    writeResultSchema
  );
  const customized = await read();
  expect(customized.project.components).toEqual(original.project.components);
  const changedItems = customized.project.tracks.flatMap((t) => t.items);
  expect(changedItems.find((i) => i.id === right.id)).toEqual(right);
  expect(changedItems.find((i) => i.id === left.id)).toEqual({
    ...left,
    slotValues: update.slotValues,
  });
  await call(
    "project_undo",
    { expectedRevision: changed.revision, projectId },
    writeResultSchema
  );
  expect(content((await read()).project)).toEqual(content(original.project));
  const redo = await call(
    "project_redo",
    { expectedRevision: changed.revision + 1, projectId },
    writeResultSchema
  );
  expect(content((await read()).project)).toEqual(content(customized.project));
  await call(
    "project_undo",
    { expectedRevision: redo.revision, projectId },
    writeResultSchema
  );
  await call(
    "project_undo",
    { expectedRevision: redo.revision + 1, projectId },
    writeResultSchema
  );
  expect(content((await read()).project)).toEqual(content(before.project));
  await call(
    "project_redo",
    { expectedRevision: redo.revision + 2, projectId },
    writeResultSchema
  );
  await call(
    "project_redo",
    { expectedRevision: redo.revision + 3, projectId },
    writeResultSchema
  );
  const saved = await read();
  expect(content(saved.project)).toEqual(content(customized.project));
  if (native) {
    const artifacts: string[] = [];
    for (const [name, input] of [
      ["preview_render_frame", { timeMs: 500 }],
      [
        "preview_review_range",
        { endMs: 200, includeAudio: false, resolution: "project", startMs: 0 },
      ],
    ] as const) {
      artifacts.push(
        // biome-ignore lint/performance/noAwaitInLoops: Capture bounded intents sequentially to avoid renderer contention.
        await captureHeroArtifact(
          call,
          projectId,
          saved.project.revision,
          directory,
          name,
          input
        )
      );
      expect(await read()).toEqual(saved);
    }
    verifyNativeHeroArtifacts(
      native,
      artifacts,
      recipe,
      saved.project.revision,
      required(beforeFrame)
    );
  }
  expect(await read()).toEqual(saved);
};
