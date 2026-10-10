export const RELEASE_RECIPE_SHA =
  "41deb91f35ccbbf379e7d3ffd0e4d402077fbaa37bee3a9253363fe8221e9cc0";
export const RELEASE_FONTS = {
  "DejaVuSans-Bold.ttf":
    "e6476c1b80502924294eed40894c5b18e06c181444ca953e5334262df9c27724",
  "DejaVuSans-BoldOblique.ttf":
    "eb436dca0c2594b73d8b603b892e374fdfd8d885d25ffb4f18df4c4c0b49e50f",
  "DejaVuSans-Oblique.ttf":
    "4af75fa16ee6d3ad43e1ecec41862c24954af26a55c6bb1ebb27bd486a50f5f4",
  "DejaVuSans.ttf":
    "7da195a74c55bef988d0d48f9508bd5d849425c1770dba5d7bfc6ce9ed848954",
} as const;

const record = (value: unknown): Record<string, unknown> => {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("Missing release evidence object");
  }
  return value as Record<string, unknown>;
};
const closed = (value: unknown, keys: string[]) => {
  const object = record(value);
  if (Object.keys(object).sort().join(",") !== [...keys].sort().join(",")) {
    throw new Error("Unknown or absent release evidence fields");
  }
  return object;
};
const requireFields = (
  object: Record<string, unknown>,
  expected: Record<string, unknown>
) => {
  for (const [name, value] of Object.entries(expected)) {
    if (JSON.stringify(object[name]) !== JSON.stringify(value)) {
      throw new Error(`Release identity or witness mismatch: ${name}`);
    }
  }
};
const positiveInteger = (value: unknown): number => {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value <= 0) {
    throw new Error("Absent, unsafe or non-positive release measurement");
  }
  return value;
};
const finite = (value: unknown, min: number, max: number) => {
  if (
    typeof value !== "number" ||
    !Number.isFinite(value) ||
    value < min ||
    value > max
  ) {
    throw new Error("Invalid native media witness");
  }
};
const requireMedia = (value: unknown) => {
  const probe = record(value);
  if (!Array.isArray(probe.streams) || probe.streams.length !== 2) {
    throw new Error("Both actual video and audio streams required");
  }
  const streams = probe.streams.map(record);
  const video = streams.find((stream) => stream.codec_type === "video");
  const audio = streams.find((stream) => stream.codec_type === "audio");
  if (!(video && audio)) {
    throw new Error("Missing native stream");
  }
  requireFields(video, {
    avg_frame_rate: "10/1",
    codec_name: "h264",
    height: 108,
    nb_frames: "60",
    width: 192,
  });
  requireFields(audio, {
    channels: 2,
    codec_name: "aac",
    sample_rate: "48000",
  });
  if (typeof audio.duration !== "string" || audio.duration.trim() === "") {
    throw new Error("Actual audio duration required");
  }
  finite(Number(audio.duration), 5.9, 6.1);
  const { duration } = record(probe.format);
  if (typeof duration !== "string" || duration.trim() === "") {
    throw new Error("Actual media duration required");
  }
  finite(Number(duration), 5.9, 6.1);
};

/** Private workload evidence, never a project validator or #15 rebaseline. */
export const requireMotionCoreReport = (
  value: unknown,
  identity: { nonce: string; platform: string; architecture: string }
) => {
  if (
    !(
      identity.nonce &&
      ["linux", "windows", "macos"].includes(identity.platform) &&
      ["x86_64", "aarch64"].includes(identity.architecture)
    )
  ) {
    throw new Error("Unsupported release measurement identity");
  }
  const report = closed(value, [
    "version",
    "runNonce",
    "fixtureSha256",
    "schemaVersion",
    "platform",
    "architecture",
    "buildProfile",
    "instrumented",
    "cacheMode",
    "fonts",
    "tools",
    "warmupCaptures",
    "measuredCaptures",
    "sampleIntervalMs",
    "memoryScope",
    "memoryAggregation",
    "timingScope",
    "captures",
    "witnesses",
  ]);
  requireFields(report, {
    architecture: identity.architecture,
    buildProfile: "release",
    cacheMode:
      identity.platform === "macos" ? "native_identity_bypass" : "native_reuse",
    fixtureSha256: RELEASE_RECIPE_SHA,
    instrumented: false,
    measuredCaptures: 3,
    memoryAggregation: "maximum_sampled_resident",
    memoryScope: "isolated_test_process_tree",
    platform: identity.platform,
    runNonce: identity.nonce,
    sampleIntervalMs: 5,
    schemaVersion: 44,
    timingScope: "direct_whole_capture_including_preparation_and_oracles",
    version: 1,
    warmupCaptures: 1,
  });
  requireFields(
    closed(report.fonts, Object.keys(RELEASE_FONTS)),
    RELEASE_FONTS
  );
  const tools = closed(report.tools, ["ffmpeg", "ffprobe"]);
  for (const name of ["ffmpeg", "ffprobe"]) {
    if (
      typeof tools[name] !== "string" ||
      !tools[name].startsWith(`${name} version `) ||
      tools[name].includes("\n")
    ) {
      throw new Error("Actual native version required");
    }
  }
  const expectedWitnesses = {
    cueTimesMs: [500, 1000, 1500, 2400, 3200, 4300],
    goldNegative: true,
    groups: 10,
    historyReopen: true,
    operations: 67,
    stateFailures: true,
    voiceNegative: true,
  };
  requireFields(
    closed(report.witnesses, Object.keys(expectedWitnesses)),
    expectedWitnesses
  );
  if (!Array.isArray(report.captures) || report.captures.length !== 3) {
    throw new Error("Exactly three actual measured captures required");
  }
  for (const observation of report.captures) {
    const capture = closed(observation, [
      "elapsedMs",
      "peakResidentBytes",
      "memorySamples",
      "nativeMembersObserved",
      "coldFinalCalls",
      "warmFinalCalls",
      "ssim",
      "pcmRms",
      "alignmentOffsetSamples",
      "rangeProbe",
      "exportProbe",
    ]);
    const elapsed = positiveInteger(capture.elapsedMs);
    const memory = positiveInteger(capture.peakResidentBytes);
    positiveInteger(capture.memorySamples);
    positiveInteger(capture.nativeMembersObserved);
    positiveInteger(capture.coldFinalCalls);
    requireFields(capture, { alignmentOffsetSamples: 0 });
    if (identity.platform === "macos") {
      positiveInteger(capture.warmFinalCalls);
    } else {
      requireFields(capture, { warmFinalCalls: 0 });
    }
    finite(capture.ssim, 0.99, 1);
    finite(capture.pcmRms, 0, 0.0001);
    requireMedia(capture.rangeProbe);
    requireMedia(capture.exportProbe);
    if (
      identity.platform === "linux" &&
      identity.architecture === "x86_64" &&
      (elapsed > 600_000 || memory > 2_147_483_648)
    ) {
      throw new Error(
        "Linux release workload exceeds inclusive private envelope"
      );
    }
  }
};
