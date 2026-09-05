import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { afterAll, beforeAll, expect, it } from "vitest";
import { type ZodType, z } from "zod/v4";

import {
  jobSchema,
  projectStateSchema,
  speechEstimateSchema,
  speechVoiceListResultSchema,
  statusSchema,
  ttsResultSchema,
  ttsStatusSchema,
  writeResultSchema,
} from "../src/schemas";

import { verifyGroupWorkflow } from "./group-workflow";

const SHA256_PATTERN = /^[a-f0-9]{64}$/;

const root = mkdtempSync(join(tmpdir(), "opencut-agent-smoke-"));
const projects = join(root, "projects");
const media = join(root, "media");
const exportsDirectory = join(root, "exports");
const ttsModel = join(root, "tts-model");
const ttsWork = join(root, "tts-work");
mkdirSync(projects, { recursive: true });
mkdirSync(media, { recursive: true });
mkdirSync(exportsDirectory, { recursive: true });
mkdirSync(ttsModel, { recursive: true });
mkdirSync(ttsWork, { recursive: true });
const fixture = join(media, "frame.ppm");
copyFileSync(resolve(import.meta.dirname, "fixtures/frame.ppm"), fixture);
const mediaTool = resolve(import.meta.dirname, "fixtures/fake-media-tool.mjs");
const toolWrapper = (mode: "ffmpeg" | "ffprobe") => {
  const path = join(
    root,
    `${mode}${process.platform === "win32" ? ".cmd" : ""}`
  );
  const body =
    process.platform === "win32"
      ? `@bun "${mediaTool}" ${mode} %*\r\n`
      : `#!/bin/sh\nexec bun "${mediaTool}" ${mode} "$@"\n`;
  writeFileSync(path, body);
  if (process.platform !== "win32") {
    chmodSync(path, 0o755);
  }
  return path;
};
const fakeFfmpeg = toolWrapper("ffmpeg");
const fakeFfprobe = toolWrapper("ffprobe");

const client = new Client({ name: "opencut-smoke", version: "0.1.0" });
const bridgeExecutable = process.env.OPENCUT_TEST_BRIDGE_PATH ?? "bun";
const transport = new StdioClientTransport({
  args: process.env.OPENCUT_TEST_BRIDGE_PATH
    ? []
    : ["run", resolve(import.meta.dirname, "../src/index.ts")],
  command: bridgeExecutable,
  cwd: resolve(import.meta.dirname, ".."),
  env: {
    ...Object.fromEntries(
      Object.entries(process.env).filter(
        (entry): entry is [string, string] => entry[1] !== undefined
      )
    ),
    OPENCUT_ALLOWED_MEDIA_DIRS: media,
    OPENCUT_EXPORTS_DIR: exportsDirectory,
    OPENCUT_FFMPEG_PATH: fakeFfmpeg,
    OPENCUT_FFPROBE_PATH: fakeFfprobe,
    OPENCUT_HEADLESS_PATH:
      process.env.OPENCUT_TEST_HEADLESS_PATH ??
      resolve(
        import.meta.dirname,
        "../../../target/debug",
        process.platform === "win32"
          ? "opencut-headless.exe"
          : "opencut-headless"
      ),
    OPENCUT_KOKORO_MODEL_DIR: ttsModel,
    OPENCUT_KOKORO_PYTHON: process.env.OPENCUT_TEST_PYTHON ?? "python",
    OPENCUT_KOKORO_WORKER: resolve(
      import.meta.dirname,
      "fixtures/fake_tts_worker.py"
    ),
    OPENCUT_PROJECTS_DIR: projects,
    OPENCUT_TTS_WORK_DIR: ttsWork,
  },
  stderr: "inherit",
});

const call = async <Output>(
  name: string,
  arguments_: Record<string, unknown>,
  schema: ZodType<Output>
) => {
  const response = await client.callTool({ arguments: arguments_, name });
  if (response.isError) {
    throw new Error(
      JSON.stringify(response.structuredContent ?? response.content)
    );
  }
  return schema.parse(response.structuredContent);
};

