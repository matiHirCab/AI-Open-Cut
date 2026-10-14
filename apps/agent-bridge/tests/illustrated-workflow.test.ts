import { expect, it } from "vitest";
import { headlessEditSchema } from "../src/schemas";
import {
  illustratedTargetOperations,
  ORIGINAL_TARGETS,
  TARGET_IDS,
  TARGET_TIMING,
} from "./fixtures/illustrated-targets";
import {
  type LeafObservation,
  observeIllustratedTarget,
  type TargetObservation,
  validateIllustratedTarget,
} from "./illustrated-target-oracle";

it("authors all original targets using unchanged native operation inputs and literal final timing", () => {
  expect(TARGET_IDS).toEqual([
    "illustrated_device_intro",
    "sticker_motif_intro",
    "role_badge_team_intro",
  ]);
  expect(TARGET_TIMING).toEqual({
    canvas: { fps: 30, height: 1080, width: 1920 },
    durationMs: 3000,
    range: [0, 3000],
    samples: [0, 1500, 2966],
  });
  for (const id of TARGET_IDS) {
    for (const operation of illustratedTargetOperations(
      id,
      "original_art",
      "overlay"
    )) {
      expect(() => headlessEditSchema.parse(operation)).not.toThrow();
      if ("durationMs" in operation) {
        expect(operation.durationMs).toBe(3000);
      }
    }
  }
  expect(ORIGINAL_TARGETS.every((t) => t.implemented === false)).toBe(true);
});
const leaf = (id: string, box: number[], text = ""): LeafObservation => ({
  animatedChannels: 0,
  artworkId: id === "artwork" ? "original_art" : undefined,
  box,
  id,
  kind: id === "artwork" ? "media" : "shape",
  pathCommands: 0,
  text,
});
const device = (): TargetObservation => ({
  artworkDimensions: [64],
  durationMs: 3000,
  groups: 0,
  id: "illustrated_device_intro",
  instances: 1,
  itemDurations: [3000, 3000],
  leaves: [
    leaf("device_frame", [260, 180, 880, 680]),
    leaf("screen_panel", [300, 220, 800, 520]),
    leaf("artwork", [620, 360, 256, 256]),
    { ...leaf("label", [1230, 330, 440, 100], "NATIVE INTRO"), kind: "text" },
    leaf("accent", [300, 770, 220, 32]),
    leaf("backdrop", [0, 0, 1920, 1080]),
  ],
  settings: { fps: 30, height: 1080, width: 1920 },
  storedArtworkAssets: 1,
});
it("accepts measured smaller inventories and rejects each original-role omission", () => {
  expect(validateIllustratedTarget(device())).toEqual({
    animatedChannels: 0,
    artworkCount: 1,
    artworkDimension: 64,
    distinctArtworkAssets: 1,
    instances: 1,
    pathCommands: 0,
    storedArtworkAssets: 1,
    textScalars: 12,
    visualLeaves: 6,
  });
  for (const role of device().leaves) {
    const missing = device();
    missing.leaves = missing.leaves.filter(
      (candidate) => candidate.id !== role.id
    );
    expect(() => validateIllustratedTarget(missing)).toThrow(
      "role/layout missing"
    );
  }
});
it("rejects altered duration, layout, canvas and every inclusive cap overflow", () => {
  const changed = device();
  changed.durationMs = 1000;
  expect(() => validateIllustratedTarget(changed)).toThrow("three-second");
  const shifted = device();
  if (shifted.leaves[0]) {
    shifted.leaves[0].box[0] = 261;
  }
  expect(() => validateIllustratedTarget(shifted)).toThrow("layout missing");
  const wrongCanvas = device();
  wrongCanvas.settings.fps = 24;
  expect(() => validateIllustratedTarget(wrongCanvas)).toThrow(
    "canvas changed"
  );
  for (const mutate of [
    (v: TargetObservation) => {
      v.instances = 2;
    },
    (v: TargetObservation) => {
      v.leaves.push(
        ...Array.from({ length: 25 }, () => leaf("extra", [0, 0, 1, 1]))
      );
    },
    (v: TargetObservation) => {
      if (v.leaves[0]) {
        v.leaves[0].text = "X".repeat(129);
      }
    },
    (v: TargetObservation) => {
      if (v.leaves[0]) {
        v.leaves[0].artworkId = "another";
      }
    },
    (v: TargetObservation) => {
      v.artworkDimensions = [1025];
    },
    (v: TargetObservation) => {
      if (v.leaves[0]) {
        v.leaves[0].pathCommands = 257;
      }
    },
    (v: TargetObservation) => {
      if (v.leaves[0]) {
        v.leaves[0].animatedChannels = 9;
      }
    },
  ]) {
    const candidate = device();
    mutate(candidate);
    expect(() => validateIllustratedTarget(candidate)).toThrow("cap exceeded");
  }
});

