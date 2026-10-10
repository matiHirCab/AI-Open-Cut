import { describe, expect, it } from "vitest";
import {
  RELEASE_FONTS,
  RELEASE_RECIPE_SHA,
  requireMotionCoreReport,
} from "../scripts/motion-release-report";

const ENVELOPE = /envelope/u;
const identity = {
  architecture: "x86_64",
  nonce: "fresh-owned-run",
  platform: "linux",
};
const media = () => ({
  format: { duration: "6.000000" },
  streams: [
    {
      avg_frame_rate: "10/1",
      codec_name: "h264",
      codec_type: "video",
      height: 108,
      nb_frames: "60",
      width: 192,
    },
    {
      channels: 2,
      codec_name: "aac",
      codec_type: "audio",
      duration: "6.000000",
      sample_rate: "48000",
    },
  ],
});
const capture = () => ({
  alignmentOffsetSamples: 0,
  coldFinalCalls: 1,
  elapsedMs: 600_000,
  exportProbe: media(),
  memorySamples: 4,
  nativeMembersObserved: 1,
  pcmRms: 0.0001,
  peakResidentBytes: 2_147_483_648,
  rangeProbe: media(),
  ssim: 0.99,
  warmFinalCalls: 0,
});
const report = () => ({
  architecture: "x86_64",
  buildProfile: "release",
  cacheMode: "native_reuse",
  captures: [capture(), capture(), capture()] as [
    ReturnType<typeof capture>,
    ReturnType<typeof capture>,
    ReturnType<typeof capture>,
  ],
  fixtureSha256: RELEASE_RECIPE_SHA,
  fonts: { ...RELEASE_FONTS },
  instrumented: false,
  measuredCaptures: 3,
  memoryAggregation: "maximum_sampled_resident",
  memoryScope: "isolated_test_process_tree",
  platform: "linux",
  runNonce: identity.nonce,
  sampleIntervalMs: 5,
  schemaVersion: 44,
  timingScope: "direct_whole_capture_including_preparation_and_oracles",
  tools: { ffmpeg: "ffmpeg version actual", ffprobe: "ffprobe version actual" },
  version: 1,
  warmupCaptures: 1,
  witnesses: {
    cueTimesMs: [500, 1000, 1500, 2400, 3200, 4300],
    goldNegative: true,
    groups: 10,
    historyReopen: true,
    operations: 67,
    stateFailures: true,
    voiceNegative: true,
  },
});