const waitForJob = async (
  jobId: string,
  attemptsRemaining = 300
): Promise<ReturnType<typeof jobSchema.parse>> => {
  if (attemptsRemaining === 0) {
    throw new Error("render job timed out");
  }
  const job = await call("job_get_status", { jobId }, jobSchema);
  if (job.status === "completed") {
    return job;
  }
  if (job.status === "failed") {
    throw new Error(JSON.stringify(job.error));
  }
  await new Promise((resolvePromise) => setTimeout(resolvePromise, 100));
  return waitForJob(jobId, attemptsRemaining - 1);
};

const waitForTerminalJob = async (
  jobId: string,
  attemptsRemaining = 300
): Promise<ReturnType<typeof jobSchema.parse>> => {
  if (attemptsRemaining === 0) {
    throw new Error("job timed out");
  }
  const job = await call("job_get_status", { jobId }, jobSchema);
  if (["completed", "failed", "cancelled"].includes(job.status)) {
    return job;
  }
  await new Promise((resolvePromise) => setTimeout(resolvePromise, 25));
  return waitForTerminalJob(jobId, attemptsRemaining - 1);
};

const revisionOf = (result: ReturnType<typeof writeResultSchema.parse>) =>
  result.revision;

it("parents groups through standalone and atomic MCP tools with history", async () => {
  const created = await call(
    "project_create",
    { name: "Groups smoke" },
    writeResultSchema
  );
  const { projectId } = created;
  const state = await call(
    "project_get_state",
    { projectId },
    projectStateSchema
  );
  const trackId = state.project.tracks[1]?.id;
  if (!trackId) {
    throw new Error("overlay missing");
  }
  const group = await call(
    "add_group",
    { durationMs: 1000, expectedRevision: 0, projectId, startMs: 0, trackId },
    writeResultSchema
  );
  const [parentId] = group.changedIds;
  const added = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 1,
      operations: [
        {
          durationMs: 1000,
          operation: "add_group",
          resultAlias: "child",
          startMs: 0,
          trackId,
        },
        {
          itemId: "@child",
          operation: "item_set_parent",
          parent: { id: parentId, scope: "root" },
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  const itemId = added.aliases.child;
  const reopened = await call(
    "project_open",
    { projectId },
    projectStateSchema
  );
  expect(reopened.project.tracks[1]?.items[1]).toMatchObject({
    id: itemId,
    parent: { id: parentId, scope: "root" },
    type: "group",
  });
  const invalidResults = await Promise.all(
    [
      { expectedRevision: 0, itemId, parent: null },
      {
        expectedRevision: 2,
        itemId: parentId,
        parent: { id: itemId, scope: "root" },
      },
      { expectedRevision: 2, itemId, parent: { id: "absent", scope: "root" } },
    ].map((args) =>
      client.callTool({
        arguments: { projectId, ...args },
        name: "item_set_parent",
      })
    )
  );
  expect(invalidResults.every((result) => result.isError)).toBe(true);
  const unchanged = await call(
    "project_get_state",
    { projectId },
    projectStateSchema
  );
  expect(unchanged).toEqual(reopened);
  await call(
    "item_set_parent",
    { expectedRevision: 2, itemId, parent: null, projectId },
    writeResultSchema
  );
  await call(
    "project_undo",
    { expectedRevision: 3, projectId },
    writeResultSchema
  );
  const restored = await call(
    "project_get_state",
    { projectId },
    projectStateSchema
  );
  expect(restored.project.tracks[1]?.items[1]).toMatchObject({
    parent: { id: parentId, scope: "root" },
  });
  await call(
    "project_redo",
    { expectedRevision: 4, projectId },
    writeResultSchema
  );
  const detached = await call(
    "project_open",
    { projectId },
    projectStateSchema
  );
  expect(detached.project.tracks[1]?.items[1]?.parent).toBeUndefined();
});

beforeAll(async () => {
  await client.connect(transport);
});

afterAll(async () => {
  await client.close();
  rmSync(root, { force: true, recursive: true });
});

it("negotiates the public protocol version through MCP", async () => {
  const current = await call(
    "editor_get_status",
    { protocolVersion: 1 },
    statusSchema
  );
  expect(current.protocolVersion).toBe(1);

  let unsupportedRejected = false;
  try {
    const response = await client.callTool({
      arguments: { protocolVersion: 2 },
      name: "editor_get_status",
    });
    unsupportedRejected = response.isError === true;
  } catch {
    unsupportedRejected = true;
  }
  expect(unsupportedRejected).toBe(true);
});

it("edits a project and persists fake speech provenance through MCP", async () => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.ready).toBe(true);
  expect(status.subsystems).toMatchObject({
    editor: { ready: true },
    rendering: { ready: true },
    speech: { ready: true },
  });
  const speechStatus = await call("tts_get_status", {}, ttsStatusSchema);
  expect(speechStatus).toMatchObject({
    modelId: "fake/model",
    providerId: "fake-speech",
    sampleRateHz: 24_000,
  });

  const created = await call(
    "project_create",
    { name: "Smoke Intro" },
    writeResultSchema
  );
  let revision = revisionOf(created);
  let state = await call(
    "project_get_state",
    { projectId: created.projectId },
    projectStateSchema
  );
  const videoTrack = state.project.tracks.find(
    (track) => track.trackType === "video"
  );
  const overlayTrack = state.project.tracks.find(
    (track) => track.trackType === "overlay"
  );
  const audioTrack = state.project.tracks.find(
    (track) => track.trackType === "audio"
  );
  expect(videoTrack).toBeDefined();
  expect(overlayTrack).toBeDefined();
  if (!(videoTrack && overlayTrack && audioTrack)) {
    throw new Error("canonical tracks were not created");
  }

  const imported = await call(
    "asset_import",
    {
      expectedRevision: revision,
      mediaType: "image",
      path: fixture,
      projectId: created.projectId,
    },
    writeResultSchema
  );
  revision = revisionOf(imported);
  const assetId = imported.changedIds.at(0);
  if (!assetId) {
    throw new Error("asset ID was not returned");
  }

  const duplicate = await call(
    "asset_import",
    {
      expectedRevision: revision,
      mediaType: "image",
      path: fixture,
      projectId: created.projectId,
    },
    writeResultSchema
  );
  revision = revisionOf(duplicate);
  const duplicateAssetId = duplicate.changedIds.at(0);
  state = await call(
    "project_get_state",
    { projectId: created.projectId },
    projectStateSchema
  );
  const importedAssets = state.project.assets.filter((asset) =>
    [assetId, duplicateAssetId].includes(asset.id)
  );
  expect(importedAssets).toHaveLength(2);
  expect(importedAssets[0]?.contentHash).toEqual(
    importedAssets[1]?.contentHash
  );
  expect(importedAssets[0]?.projectRelativePath).toBe(
    importedAssets[1]?.projectRelativePath
  );
  expect(importedAssets[0]?.probe.hasVideo).toBe(true);
  if (!duplicateAssetId) {
    throw new Error("duplicate asset ID was not returned");
  }
  revision = revisionOf(
    await call(
      "asset_delete",
      {
        assetId: duplicateAssetId,
        expectedRevision: revision,
        projectId: created.projectId,
      },
      writeResultSchema
    )
  );

  const mediaItem = await call(
    "timeline_add_media",
    {
      assetId,
      durationMs: 5000,
      expectedRevision: revision,
      projectId: created.projectId,
      startMs: 0,
      trackId: videoTrack.id,
    },
    writeResultSchema
  );
  revision = revisionOf(mediaItem);
  const mediaItemId = mediaItem.changedIds.at(0);
  if (!mediaItemId) {
    throw new Error("media item ID was not returned");
  }

  const textItem = await call(
    "timeline_add_text",
    {
      color: "#ffffff",
      durationMs: 4000,
      expectedRevision: revision,
      fontSize: 72,
      projectId: created.projectId,
      startMs: 250,
      text: "OPEN CUT",
      trackId: overlayTrack.id,
    },
    writeResultSchema
  );
  revision = revisionOf(textItem);

  const tts = await call(
    "tts_generate_and_insert",
    {
      expectedRevision: revision,
      projectId: created.projectId,
      speed: 1,
      startMs: 500,
      text: "Local speech",
      trackId: audioTrack.id,
    },
    jobSchema
  );
  const ttsDone = await waitForJob(tts.jobId);
  expect(ttsDone.kind).toBe("tts");
  expect(ttsDone.result?.durationMs).toBe(100);
  expect(ttsDone.result).toMatchObject({
    language: "en-US",
    modelId: "fake/model",
    providerId: "fake-speech",
    voice: "test_voice",
  });
  revision = ttsDone.result?.revision ?? revision;

  state = await call(
    "project_get_state",
    { projectId: created.projectId },
    projectStateSchema
  );
  const speechAsset = state.project.assets.find(
    (asset) => asset.id === ttsDone.result?.assetId
  );
  expect(speechAsset).toMatchObject({
    fileName: "Test Voice - Local speech.wav",
    hasAudio: true,
    probe: { audioSampleRateHz: 24_000, durationMs: 100, hasAudio: true },
    sizeBytes: expect.any(Number),
  });
  expect(speechAsset?.contentHash).toMatchObject({
    algorithm: "sha256",
    digest: expect.stringMatching(SHA256_PATTERN),
  });
  expect(speechAsset?.origin).toMatchObject({
    generation: {
      modelId: "fake/model",
      modelVersion: "1",
      providerId: "fake-speech",
      request: {
        language: "en-US",
        speed: 1,
        text: "Local speech",
        voiceId: "test_voice",
      },
      sampleRateHz: 24_000,
    },
    type: "speech_synthesis",
  });
  revision = revisionOf(
    await call(
      "project_undo",
      { expectedRevision: revision, projectId: created.projectId },
      writeResultSchema
    )
  );
  state = await call(
    "project_get_state",
    { projectId: created.projectId },
    projectStateSchema
  );
  expect(
    state.project.assets.some((asset) => asset.id === ttsDone.result?.assetId)
  ).toBe(false);
  revision = revisionOf(
    await call(
      "project_redo",
      { expectedRevision: revision, projectId: created.projectId },
      writeResultSchema
    )
  );
  state = await call(
    "project_get_state",
    { projectId: created.projectId },
    projectStateSchema
  );
  expect(
    state.project.assets.find((asset) => asset.id === ttsDone.result?.assetId)
      ?.origin
  ).toEqual(speechAsset?.origin);

  const keyframed = await call(
    "timeline_set_keyframes",
    {
      expectedRevision: revision,
      itemId: mediaItemId,
      keyframes: [
        {
          easing: "linear",
          property: "position",
          timeMs: 0,
          value: { type: "position", x: 0, y: 0 },
        },
        {
          easing: "ease_out",
          property: "position",
          timeMs: 4000,
          value: { type: "position", x: 80, y: 40 },
        },
        {
          easing: "linear",
          property: "scale",
          timeMs: 0,
          value: { type: "scalar", value: 1 },
        },
        {
          easing: "ease_in_out",
          property: "scale",
          timeMs: 4000,
          value: { type: "scalar", value: 1.15 },
        },
      ],
      projectId: created.projectId,
    },
    writeResultSchema
  );
  revision = revisionOf(keyframed);

  revision = revisionOf(
    await call(
      "timeline_move_item",
      {
        expectedRevision: revision,
        itemId: mediaItemId,
        projectId: created.projectId,
        startMs: 100,
        trackId: videoTrack.id,
      },
      writeResultSchema
    )
  );
  revision = revisionOf(
    await call(
      "timeline_trim_item",
      {
        durationMs: 4500,
        expectedRevision: revision,
        itemId: mediaItemId,
        projectId: created.projectId,
        startMs: 100,
      },
      writeResultSchema
    )
  );
  revision = revisionOf(
    await call(
      "project_undo",
      { expectedRevision: revision, projectId: created.projectId },
      writeResultSchema
    )
  );

  state = await call(
    "project_get_state",
    { projectId: created.projectId, timeRange: { endMs: 5000, startMs: 0 } },
    projectStateSchema
  );
  expect(state.project.tracks.flatMap((track) => track.items)).toHaveLength(3);

  expect(readdirSync(ttsWork)).toHaveLength(0);
}, 60_000);