it("independently detects silent, omitted and wrong-clock synthetic audio", async () => {
  const { syntheticPcm, verifyOriginalAudio } = await import(
    "./illustrated-media-oracle"
  );
  const source = syntheticPcm();
  const decoded = Buffer.alloc(source.length * 8);
  for (let i = 0; i < source.length; i += 1) {
    decoded.writeFloatLE(source[i] ?? 0, i * 8);
    decoded.writeFloatLE(source[i] ?? 0, i * 8 + 4);
  }
  expect(verifyOriginalAudio(decoded, source)).toEqual({
    alignmentMs: 0,
    alignmentSamples: 0,
    correlation: 1,
  });
  expect(() =>
    verifyOriginalAudio(Buffer.alloc(decoded.length), source)
  ).toThrow("correlation failed");
  expect(() => verifyOriginalAudio(Buffer.alloc(0), source)).toThrow("omitted");
  const shifted = Buffer.alloc(decoded.length);
  decoded.copy(shifted, 1920 * 8, 0, decoded.length - 1920 * 8);
  expect(() => verifyOriginalAudio(shifted, source)).toThrow("clock exceeds");
  for (const channel of [0, 1]) {
    const silentChannel = Buffer.from(decoded);
    const shiftedChannel = Buffer.from(decoded);
    for (let i = 0; i < source.length; i += 1) {
      silentChannel.writeFloatLE(0, i * 8 + channel * 4);
      shiftedChannel.writeFloatLE(
        i >= 1920 ? (source[i - 1920] ?? 0) : 0,
        i * 8 + channel * 4
      );
    }
    expect(() => verifyOriginalAudio(silentChannel, source)).toThrow(
      "correlation failed"
    );
    expect(() => verifyOriginalAudio(shiftedChannel, source)).toThrow(
      "clock exceeds"
    );
  }
});

