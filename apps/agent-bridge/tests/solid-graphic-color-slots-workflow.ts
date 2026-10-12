import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFileSync,
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
import catalog from "../../../contracts/solid-graphic-color-slots-v1.json";
import {
  editDraftSchema,
  jobSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;
interface Native {
  evidenceDirectory?: string;
  ffmpeg: string;
  ffprobe: string;
  mode: string;
}
const WIDTH = 320,
  HEIGHT = 180;
// Independently authored sample positions and expected sRGB over black.
// Native alpha is quantized to8-bit once: .5 ->128/255; .25 ->64/255.
// The existing linear-light source-over contract encodes those as188/137.
export const assertSolidGraphicPixels = (
  rgb: Buffer,
  draft = false,
  tolerance = 3
) => {
  expect(rgb.length).toBe(WIDTH * HEIGHT * 3);
  const samples: [number, number, readonly number[]][] = [
    [25, 30, draft ? [0, 188, 0] : [188, 0, 0]],
    [10, 30, [0, 255, 0]],
    [100, 30, [137, 137, 137]],
    [185, 30, [0, 0, 188]],
    [170, 30, [255, 204, 0]],
    [260, 30, [0, 137, 137]],
    [300, 150, [0, 0, 0]],
  ];
  for (const [x, y, expected] of samples) {
    const actual = [
      ...rgb.subarray((y * WIDTH + x) * 3, (y * WIDTH + x) * 3 + 3),
    ];
    expect(
      actual.every(
        (value, index) =>
          Math.abs(value - (expected[index] ?? -100)) <= tolerance
      ),
      `(${x},${y}) actual=${actual} expected=${expected} tolerance=${tolerance}`
    ).toBe(true);
  }
};
const content = (value: unknown) =>
  Object.fromEntries(
    Object.entries(value as Record<string, unknown>).filter(
      ([key]) => !["revision", "updatedAtMs"].includes(key)
    )
  );
const files = (root: string): Record<string, string> => {
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

export async function verifySolidGraphicSlots(
  client: Client,
  call: Call,
  projectsDirectory: string,
  native?: Native
) {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.capabilities).toContain(catalog.capability);
  const { projectId } = await call(
    "project_create",
    {
      fps: 10,
      height: HEIGHT,
      name: "Solid graphic color slots",
      width: WIDTH,
    },
    writeResultSchema
  );
  const directory = join(projectsDirectory, projectId);
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const initial = await read();
  const trackId = initial.project.tracks.find(
    (t) => t.trackType === "overlay"
  )?.id;
  if (!trackId) {
    throw new Error("Overlay track missing");
  }
  const { slots, ...definition } = catalog.definition;
  const instance = (alias: string, x: number, slotValues: unknown) => ({
    componentId: "@card",
    durationMs: 1000,
    operation: "add_component_instance",
    resultAlias: alias,
    slotValues,
    startMs: 0,
    timeScale: 1,
    trackId,
    transform: { opacity: 1, positionX: x, positionY: 0, scale: 1 },
    trimStartMs: 0,
  });
  const operations = [
    { ...definition, resultAlias: "card", slots: [] },
    { componentId: "@card", operation: "component_define_slots", slots },
    instance("left", 0, {}),
    instance("right", 160, catalog.overrides),
    {
      animationChannels: [
        {
          keyframes: [
            { curve: "hold", timeMs: 0, value: { type: "scalar", value: 1 } },
          ],
          property: "transform.opacity",
        },
      ],
      itemId: "@left",
      operation: "set_animation_channels",
    },
  ];
  const reject = async (
    name: string,
    input: Record<string, unknown>,
    code: string,
    retryable = false
  ) => {
    const before = await read(),
      bytes = files(directory);
    const result = await client.callTool({
      arguments: { projectId, ...input },
      name,
    });
    expect(result.isError).toBe(true);
    expect(
      result.structuredContent,
      JSON.stringify(result.content)
    ).toMatchObject({ error: { code, retryable } });
    expect(await read()).toEqual(before);
    expect(files(directory)).toEqual(bytes);
  };
  await reject(
    "timeline_batch_edit",
    {
      expectedRevision: 0,
      operations: [
        ...operations,
        { componentId: "missing", operation: "component_delete" },
      ],
    },
    "ITEM_NOT_FOUND"
  );
  const result = await call(
    "timeline_batch_edit",
    { expectedRevision: 0, operations, projectId },
    writeResultSchema
  );
  expect(result.revision).toBe(1);
  const card = result.aliases?.card,
    left = result.aliases?.left,
    right = result.aliases?.right;
  if (!(card && left && right)) {
    throw new Error("Graphic batch aliases missing");
  }
  const seeded = await read();
  expect(seeded.project.schemaVersion).toBe(45);
  const update = (slotValues: unknown) => ({
    componentId: card,
    durationMs: 1000,
    itemId: left,
    operation: "component_instance_update",
    slotValues,
    startMs: 0,
    timeScale: 1,
    trimStartMs: 0,
  });
  const green = { fill: { type: "color", value: "#00ff00" } };
  // MCP inputs exclude the operation tag accepted by the headless batch union.
  const standalone = (operation: Record<string, unknown>) =>
    Object.fromEntries(
      Object.entries(operation).filter(([key]) => key !== "operation")
    );
  await reject(
    "component_instance_update",
    { ...standalone(update(green)), expectedRevision: 0 },
    "REVISION_CONFLICT",
    true
  );
  for (const value of catalog.invalidValues) {
    if (value?.type === "number") {
      // biome-ignore lint/performance/noAwaitInLoops: Serial failures preserve the same revision and byte snapshot.
      await reject(
        "component_instance_update",
        { ...standalone(update({ fill: value })), expectedRevision: 1 },
        "INVALID_ARGUMENT"
      );
      continue;
    }
    const before = await read(),
      bytes = files(directory);
    const malformed = await client.callTool({
      arguments: {
        expectedRevision: 1,
        projectId,
        ...standalone(update({ fill: value })),
      },
      name: "component_instance_update",
    });
    expect(malformed.isError).toBe(true);
    expect(JSON.stringify(malformed.content)).toContain(
      "Input validation error: Invalid arguments for tool component_instance_update"
    );
    expect(await read()).toEqual(before);
    expect(files(directory)).toEqual(bytes);
  }
  await call(
    "timeline_batch_edit",
    {
      expectedRevision: 1,
      operations: [{ locked: true, operation: "update_track", trackId }],
      projectId,
    },
    writeResultSchema
  );
  await reject(
    "component_instance_update",
    { ...standalone(update(green)), expectedRevision: 2 },
    "TRACK_LOCKED"
  );
  await call(
    "timeline_batch_edit",
    {
      expectedRevision: 2,
      operations: [{ locked: false, operation: "update_track", trackId }],
      projectId,
    },
    writeResultSchema
  );
  const saved = await read();
  const { revision } = saved.project;
  const draft = await call(
    "draft_create",
    { expectedRevision: revision, operations: [update(green)], projectId },
    editDraftSchema
  );
  const updatedDraft = await call(
    "draft_update",
    {
      draftId: draft.id,
      expectedRevision: revision,
      label: "Independent green candidate",
      operations: [update(green)],
      projectId,
    },
    editDraftSchema
  );
  expect(updatedDraft.operations).toEqual(draft.operations);
  expect(
    await call("draft_get", { draftId: draft.id, projectId }, editDraftSchema)
  ).toEqual(updatedDraft);
  expect(await read()).toEqual(saved);

  if (native) {
    const durableFiles = ["project.json", "history.json"] as const;
    const durableBytes = durableFiles.map((name) =>
      readFileSync(join(directory, name))
    );
    const execute = (command: string, args: string[]) => {
      const r = spawnSync(command, args, { maxBuffer: 16 * 1024 * 1024 });
      if (r.status !== 0) {
        throw new Error(`${command} failed: ${r.stderr?.toString()}`);
      }
      return r.stdout;
    };
    const render = async (name: string, input: Record<string, unknown>) => {
      let job = await call(name, { projectId, ...input }, jobSchema);
      for (
        let count = 0;
        !["completed", "failed", "cancelled"].includes(job.status);
        count += 1
      ) {
        if (count >= 1200) {
          throw new Error("Graphic native job exceeded two minute allowance");
        }
        // biome-ignore lint/performance/noAwaitInLoops: Poll one real job serially under the existing deadline.
        await new Promise((done) => setTimeout(done, 100));
        job = await call("job_get_status", { jobId: job.jobId }, jobSchema);
      }
      expect(job.status, JSON.stringify(job.error)).toBe("completed");
      expect(job.revision).toBe(revision);
      if (!job.artifact) {
        throw new Error("Graphic native artifact missing");
      }
      return join(directory, job.artifact.relativePath);
    };
    const frame = await render("preview_render_frame", {
      expectedRevision: revision,
      timeMs: 300,
    });
    const range = await render("preview_render_range", {
      endMs: 200,
      expectedRevision: revision,
      fps: 10,
      includeAudio: false,
      resolution: { height: HEIGHT, width: WIDTH },
      startMs: 0,
    });
    const preview = await render("draft_preview_frame", {
      draftId: draft.id,
      timeMs: 300,
    });
    const decode = (path: string) =>
      execute(native.ffmpeg, [
        "-v",
        "error",
        "-i",
        path,
        "-an",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "pipe:1",
      ]);
    const rgb = decode(frame),
      draftRgb = decode(preview),
      rangeRgb = decode(range);
    assertSolidGraphicPixels(rgb);
    assertSolidGraphicPixels(draftRgb, true);
    expect(rangeRgb.length).toBe(WIDTH * HEIGHT * 3 * 2);
    for (let index = 0; index < 2; index += 1) {
      assertSolidGraphicPixels(
        rangeRgb.subarray(index * rgb.length, (index + 1) * rgb.length),
        false,
        8
      );
    }
    // Native draft edits affect left fill only; the entire other occurrence is equal.
    for (let y = 0; y < HEIGHT; y += 1) {
      const start = (y * WIDTH + 160) * 3,
        end = (y * WIDTH + WIDTH) * 3;
      expect(draftRgb.subarray(start, end)).toEqual(rgb.subarray(start, end));
    }
    for (const png of [frame, preview]) {
      const dimensions = JSON.parse(
        execute(native.ffprobe, [
          "-v",
          "error",
          "-select_streams",
          "v:0",
          "-show_entries",
          "stream=width,height",
          "-of",
          "json",
          png,
        ]).toString()
      ) as { streams: { width: number; height: number }[] };
      expect(dimensions.streams[0]).toMatchObject({
        height: HEIGHT,
        width: WIDTH,
      });
    }
    const probe = JSON.parse(
      execute(native.ffprobe, [
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-count_frames",
        "-show_entries",
        "stream=width,height,nb_read_frames",
        "-of",
        "json",
        range,
      ]).toString()
    ) as {
      streams: { width: number; height: number; nb_read_frames: string }[];
    };
    expect(probe.streams[0]).toMatchObject({
      height: HEIGHT,
      nb_read_frames: "2",
      width: WIDTH,
    });
    expect(await read()).toEqual(saved);
    for (const [index, name] of durableFiles.entries()) {
      expect(readFileSync(join(directory, name)), name).toEqual(
        durableBytes[index]
      );
    }
    const nativeRecord = {
      artifacts: [frame, range, preview].map((path) => ({
        sha256: createHash("sha256").update(readFileSync(path)).digest("hex"),
      })),
      dimensions: [WIDTH, HEIGHT],
      ffmpegVersion: execute(native.ffmpeg, ["-version"])
        .toString()
        .split("\n")[0],
      helperSha256: createHash("sha256")
        .update(readFileSync(new URL(import.meta.url)))
        .digest("hex"),
      mode: native.mode,
      platform: process.platform,
      projectRevision: revision,
      rangeFrames: 2,
      recipeSha256: createHash("sha256")
        .update(JSON.stringify(catalog))
        .digest("hex"),
      sourceRevision: execute("git", ["rev-parse", "HEAD"]).toString().trim(),
    };
    if (native.evidenceDirectory) {
      mkdirSync(native.evidenceDirectory, { recursive: true });
      for (const [path, suffix] of [
        [frame, "frame.png"],
        [range, "range.mp4"],
        [preview, "draft.png"],
      ] as const) {
        copyFileSync(
          path,
          join(
            native.evidenceDirectory,
            `${native.mode}-solid-graphic-${suffix}`
          )
        );
      }
      writeFileSync(
        join(native.evidenceDirectory, `${native.mode}-solid-graphic.json`),
        `${JSON.stringify(nativeRecord, null, 2)}\n`
      );
    }
  }

  await call(
    "draft_commit",
    { draftId: draft.id, expectedRevision: revision, projectId },
    writeResultSchema
  );
  const changed = await read();
  expect(changed.project.components).toEqual(saved.project.components);
  const items = (state: typeof saved) =>
    state.project.tracks.flatMap((t) => t.items);
  expect(items(changed).find((i) => i.id === right)).toEqual(
    items(saved).find((i) => i.id === right)
  );
  const oldLeft = items(saved).find((i) => i.id === left),
    newLeft = items(changed).find((i) => i.id === left);
  expect({ ...newLeft, slotValues: undefined }).toEqual({
    ...oldLeft,
    slotValues: undefined,
  });
  expect(newLeft).toMatchObject({ slotValues: green });
  await call(
    "project_undo",
    { expectedRevision: revision + 1, projectId },
    writeResultSchema
  );
  expect(content((await read()).project)).toEqual(content(saved.project));
  await call(
    "project_redo",
    { expectedRevision: revision + 2, projectId },
    writeResultSchema
  );
  expect(content((await read()).project)).toEqual(content(changed.project));
  expect(content((await read()).project)).toEqual(content(changed.project));
  const discard = await call(
    "draft_create",
    { expectedRevision: revision + 3, operations: [update({})], projectId },
    editDraftSchema
  );
  const bytes = files(directory);
  await call(
    "draft_discard",
    { draftId: discard.id, projectId },
    editDraftSchema
  );
  const afterDiscard = files(directory);
  const remaining = Object.fromEntries(
    Object.entries(bytes).filter(([path]) => !path.includes(discard.id))
  );
  expect(afterDiscard).toEqual(remaining);
  expect(content((await read()).project)).toEqual(content(changed.project));
  expect(seeded.project.components).toEqual(changed.project.components);
}