it("supports discoverable voices, preview, commit, discard, and in-place regeneration", async () => {
  const voices = await call(
    "speech_list_voices",
    { language: "en-US" },
    speechVoiceListResultSchema
  );
  expect(voices.voices).toEqual([
    expect.objectContaining({
      accent: "American English",
      available: true,
      id: "test_voice",
      label: "Test Voice",
      previewSupported: true,
    }),
  ]);
  const estimate = await call(
    "speech_estimate",
    {
      source: {
        text: "First sentence. Second sentence.",
        textOptions: { sentencePauseMs: 250 },
        type: "request",
      },
    },
    speechEstimateSchema
  );
  expect(estimate).toMatchObject({
    chunks: 2,
    cost: { amount: 0, billing: "local", currency: null },
    modelCached: true,
    resources: { minimumLogicalCpus: 2, recommendedLogicalCpus: 4 },
  });

  const created = await call(
    "project_create",
    { name: "Speech workflows" },
    writeResultSchema
  );
  let state = await call(
    "project_get_state",
    { projectId: created.projectId },
    projectStateSchema
  );
  const audioTrack = state.project.tracks.find(
    (track) => track.trackType === "audio"
  );
  if (!audioTrack) {
    throw new Error("canonical audio track was not created");
  }
  const previewJob = await call(
    "speech_preview",
    {
      source: {
        text: "First sentence. Second sentence.",
        textOptions: { sentencePauseMs: 250 },
        type: "request",
      },
    },
    jobSchema
  );
  const previewDone = await waitForJob(previewJob.jobId);
  expect(previewDone.speechPreview).toMatchObject({
    durationMs: 450,
    token: expect.any(String),
  });
  const playable = await client.callTool({
    arguments: { jobId: previewJob.jobId },
    name: "job_get_status",
  });
  expect(playable.content).toEqual(
    expect.arrayContaining([
      expect.objectContaining({ mimeType: "audio/wav", type: "audio" }),
    ])
  );
  const token = previewDone.speechPreview?.token;
  if (!token) {
    throw new Error("preview did not return a token");
  }
  const committed = await call(
    "speech_commit_preview",
    {
      expectedRevision: created.revision,
      placement: { startMs: 700, trackId: audioTrack.id, type: "insert" },
      projectId: created.projectId,
      token,
    },
    ttsResultSchema
  );
  expect(committed.durationMs).toBe(450);
  expect(readdirSync(ttsWork)).toHaveLength(0);

  const audioUpdated = await call(
    "timeline_set_audio",
    {
      audio: { fadeInMs: 50, fadeOutMs: 75, muted: false, volume: 0.6 },
      expectedRevision: committed.revision,
      itemId: committed.itemId,
      projectId: created.projectId,
    },
    writeResultSchema
  );
  const regenerateJob = await call(
    "speech_regenerate",
    {
      expectedRevision: audioUpdated.revision,
      itemId: committed.itemId,
      language: "en-GB",
      projectId: created.projectId,
      text: "Changed voice and text.",
      voice: "test_voice_gb",
    },
    jobSchema
  );
  const regenerated = await waitForJob(regenerateJob.jobId);
  expect(regenerated.result).toMatchObject({
    itemId: committed.itemId,
    replacedAssetId: committed.assetId,
    voice: "test_voice_gb",
  });
  state = await call(
    "project_get_state",
    { projectId: created.projectId },
    projectStateSchema
  );
  const item = state.project.tracks
    .flatMap((track) => track.items)
    .find((candidate) => candidate.id === committed.itemId);
  expect(item).toMatchObject({
    audio: { fadeInMs: 50, fadeOutMs: 75, muted: false, volume: 0.6 },
    id: committed.itemId,
    startMs: 700,
  });
  expect(
    state.project.assets.some((asset) => asset.id === committed.assetId)
  ).toBe(false);

  const discardJob = await call(
    "speech_preview",
    { source: { text: "Discard this preview.", type: "request" } },
    jobSchema
  );
  const discardDone = await waitForJob(discardJob.jobId);
  await call(
    "speech_discard_preview",
    { token: discardDone.speechPreview?.token },
    z.object({ discarded: z.literal(true), token: z.string() }).strict()
  );
  expect(readdirSync(ttsWork)).toHaveLength(0);
}, 60_000);