const sampleReport = () => {
  const observation = device();
  const job = (kind: string, index: number) => ({
    artifact: {
      mimeType: kind === "preview" ? "image/png" : "video/mp4",
      relativePath: `artifact-${index}`,
      sizeBytes: 1,
      warnings: [],
    },
    createdAtMs: 1,
    expiresAtMs: null,
    jobId: `job-${index}`,
    kind,
    persistence: "process",
    progress: 1,
    projectId: "owned-project",
    revision: 5,
    status: "completed",
    updatedAtMs: 2,
  });
  const renderRequests = [
    ...TARGET_TIMING.samples.map((timeMs, index) => ({
      elapsedMs: 1,
      evidencePath: `source-illustrated_device_intro-sample-${timeMs}.png`,
      input: { timeMs },
      job: job("preview", index),
      name: "preview_render_frame",
    })),
    {
      elapsedMs: 1,
      evidencePath: "source-illustrated_device_intro-range.mp4",
      input: { endMs: 3000, startMs: 0 },
      job: job("preview_range", 3),
      name: "preview_review_range",
    },
    {
      elapsedMs: 1,
      evidencePath: "source-illustrated_device_intro-export.mp4",
      input: {},
      job: job("export", 4),
      name: "project_export_video",
    },
    {
      elapsedMs: 1,
      evidencePath: "source-illustrated_device_intro-edited.png",
      input: { timeMs: 1500 },
      job: job("preview", 5),
      name: "preview_render_frame",
    },
  ];
  const { artifact: _artifact, ...cancelled } = job("preview_range", 6);
  return {
    cancellation: { ...cancelled, progress: 0, status: "cancelled" },
    id: "illustrated_device_intro",
    inventory: validateIllustratedTarget(observation),
    lifecycle: {
      lateAtomicFailure: true,
      nativeSourceEdit: true,
      overrideIsolation: false,
      reconnectReopen: true,
      staleRevision: true,
      undoRedo: true,
    },
    observation,
    originalAudio: [
      { alignmentMs: 0, alignmentSamples: 0, correlation: 1 },
      { alignmentMs: 0, alignmentSamples: 0, correlation: 1 },
    ],
    pcmRms: 0,
    probes: Array.from({ length: 2 }, () => ({
      format: { duration: "3" },
      streams: [
        { codec_type: "video", height: 1080, nb_frames: "90", width: 1920 },
        { codec_type: "audio" },
      ],
    })),
    projectId: "owned-project",
    renderRequests,
    ssim: 1,
    stateRevision: 9,
    timing: TARGET_TIMING,
  };
};
it("rejects missing, corrupted or misattributed release metadata and native role kinds", async () => {
  const { requireIllustratedEvidence } = await import("./illustrated-evidence");
  expect(requireIllustratedEvidence(sampleReport(), "source").projectId).toBe(
    "owned-project"
  );
  expect(() => requireIllustratedEvidence(sampleReport(), "packaged")).toThrow(
    "attribution differs"
  );
  for (const mutate of [
    (v: ReturnType<typeof sampleReport>) => {
      v.ssim = 0.98;
    },
    (v: ReturnType<typeof sampleReport>) => {
      v.pcmRms = 0.001;
    },
    (v: ReturnType<typeof sampleReport>) => {
      v.lifecycle.reconnectReopen = false;
    },
    (v: ReturnType<typeof sampleReport>) => {
      v.cancellation.status = "completed";
    },
    (v: ReturnType<typeof sampleReport>) => {
      v.renderRequests.pop();
    },
    (v: ReturnType<typeof sampleReport>) => {
      if (v.renderRequests[0]) {
        v.renderRequests[0].input = { timeMs: 500 };
      }
    },
    (v: ReturnType<typeof sampleReport>) => {
      if (v.observation.leaves[3]) {
        v.observation.leaves[3].kind = "shape";
      }
    },
    (v: ReturnType<typeof sampleReport>) => {
      v.originalAudio[0] = {
        alignmentMs: 0,
        alignmentSamples: 0,
        correlation: 0.98,
      };
    },
    (v: ReturnType<typeof sampleReport>) => {
      v.originalAudio[0] = {
        alignmentMs: 40,
        alignmentSamples: 1920,
        correlation: 1,
      };
    },
  ]) {
    const report = sampleReport();
    mutate(report);
    expect(() => requireIllustratedEvidence(report, "source")).toThrow();
  }
});

it("rejects moved Transform2D parents and unsupported fixture anchor/rotation/skew", () => {
  const transform2d = {
    anchor: { x: 0, y: 0 },
    opacity: 1,
    position: { unit: "pixels", x: 0, y: 0 },
    rotationDeg: 0,
    scaleX: 1,
    scaleY: 1,
    skewXDeg: 0,
    skewYDeg: 0,
  };
  const group = {
    durationMs: 3000,
    id: "team",
    transform: { positionX: 0, positionY: 0, scale: 1 },
    transform2d,
    type: "group",
  };
  const child = {
    durationMs: 3000,
    geometry: { height: 1, type: "rectangle", width: 1 },
    id: "leaf",
    parent: { id: "team" },
    transform: group.transform,
    transform2d: structuredClone(transform2d),
    type: "shape",
  };
  const state = {
    durationMs: 3000,
    project: {
      assets: [],
      components: [],
      settings: TARGET_TIMING.canvas,
      tracks: [{ items: [group, child] }],
    },
  } as unknown as Parameters<typeof observeIllustratedTarget>[1];
  expect(() =>
    observeIllustratedTarget("role_badge_team_intro", state)
  ).not.toThrow();
  group.transform2d.position.x = 100;
  expect(() =>
    observeIllustratedTarget("role_badge_team_intro", state)
  ).toThrow("parent transform changed");
  group.transform2d.position.x = 0;
  for (const mutate of [
    () => {
      child.transform2d.position.unit = "normalized";
    },
    () => {
      child.transform2d.anchor.x = 1;
    },
    () => {
      child.transform2d.rotationDeg = 1;
    },
    () => {
      child.transform2d.skewXDeg = 1;
    },
    () => {
      child.transform2d.skewYDeg = 1;
    },
  ]) {
    child.transform2d = structuredClone(transform2d);
    mutate();
    expect(() =>
      observeIllustratedTarget("role_badge_team_intro", state)
    ).toThrow("unsupported fixture transform");
  }
});

