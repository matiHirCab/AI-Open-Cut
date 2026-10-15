import { isDeepStrictEqual } from "node:util";
import { z } from "zod/v4";
import { jobSchema } from "../src/schemas";
import { TARGET_IDS, TARGET_TIMING } from "./fixtures/illustrated-targets";
import {
  type TargetObservation,
  validateIllustratedTarget,
} from "./illustrated-target-oracle";

const leaf = z.object({
  animatedChannels: z.int().nonnegative(),
  artworkId: z.string().optional(),
  box: z.array(z.number().finite()).length(4),
  id: z.string(),
  kind: z.string(),
  pathCommands: z.int().nonnegative(),
  text: z.string(),
});
const observationSchema = z.object({
  artworkDimensions: z.array(z.int()),
  durationMs: z.int(),
  groups: z.int().nonnegative(),
  id: z.string(),
  instanceAnimatedChannels: z.int().nonnegative().optional(),
  instances: z.int().nonnegative(),
  itemDurations: z.array(z.int()).min(1),
  leaves: z.array(leaf),
  settings: z.object({ fps: z.int(), height: z.int(), width: z.int() }),
  storedArtworkAssets: z.int().nonnegative(),
});
const evidenceSchema = z.object({
  cancellation: jobSchema,
  id: z.string(),
  inventory: z.record(z.string(), z.number()),
  lifecycle: z.object({
    lateAtomicFailure: z.literal(true),
    nativeSourceEdit: z.literal(true),
    overrideIsolation: z.boolean(),
    reconnectReopen: z.literal(true),
    staleRevision: z.literal(true),
    undoRedo: z.literal(true),
  }),
  observation: observationSchema,
  originalAudio: z
    .array(
      z.object({
        alignmentMs: z.number().finite(),
        alignmentSamples: z.int().min(-1600).max(1600),
        correlation: z.number().min(0.99).max(1),
      })
    )
    .length(2),
  pcmRms: z.number().min(0).max(0.0001),
  probes: z
    .array(
      z
        .object({
          format: z.object({ duration: z.string() }).passthrough(),
          streams: z.array(
            z
              .object({
                codec_type: z.string(),
                height: z.number().optional(),
                nb_frames: z.string().optional(),
                width: z.number().optional(),
              })
              .passthrough()
          ),
        })
        .passthrough()
    )
    .length(2),
  projectId: z.string(),
  renderRequests: z
    .array(
      z.object({
        elapsedMs: z.number().finite().nonnegative(),
        evidencePath: z.string(),
        input: z.record(z.string(), z.unknown()),
        job: jobSchema,
        name: z.string(),
      })
    )
    .length(6),
  ssim: z.number().min(0.99).max(1),
  stateRevision: z.int().nonnegative(),
  timing: z.unknown(),
});
const requireCondition = (condition: boolean, message: string) => {
  if (!condition) {
    throw new Error(`Illustrated evidence: ${message}`);
  }
};
const requireRoleKinds = (observation: TargetObservation) => {
  for (const role of observation.leaves) {
    let expected = "shape";
    if (["label", "role_label", "team_title"].includes(role.id)) {
      expected = "text";
    }
    if (role.id === "artwork") {
      expected = "media";
    }
    if (role.id === "dot_backdrop") {
      expected = "grid";
    }
    requireCondition(role.kind === expected, `${role.id} native kind differs`);
    if (expected === "text") {
      requireCondition(role.text.trim().length > 0, "empty native label");
    }
    if (expected === "media") {
      requireCondition(Boolean(role.artworkId), "unmanaged artwork");
    }
  }
};
// Metadata validation is necessary for attribution; it never substitutes for the
// real MCP assertions, independent decoded pixels/audio, or actual GUI input.
export const requireIllustratedEvidence = (
  value: unknown,
  mode: "source" | "packaged"
) => {
  const evidence = evidenceSchema.parse(value);
  requireCondition(TARGET_IDS.includes(evidence.id), "unknown original target");
  requireCondition(
    evidence.id === evidence.observation.id,
    "target attribution differs"
  );
  requireCondition(
    isDeepStrictEqual(evidence.timing, TARGET_TIMING),
    "agreed timing differs"
  );
  const observation = evidence.observation as TargetObservation;
  requireRoleKinds(observation);
  requireCondition(
    isDeepStrictEqual(
      evidence.inventory,
      validateIllustratedTarget(observation)
    ),
    "inventory attribution differs"
  );
  requireCondition(
    evidence.cancellation.status === "cancelled" &&
      !evidence.cancellation.artifact &&
      evidence.cancellation.projectId === evidence.projectId,
    "cancellation not established"
  );
  requireCondition(
    evidence.id !== "role_badge_team_intro" ||
      evidence.lifecycle.overrideIsolation,
    "team isolation unverified"
  );
  for (const audio of evidence.originalAudio) {
    requireCondition(
      Math.abs(audio.alignmentMs - audio.alignmentSamples / 48) < 1e-9,
      "audio clock attribution differs"
    );
  }
  const samples = evidence.renderRequests.filter((request) =>
    request.evidencePath.includes("-sample-")
  );
  requireCondition(
    JSON.stringify(samples.map((request) => request.input.timeMs)) ===
      JSON.stringify(TARGET_TIMING.samples),
    "sample coverage differs"
  );
  for (const request of evidence.renderRequests) {
    requireCondition(
      request.job.status === "completed" &&
        request.job.projectId === evidence.projectId &&
        Boolean(request.job.artifact) &&
        request.evidencePath.includes(`${mode}-${evidence.id}-`),
      "runtime/job attribution differs"
    );
  }
  requireCondition(
    evidence.renderRequests.filter((r) => r.name === "preview_review_range")
      .length === 1 &&
      evidence.renderRequests.filter((r) => r.name === "project_export_video")
        .length === 1,
    "range/export coverage differs"
  );
  for (const probe of evidence.probes) {
    const video = probe.streams.find((stream) => stream.codec_type === "video");
    requireCondition(
      video?.width === 1920 &&
        video.height === 1080 &&
        video.nb_frames === "90" &&
        probe.streams.some((stream) => stream.codec_type === "audio") &&
        Math.abs(Number(probe.format.duration) - 3) <= 1 / 30,
      "full-range media differs"
    );
  }
  return evidence;
};