it("reuses completed speech after a revision conflict", async () => {
  const created = await call(
    "project_create",
    { name: "Speech conflict" },
    writeResultSchema
  );
  const state = await call(
    "project_get_state",
    { projectId: created.projectId },
    projectStateSchema
  );
  const audioTrack = state.project.tracks.find(
    (track) => track.trackType === "audio"
  );
  const overlayTrack = state.project.tracks.find(
    (track) => track.trackType === "overlay"
  );
  if (!(audioTrack && overlayTrack)) {
    throw new Error("canonical tracks were not created");
  }
  const queued = await call(
    "tts_generate_and_insert",
    {
      expectedRevision: 0,
      projectId: created.projectId,
      startMs: 0,
      text: "delay",
      trackId: audioTrack.id,
    },
    jobSchema
  );
  const concurrentEdit = await call(
    "timeline_add_text",
    {
      durationMs: 1000,
      expectedRevision: 0,
      fontSize: 48,
      projectId: created.projectId,
      startMs: 0,
      text: "Concurrent edit",
      trackId: overlayTrack.id,
    },
    writeResultSchema
  );
  const conflict = await waitForTerminalJob(queued.jobId);
  if (conflict.error?.code !== "REVISION_CONFLICT") {
    throw new Error(`unexpected TTS failure: ${JSON.stringify(conflict)}`);
  }
  expect(conflict).toMatchObject({
    error: { code: "REVISION_CONFLICT", retryable: true },
    generatedArtifact: {
      expiresAtMs: expect.any(Number),
      token: expect.any(String),
    },
    status: "failed",
  });
  const token = conflict.generatedArtifact?.token;
  if (!token) {
    throw new Error("revision conflict did not retain generated speech");
  }
  const committed = await call(
    "tts_commit_generated_artifact",
    { artifactToken: token, expectedRevision: concurrentEdit.revision },
    ttsResultSchema
  );
  expect(committed.revision).toBe(concurrentEdit.revision + 1);
  expect(readdirSync(ttsWork)).toHaveLength(0);
}, 30_000);

