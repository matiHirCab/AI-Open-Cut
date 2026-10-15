import { spawnSync } from "node:child_process";
import { cpSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { z } from "zod/v4";
import {
  componentFieldsSchema,
  jobSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";
import {
  illustratedTargetOperations,
  TARGET_IDS,
  TARGET_TIMING,
} from "./fixtures/illustrated-targets";
import {
  FRAME_BYTES,
  syntheticPcm,
  syntheticWav,
  verifyIllustratedPixels,
  verifyOriginalAudio,
} from "./illustrated-media-oracle";
import {
  observeIllustratedTarget,
  validateIllustratedTarget,
} from "./illustrated-target-oracle";
import {
  type TeamCall,
  type TeamNative,
  teamProjectInventory,
} from "./native-role-team-native";

type State = z.infer<typeof projectStateSchema>;
const SSIM = /All:([0-9.]+)/u;
const needed = <T>(value: T | undefined): T => {
  if (value === undefined) {
    throw new Error("Missing illustrated workflow evidence");
  }
  return value;
};
const originalArtwork = (blue: boolean) => {
  const b = Buffer.alloc(54 + 64 * 64 * 3);
  b.write("BM");
  b.writeUInt32LE(b.length, 2);
  b.writeUInt32LE(54, 10);
  b.writeUInt32LE(40, 14);
  b.writeUInt32LE(64, 18);
  b.writeUInt32LE(64, 22);
  b.writeUInt16LE(1, 26);
  b.writeUInt16LE(24, 28);
  for (let y = 0; y < 64; y += 1) {
    for (let x = 0; x < 64; x += 1) {
      const right = blue ? [0, 0, 255] : [255, 0, 0];
      const color = x < 32 ? [255, 255, 255] : right;
      const p = 54 + (y * 64 + x) * 3;
      b[p] = color[2] ?? 0;
      b[p + 1] = color[1] ?? 0;
      b[p + 2] = color[0] ?? 0;
    }
  }
  return b;
};
const assertNativeRgb = (actual: number[], expected: number[]) => {
  for (const [channel, value] of expected.entries()) {
    expect(Math.abs(needed(actual[channel]) - value)).toBeLessThanOrEqual(3);
  }
};
const content = (state: State) => {
  const {
    revision: _revision,
    updatedAtMs: _updated,
    ...project
  } = state.project;
  return project;
};
const terminal = async (call: TeamCall, jobId: string) => {
  for (let attempt = 0; attempt < 1200; attempt += 1) {
    // biome-ignore lint/performance/noAwaitInLoops: Poll exactly one bounded immutable request.
    const job = await call("job_get_status", { jobId }, jobSchema);
    if (["completed", "failed", "cancelled"].includes(job.status)) {
      return job;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("Illustrated native request exceeded two minutes");
};
type Execute = (
  command: string,
  args: string[]
) => { stdout: Buffer; stderr: Buffer };
const verifyTargetMedia = (
  id: string,
  range: string,
  exported: string,
  native: TeamNative,
  execute: Execute,
  decode: (path: string, video?: boolean) => Buffer,
  sourcePcm: Float32Array
) => {
  const probes = [range, exported].map(
    (path) =>
      JSON.parse(
        execute(native.ffprobe, [
          "-v",
          "error",
          "-show_streams",
          "-show_format",
          "-of",
          "json",
          path,
        ]).stdout.toString()
      ) as {
        streams: {
          codec_type: string;
          width?: number;
          height?: number;
          nb_frames?: string;
        }[];
        format: { duration: string };
      }
  );
  for (const probe of probes) {
    expect(
      probe.streams.find((stream) => stream.codec_type === "video")
    ).toMatchObject({ height: 1080, nb_frames: "90", width: 1920 });
    expect(probe.streams.some((stream) => stream.codec_type === "audio")).toBe(
      true
    );
    expect(Math.abs(Number(probe.format.duration) - 3)).toBeLessThanOrEqual(
      1 / 30
    );
  }
  for (const path of [range, exported]) {
    const decodedFrames = execute(native.ffmpeg, [
      "-v",
      "error",
      "-i",
      path,
      "-vf",
      "select=eq(n\\,0)+eq(n\\,45)+eq(n\\,89)",
      "-fps_mode",
      "passthrough",
      "-f",
      "rawvideo",
      "-pix_fmt",
      "rgb24",
      "-",
    ]).stdout;
    expect(decodedFrames.length).toBe(FRAME_BYTES * 3);
    for (const [index, timeMs] of TARGET_TIMING.samples.entries()) {
      verifyIllustratedPixels(
        id,
        decodedFrames.subarray(index * FRAME_BYTES, (index + 1) * FRAME_BYTES),
        timeMs,
        8
      );
    }
  }
  const ssim = Number(
    SSIM.exec(
      execute(native.ffmpeg, [
        "-i",
        range,
        "-i",
        exported,
        "-lavfi",
        "[0:v][1:v]ssim",
        "-f",
        "null",
        "-",
      ]).stderr.toString()
    )?.[1]
  );
  expect(ssim).toBeGreaterThanOrEqual(0.99);
  const left = decode(range, false),
    right = decode(exported, false);
  expect(left.length).toBe(right.length);
  let difference = 0,
    energy = 0;
  for (let p = 0; p < left.length; p += 4) {
    const sample = left.readFloatLE(p),
      delta = sample - right.readFloatLE(p);
    difference += delta * delta;
    energy += sample * sample;
  }
  const pcmRms = Math.sqrt(difference / (left.length / 4));
  expect(pcmRms).toBeLessThanOrEqual(0.0001);
  expect(Math.sqrt(energy / (left.length / 4))).toBeGreaterThan(0.005);
  const originalAudio = [left, right].map((pcm) =>
    verifyOriginalAudio(pcm, sourcePcm)
  );
  const noAudio = Buffer.alloc(left.length);
  expect(() => verifyOriginalAudio(noAudio, sourcePcm)).toThrow("correlation");
  expect(() => verifyOriginalAudio(Buffer.alloc(0), sourcePcm)).toThrow(
    "omitted"
  );
  const shifted = Buffer.alloc(left.length);
  left.copy(shifted, 1920 * 8, 0, left.length - 1920 * 8);
  expect(() => verifyOriginalAudio(shifted, sourcePcm)).toThrow("clock");
  return { originalAudio, pcmRms, probes, ssim };
};
const verifyNativeEdits = async (context: {
  id: string;
  projectId: string;
  state: State;
  aliases: Record<string, string>;
  operations: Record<string, unknown>[];
  imported: string[];
  frames: string[];
  client: Client;
  call: TeamCall;
  read: () => Promise<State>;
  authoritative: () => Record<string, string>;
  decode: (path: string) => Buffer;
  capture: (
    name: string,
    input: Record<string, unknown>,
    label: string
  ) => Promise<string>;
  setState: (state: State) => void;
}) => {
  const {
    id,
    projectId,
    aliases,
    operations,
    imported,
    frames,
    client,
    call,
    read,
    authoritative,
    decode,
    capture,
    setState,
  } = context;
  let { state } = context;
  const sourceId = needed(
    aliases[id === "role_badge_team_intro" ? "badge" : "source"]
  );
  const primaryId = needed(
    aliases[id === "role_badge_team_intro" ? "member_1" : "primary"]
  );
  const authored = needed(
    operations.find(
      (op) =>
        op.operation === "component_create" &&
        op.resultAlias === (id === "role_badge_team_intro" ? "badge" : "source")
    )
  );
  const { operation: _operation, resultAlias: _alias, ...fields } = authored;
  const source = componentFieldsSchema.parse(fields);
  const edited = structuredClone(source);
  edited.slots = structuredClone(
    needed(
      state.project.components.find((component) => component.id === sourceId)
    ).slots
  );
  for (const slot of edited.slots) {
    if (slot.id === "screen_panel_color") {
      slot.defaultValue = { type: "color", value: "#008000" };
    }
  }
  const sourceLayer = edited.tracks
    .flatMap((track) => track.items)
    .find(
      (item) =>
        item.id ===
        (id === "sticker_motif_intro"
          ? "authored_sticker_contour"
          : "screen_panel")
    );
  if (!(sourceLayer?.type === "shape" && sourceLayer.fill?.type === "solid")) {
    throw new Error("Missing editable native source paint");
  }
  sourceLayer.fill.color = { a: 1, b: 0, g: 0.5, r: 0 };
  const update = {
    componentId: sourceId,
    durationMs: 3000,
    itemId: primaryId,
    projectId,
    slotValues: {
      [id === "role_badge_team_intro" ? "role_label" : "label"]: {
        type: "text",
        value: "EDITED INTRO",
      },
      artwork: {
        type: "asset",
        value: { id: needed(imported[1]), kind: "asset", scope: "project" },
      },
    },
    startMs: 0,
    timeScale: 1,
    trimStartMs: 0,
  };
  const bytesBeforeInvalid = authoritative();
  const stale = await client.callTool({
    arguments: { ...update, expectedRevision: state.project.revision - 1 },
    name: "component_instance_update",
  });
  expect(stale.isError).toBe(true);
  expect(stale.structuredContent).toMatchObject({
    error: { code: "REVISION_CONFLICT", retryable: true },
  });
  expect(authoritative()).toEqual(bytesBeforeInvalid);
  await call(
    "component_update",
    {
      componentId: sourceId,
      expectedRevision: state.project.revision,
      projectId,
      ...edited,
    },
    writeResultSchema
  );
  state = await read();
  const beforeInstanceUpdate = state;
  await call(
    "component_instance_update",
    { ...update, expectedRevision: state.project.revision },
    writeResultSchema
  );
  state = await read();
  const customized = state;
  await call(
    "project_undo",
    { expectedRevision: state.project.revision, projectId },
    writeResultSchema
  );
  const undone = await read();
  expect(content(undone)).toEqual(content(beforeInstanceUpdate));
  await call(
    "project_redo",
    { expectedRevision: undone.project.revision, projectId },
    writeResultSchema
  );
  state = await read();
  expect(content(state)).toEqual(content(customized));
  setState(state);
  const changed = await capture(
    "preview_render_frame",
    { timeMs: 1500 },
    "edited"
  );
  const beforePixels = decode(needed(frames[1])),
    afterPixels = decode(changed);
  expect(afterPixels).not.toEqual(beforePixels);
  const pixel = (pixels: Buffer, x: number, y: number) => [
    ...pixels.subarray((y * 1920 + x) * 3, (y * 1920 + x) * 3 + 3),
  ];
  const probesByTarget = {
    illustrated_device_intro: { blue: [840, 400], source: [350, 250] },
    role_badge_team_intro: { blue: [440, 530], source: [200, 450] },
    sticker_motif_intro: { blue: [1040, 450], source: [600, 500] },
  };
  const sourceProbe = needed(
    probesByTarget[id as keyof typeof probesByTarget]
  ).source;
  const sourcePixel = pixel(
    afterPixels,
    needed(sourceProbe[0]),
    needed(sourceProbe[1])
  );
  assertNativeRgb(sourcePixel, [0, 128, 0]);
  const blueProbe = needed(
    probesByTarget[id as keyof typeof probesByTarget]
  ).blue;
  const editedBlue = pixel(
    afterPixels,
    needed(blueProbe[0]),
    needed(blueProbe[1])
  );
  assertNativeRgb(editedBlue, [0, 0, 255]);
  if (id === "role_badge_team_intro") {
    const independentRed = pixel(afterPixels, 1000, 530);
    assertNativeRgb(independentRed, [255, 0, 0]);
    const other = state.project.tracks
      .flatMap((t) => t.items)
      .find((item) => item.id === aliases.member_2);
    expect(other?.type === "component_instance" && other.slotValues).toEqual({
      role_label: { type: "text", value: "ROLE 02" },
    });
  }
  return state;
};
const verifyRoleOmissions = (
  observation: ReturnType<typeof observeIllustratedTarget>
) => {
  const roles = {
    illustrated_device_intro: [
      "device_frame",
      "screen_panel",
      "artwork",
      "label",
      "accent",
      "backdrop",
    ],
    role_badge_team_intro: [
      "background",
      "device_frame",
      "screen_panel",
      "label_panel",
      "accent",
      "artwork",
      "role_label",
      "rays",
      "sun_disc",
      "team_title",
      "backdrop",
    ],
    sticker_motif_intro: [
      "authored_sticker_contour",
      "artwork",
      "dot_backdrop",
      "sun_rays",
      "speed_lines",
      "label",
    ],
  };
  for (const role of needed(roles[observation.id as keyof typeof roles])) {
    const omitted = structuredClone(observation);
    omitted.leaves = omitted.leaves.filter((leaf) => leaf.id !== role);
    expect(() => validateIllustratedTarget(omitted)).toThrow();
  }
  if (observation.id === "role_badge_team_intro") {
    const noParent = structuredClone(observation);
    noParent.groups = 0;
    expect(() => validateIllustratedTarget(noParent)).toThrow("team_parent");
  }
};
export const verifyIllustratedWorkflow = async (
  client: Client,
  call: TeamCall,
  media: string,
  projects: string,
  exports: string,
  native: TeamNative
) => {
  const execute = (
    command: string,
    args: string[],
    maxBuffer = FRAME_BYTES * 4 + 1024 * 1024
  ) => {
    const result = spawnSync(command, args, { maxBuffer });
    expect(
      result.status,
      result.error?.message ?? result.stderr.toString()
    ).toBe(0);
    return result;
  };
  const decode = (path: string, video = true) =>
    execute(
      native.ffmpeg,
      video
        ? [
            "-v",
            "error",
            "-i",
            path,
            "-frames:v",
            "1",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-",
          ]
        : [
            "-v",
            "error",
            "-i",
            path,
            "-vn",
            "-ac",
            "2",
            "-ar",
            "48000",
            "-f",
            "f32le",
            "-",
          ]
    ).stdout;
  const sourcePcm = syntheticPcm();
  const paths = [
    join(media, "illustrated-original-red.bmp"),
    join(media, "illustrated-original-blue.bmp"),
    join(media, "illustrated-original-clock.wav"),
  ];
  writeFileSync(needed(paths[0]), originalArtwork(false));
  writeFileSync(needed(paths[1]), originalArtwork(true));
  writeFileSync(needed(paths[2]), syntheticWav(sourcePcm));
  const results: {
    projectId: string;
    state: State;
    evidence: object;
    recordReopened: () => void;
  }[] = [];
  mkdirSync(native.evidence, { recursive: true });
  await TARGET_IDS.reduce(
    (previous, id) =>
      previous.then(async () => {
        const created = await call(
          "project_create",
          { name: id, ...TARGET_TIMING.canvas },
          writeResultSchema
        );
        const { projectId } = created;
        const directory = join(projects, projectId);
        const read = () =>
          call("project_open", { projectId }, projectStateSchema);
        const imported: string[] = [];
        for (const [index, path] of paths.entries()) {
          // biome-ignore lint/performance/noAwaitInLoops: Imports consume the latest project revision.
          const state = await read();
          const result = await call(
            "asset_import",
            {
              expectedRevision: state.project.revision,
              mediaType: index === 2 ? "audio" : "image",
              path,
              projectId,
            },
            writeResultSchema
          );
          imported.push(needed(result.changedIds[0]));
        }
        let state = await read();
        const overlay = needed(
          state.project.tracks.find((t) => t.trackType === "overlay")?.id
        );
        const audio = needed(
          state.project.tracks.find((t) => t.trackType === "audio")?.id
        );
        const operations = illustratedTargetOperations(
          id,
          needed(imported[0]),
          overlay
        );
        const baseline = teamProjectInventory(directory);
        const failed = await client.callTool({
          arguments: {
            expectedRevision: state.project.revision,
            operations: [
              ...operations,
              { itemId: "missing", operation: "delete_item" },
            ],
            projectId,
          },
          name: "timeline_batch_edit",
        });
        expect(failed.isError).toBe(true);
        expect(failed.structuredContent).toMatchObject({
          error: { code: "ITEM_NOT_FOUND", retryable: false },
        });
        expect(teamProjectInventory(directory)).toEqual(baseline);
        expect(await read()).toEqual(state);
        const batch = await call(
          "timeline_batch_edit",
          { expectedRevision: state.project.revision, operations, projectId },
          writeResultSchema
        );
        await call(
          "timeline_add_media",
          {
            assetId: needed(imported[2]),
            durationMs: 3000,
            expectedRevision: batch.revision,
            projectId,
            startMs: 0,
            trackId: audio,
          },
          writeResultSchema
        );
        state = await read();
        const observation = observeIllustratedTarget(id, state);
        const inventory = validateIllustratedTarget(observation);
        verifyRoleOmissions(observation);
        const authoritative = () =>
          Object.fromEntries(
            [
              ["project", directory],
              ["exports", exports],
            ].flatMap(([prefix, root]) =>
              Object.entries(teamProjectInventory(needed(root))).map(
                ([path, bytes]) => [`${prefix}/${path}`, bytes]
              )
            )
          );
        const renderRequests: object[] = [];
        const capture = async (
          name: string,
          input: Record<string, unknown>,
          label: string
        ) => {
          const bytes = authoritative();
          const start = performance.now();
          const queued = await call(
            name,
            { expectedRevision: state.project.revision, projectId, ...input },
            jobSchema
          );
          const job = await terminal(call, queued.jobId);
          expect(job.status, JSON.stringify(job.error)).toBe("completed");
          expect(job.revision).toBe(state.project.revision);
          const artifact = needed(job.artifact);
          const after = authoritative();
          for (const [path, value] of Object.entries(bytes)) {
            expect(after[path], `Render changed preexisting ${path}`).toBe(
              value
            );
          }
          expect(
            Object.keys(after).filter((entry) => !Object.hasOwn(bytes, entry))
          ).toEqual([
            `${job.kind === "export" ? "exports" : "project"}/${artifact.relativePath.replaceAll("\\", "/")}`,
          ]);
          const path = join(
            job.kind === "export" ? exports : directory,
            artifact.relativePath
          );
          const evidencePath = join(
            native.evidence,
            `${native.mode}-${id}-${label}.${job.kind === "preview" ? "png" : "mp4"}`
          );
          writeFileSync(evidencePath, readFileSync(path));
          renderRequests.push({
            elapsedMs: performance.now() - start,
            evidencePath,
            input,
            job,
            name,
          });
          return evidencePath;
        };
        const frames: string[] = [];
        for (const timeMs of TARGET_TIMING.samples) {
          // biome-ignore lint/performance/noAwaitInLoops: Capture the three agreed samples in order.
          const path = await capture(
            "preview_render_frame",
            { timeMs },
            `sample-${timeMs}`
          );
          const pixels = decode(path);
          verifyIllustratedPixels(id, pixels, timeMs);
          const corrupt = Buffer.from(pixels);
          corrupt.fill(0, (100 * 1920 + 100) * 3, (100 * 1920 + 100) * 3 + 3);
          expect(() => verifyIllustratedPixels(id, corrupt, timeMs)).toThrow(
            "pixel"
          );
          expect(() =>
            verifyIllustratedPixels(id, Buffer.alloc(FRAME_BYTES), timeMs)
          ).toThrow("pixel");
          frames.push(path);
        }
        const range = await capture(
          "preview_review_range",
          { endMs: 3000, resolution: "project", startMs: 0 },
          "range"
        );
        const exported = await capture(
          "project_export_video",
          {
            format: "mp4",
            overwrite: false,
            relativePath: `${native.mode}-${id}.mp4`,
            resolution: "project",
          },
          "export"
        );
        const { probes, ssim, pcmRms, originalAudio } = verifyTargetMedia(
          id,
          range,
          exported,
          native,
          execute,
          decode,
          sourcePcm
        );
        const nativeBefore = authoritative();
        const cancellable = await call(
          "preview_review_range",
          {
            endMs: 3000,
            expectedRevision: state.project.revision,
            projectId,
            resolution: "project",
            startMs: 0,
          },
          jobSchema
        );
        await call("job_cancel", { jobId: cancellable.jobId }, jobSchema);
        const cancelled = await terminal(call, cancellable.jobId);
        expect(cancelled.status).toBe("cancelled");
        expect(cancelled.artifact).toBeUndefined();
        expect(authoritative()).toEqual(nativeBefore);
        state = await verifyNativeEdits({
          aliases: batch.aliases,
          authoritative,
          call,
          capture,
          client,
          decode,
          frames,
          id,
          imported,
          operations,
          projectId,
          read,
          setState: (next) => {
            state = next;
          },
          state,
        });
        validateIllustratedTarget(observeIllustratedTarget(id, state));
        const preserved = join(
          native.evidence,
          `${native.mode}-${id}-native-project`,
          projectId
        );
        cpSync(directory, preserved, { recursive: true });
        const evidence = {
          cancellation: cancelled,
          id,
          inventory,
          lifecycle: {
            lateAtomicFailure: true,
            nativeSourceEdit: true,
            overrideIsolation: id === "role_badge_team_intro",
            reconnectReopen: false,
            staleRevision: true,
            undoRedo: true,
          },
          nativeProject: preserved,
          observation,
          originalAudio,
          pcmRms,
          probes,
          projectId,
          renderRequests,
          ssim,
          stateRevision: state.project.revision,
          timing: TARGET_TIMING,
        };
        writeFileSync(
          join(native.evidence, `${native.mode}-${id}-evidence.json`),
          JSON.stringify(evidence, null, 2)
        );
        const recordReopened = () => {
          evidence.lifecycle.reconnectReopen = true;
          writeFileSync(
            join(native.evidence, `${native.mode}-${id}-evidence.json`),
            JSON.stringify(evidence, null, 2)
          );
        };
        results.push({ evidence, projectId, recordReopened, state });
      }),
    Promise.resolve()
  );
  return results;
};