it("authors the reviewed native bevel with exact original bounds and vertices", () => {
  const operation = headlessEditSchema.parse(
    illustratedTargetOperations(
      "sticker_motif_intro",
      "original_art",
      "overlay"
    )[0]
  );
  if (operation.operation !== "component_create") {
    throw new Error("Missing sticker source");
  }
  const contour = operation.tracks
    .flatMap((track) => track.items)
    .find((item) => item.id === "authored_sticker_contour");
  if (!(contour?.type === "shape" && contour.geometry.type === "path")) {
    throw new Error("Missing authored native path");
  }
  expect(contour.transform2d?.position).toEqual({
    unit: "pixels",
    x: 560,
    y: 140,
  });
  expect(contour.geometry.path.commands).toEqual([
    { to: { x: 80, y: 0 }, type: "moveTo" },
    ...[
      [720, 0],
      [800, 80],
      [800, 720],
      [720, 800],
      [80, 800],
      [0, 720],
      [0, 80],
    ].map(([x, y]) => ({ to: { x, y }, type: "lineTo" })),
    { type: "close" },
  ]);
});

it("preserves the bounded native sparse dot lattice and rejects wrong dot or gap pixels", async () => {
  const operation = headlessEditSchema.parse(
    illustratedTargetOperations(
      "sticker_motif_intro",
      "original_art",
      "overlay"
    )[0]
  );
  if (operation.operation !== "component_create") {
    throw new Error("Missing sticker source");
  }
  const dots = operation.tracks
    .flatMap((track) => track.items)
    .find((item) => item.id === "dot_backdrop");
  expect(dots).toMatchObject({
    grid: {
      height: 1080,
      pattern: { radius: 5, spacingX: 320, spacingY: 320, type: "dot" },
      width: 1920,
    },
    type: "grid",
  });
  const centres = [0, 320, 640, 960, 1280, 1600, 1920].flatMap((x) =>
    [0, 320, 640, 960].map((y) => [x, y])
  );
  expect(centres).toHaveLength(28);
  expect(
    centres.filter(([x = 0, y = 0]) => x > 5 && x < 1915 && y > 5 && y < 1075)
  ).toHaveLength(15);
  const { FRAME_BYTES, verifyIllustratedPixels } = await import(
    "./illustrated-media-oracle"
  );
  const pixels = Buffer.alloc(FRAME_BYTES);
  const set = (x: number, y: number, color: number[]) => {
    pixels.set(color, (y * 1920 + x) * 3);
  };
  for (const [x, y, color] of [
    [100, 100, [24, 38, 61]],
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
    set(x, y, [...color]);
  }
  for (let x = 650; x < 750; x += 1) {
    set(x, 940, [255, 255, 255]);
  }
  expect(() =>
    verifyIllustratedPixels("sticker_motif_intro", pixels, 0)
  ).not.toThrow();
  set(120, 120, [0, 212, 255]);
  expect(() =>
    verifyIllustratedPixels("sticker_motif_intro", pixels, 0)
  ).toThrow("120,120");
  set(120, 120, [24, 38, 61]);
  set(320, 320, [24, 38, 61]);
  expect(() =>
    verifyIllustratedPixels("sticker_motif_intro", pixels, 0)
  ).toThrow("320,320");
});