it("cancels fake speech through the MCP job contract", async () => {
  const created = await call(
    "project_create",
    { name: "Speech cancellation" },
    writeResultSchema
  );
  const state = await call(
    "project_get_state",
    { projectId: created.projectId },
    projectStateSchema
  );
  const trackId = state.project.tracks.find(
    (track) => track.trackType === "audio"
  )?.id;
  if (!trackId) {
    throw new Error("canonical audio track was not created");
  }
  const queued = await call(
    "tts_generate_and_insert",
    {
      expectedRevision: created.revision,
      projectId: created.projectId,
      startMs: 0,
      text: "hang",
      trackId,
    },
    jobSchema
  );
  await new Promise((resolvePromise) => setTimeout(resolvePromise, 50));
  const cancelled = await call(
    "job_cancel",
    { jobId: queued.jobId },
    jobSchema
  );
  expect(cancelled).toMatchObject({
    error: { code: "JOB_CANCELLED", retryable: true },
    status: "cancelled",
  });
  expect((await waitForTerminalJob(queued.jobId)).status).toBe("cancelled");
}, 10_000);

it("round-trips Transform2D through MCP batch, undo, redo, and reset", async () => {
  const created = await call(
    "project_create",
    { name: "Transform2D smoke" },
    writeResultSchema
  );
  const { projectId } = created;
  const state = await call(
    "project_get_state",
    { projectId },
    projectStateSchema
  );
  const trackId = state.project.tracks.find(
    (track) => track.trackType === "overlay"
  )?.id;
  if (!trackId) {
    throw new Error("overlay missing");
  }
  const transform2d = {
    anchor: { x: 0.5, y: 0.5 },
    opacity: 0.75,
    position: { unit: "normalized", x: 0.5, y: 0.5 },
    rotationDeg: 30,
    scaleX: 1.2,
    scaleY: 0.8,
    skewXDeg: 5,
    skewYDeg: -3,
  };
  const edited = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 0,
      operations: [
        {
          color: "#ff0000",
          durationMs: 1000,
          height: 10,
          operation: "add_rectangle",
          resultAlias: "box",
          startMs: 0,
          trackId,
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
          width: 20,
        },
        { itemId: "@box", operation: "update_item", transform2d },
      ],
      projectId,
    },
    writeResultSchema
  );
  const itemId = edited.aliases.box;
  const read = async () =>
    await call("project_get_state", { projectId }, projectStateSchema);
  expect(
    (await read()).project.tracks
      .flatMap((track) => track.items)
      .find((item) => item.id === itemId)
  ).toMatchObject({ transform2d });
  await expect(
    call(
      "timeline_update_item",
      {
        expectedRevision: 1,
        itemId,
        projectId,
        transform2d: { ...transform2d, scaleX: 0 },
      },
      writeResultSchema
    )
  ).rejects.toThrow();
  await call(
    "project_undo",
    { expectedRevision: 1, projectId },
    writeResultSchema
  );
  expect(
    (await read()).project.tracks.flatMap((track) => track.items)
  ).toHaveLength(0);
  await call(
    "project_redo",
    { expectedRevision: 2, projectId },
    writeResultSchema
  );
  expect(
    (await read()).project.tracks
      .flatMap((track) => track.items)
      .find((item) => item.id === itemId)
  ).toMatchObject({ transform2d });
  await call(
    "timeline_update_item",
    { expectedRevision: 3, itemId, projectId, transform2d: null },
    writeResultSchema
  );
  expect(
    (await read()).project.tracks
      .flatMap((track) => track.items)
      .find((item) => item.id === itemId)?.transform2d
  ).toBeUndefined();
});