describe("private release measurement report", () => {
  it("accepts exact inclusive Linux envelope and refuses one unit over in every capture", () => {
    expect(() => requireMotionCoreReport(report(), identity)).not.toThrow();
    for (const index of [0, 1, 2]) {
      for (const field of ["elapsedMs", "peakResidentBytes"] as const) {
        const value = report();
        const row = value.captures[index];
        if (!row) {
          throw new Error("control capture absent");
        }
        row[field] += 1;
        expect(() => requireMotionCoreReport(value, identity)).toThrow(
          ENVELOPE
        );
      }
    }
  });
  it("reports other OS observations without claiming the Linux envelope", () => {
    for (const platform of ["macos", "windows"]) {
      const value = report();
      value.platform = platform;
      if (platform === "macos") {
        value.cacheMode = "native_identity_bypass";
        for (const row of value.captures) {
          row.warmFinalCalls = 1;
        }
      }
      for (const row of value.captures) {
        row.elapsedMs += 1;
        row.peakResidentBytes += 1;
      }
      expect(() =>
        requireMotionCoreReport(value, { ...identity, platform })
      ).not.toThrow();
    }
  });
  it("refuses absent, unknown, stale, instrumented and unlike-scope fields", () => {
    const good = report();
    for (const field of Object.keys(good)) {
      const value = { ...good } as Record<string, unknown>;
      delete value[field];
      expect(() => requireMotionCoreReport(value, identity)).toThrow();
    }
    for (const [field, replacement] of Object.entries({
      architecture: "aarch64",
      buildProfile: "debug",
      cacheMode: "disguised_wrapper",
      fixtureSha256: "changed",
      instrumented: true,
      measuredCaptures: 2,
      memoryAggregation: "sum",
      memoryScope: "only_parent",
      platform: "windows",
      runNonce: "stale",
      sampleIntervalMs: 6,
      schemaVersion: 45,
      timingScope: "subtract_stages",
      version: 2,
      warmupCaptures: 0,
    })) {
      expect(() =>
        requireMotionCoreReport({ ...good, [field]: replacement }, identity)
      ).toThrow();
    }
    expect(() =>
      requireMotionCoreReport({ ...good, unknown: 1 }, identity)
    ).toThrow();
    expect(() => requireMotionCoreReport(null, identity)).toThrow();
  });
  it("refuses missing, excess and malformed captures and false media witnesses", () => {
    for (const captures of [
      [],
      [capture()],
      [...report().captures, capture()],
      [null, capture(), capture()],
    ]) {
      expect(() =>
        requireMotionCoreReport({ ...report(), captures }, identity)
      ).toThrow();
    }
    for (const key of Object.keys(capture())) {
      const row = capture() as Record<string, unknown>;
      delete row[key];
      expect(() =>
        requireMotionCoreReport(
          { ...report(), captures: [row, capture(), capture()] },
          identity
        )
      ).toThrow();
    }
    for (const [field, bad] of Object.entries({
      alignmentOffsetSamples: 1,
      coldFinalCalls: 0,
      elapsedMs: 0,
      exportProbe: null,
      memorySamples: 0,
      nativeMembersObserved: 0,
      pcmRms: 0.000_100_01,
      peakResidentBytes: -1,
      rangeProbe: { streams: [] },
      ssim: 0.989_99,
      warmFinalCalls: 1,
    })) {
      expect(() =>
        requireMotionCoreReport(
          {
            ...report(),
            captures: [{ ...capture(), [field]: bad }, capture(), capture()],
          },
          identity
        )
      ).toThrow();
    }
    for (const bad of [
      Number.NaN,
      Number.POSITIVE_INFINITY,
      1.5,
      Number.MAX_SAFE_INTEGER + 1,
      "1",
      null,
    ]) {
      expect(() =>
        requireMotionCoreReport(
          {
            ...report(),
            captures: [{ ...capture(), elapsedMs: bad }, capture(), capture()],
          },
          identity
        )
      ).toThrow();
    }
  });
  it("refuses altered stream facts, native identity, fonts and independent controls", () => {
    const good = report();
    for (const name of Object.keys(RELEASE_FONTS)) {
      expect(() =>
        requireMotionCoreReport(
          { ...good, fonts: { ...good.fonts, [name]: "changed" } },
          identity
        )
      ).toThrow();
    }
    for (const field of Object.keys(good.witnesses)) {
      expect(() =>
        requireMotionCoreReport(
          { ...good, witnesses: { ...good.witnesses, [field]: false } },
          identity
        )
      ).toThrow();
    }
    for (const tool of ["ffmpeg", "ffprobe"]) {
      expect(() =>
        requireMotionCoreReport(
          { ...good, tools: { ...good.tools, [tool]: "" } },
          identity
        )
      ).toThrow();
    }
    for (const [index, field, bad] of [
      [0, "width", 1280],
      [0, "nb_frames", "0"],
      [0, "avg_frame_rate", "24/1"],
      [1, "channels", 1],
      [1, "sample_rate", "44100"],
      [1, "duration", "4.0"],
      [1, "duration", "NaN"],
      [1, "duration", ""],
    ] as const) {
      const value = report();
      const [row] = value.captures;
      const stream = row.rangeProbe.streams[index];
      if (!stream) {
        throw new Error("stream control absent");
      }
      Object.assign(stream, { [field]: bad });
      expect(() => requireMotionCoreReport(value, identity)).toThrow();
    }
    for (const duration of ["", "NaN", "Infinity", "5.89", "6.11"]) {
      const value = report();
      const [row] = value.captures;
      row.exportProbe.format.duration = duration;
      expect(() => requireMotionCoreReport(value, identity)).toThrow();
    }
  });
});