it("persists explicit stacking through standalone and alias batch tools", async () => {
  const created = await call(
    "project_create",
    { name: "Stacking smoke" },
    writeResultSchema
  );
  const { projectId } = created;
  const state = await call(
    "project_get_state",
    { projectId },
    projectStateSchema
  );
  const trackId = state.project.tracks[1]?.id;
  if (!trackId) {
    throw new Error("overlay track missing");
  }
  const rectangle = (resultAlias: string) => ({
    color: "#ff0000",
    durationMs: 1000,
    height: 20,
    operation: "add_rectangle",
    resultAlias,
    startMs: 0,
    trackId,
    transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
    width: 30,
  });
  const added = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 0,
      operations: [
        rectangle("a"),
        rectangle("b"),
        { itemId: "@a", operation: "item_set_z_index", zIndex: -5 },
        { index: 0, itemId: "@b", operation: "item_reorder" },
        { index: 0, operation: "track_reorder", trackId },
      ],
      projectId,
    },
    writeResultSchema
  );
  const { a } = added.aliases;
  await call(
    "item_set_z_index",
    { expectedRevision: 1, itemId: a, projectId, zIndex: 7 },
    writeResultSchema
  );
  await call(
    "item_reorder",
    { expectedRevision: 2, index: 0, itemId: a, projectId },
    writeResultSchema
  );
  await call(
    "track_reorder",
    { expectedRevision: 3, index: 1, projectId, trackId },
    writeResultSchema
  );
  const reopened = await call(
    "project_open",
    { projectId },
    projectStateSchema
  );
  expect(reopened.project.tracks[1]?.items[0]).toMatchObject({
    id: a,
    stackOrder: 0,
    zIndex: 7,
  });
  const stale = await client.callTool({
    arguments: { expectedRevision: 0, index: 0, itemId: a, projectId },
    name: "item_reorder",
  });
  expect(stale.isError).toBe(true);
});

it("ungroups through standalone and alias MCP edits with atomic failures and history", async () => {
  await verifyGroupWorkflow(client, call);
});
