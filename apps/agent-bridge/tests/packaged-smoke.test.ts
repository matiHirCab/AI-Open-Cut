import {
  chmodSync,
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
import type { ZodType } from "zod/v4";

import {
  editDraftSchema,
  jobSchema,
  projectStateSchema,
  statusSchema,
  transcriptionStatusSchema,
  ttsStatusSchema,
  writeResultSchema,
} from "../src/schemas";

import { verifyComponentWorkflow } from "./component-workflow";
import { verifyGroupWorkflow } from "./group-workflow";

const root = mkdtempSync(join(tmpdir(), "opencut-packaged-test-"));
const directories = {
  exports: join(root, "exports"),
  media: join(root, "media"),
  model: join(root, "model"),
  projects: join(root, "projects"),
  work: join(root, "work"),
};
for (const directory of Object.values(directories)) {
  mkdirSync(directory, { recursive: true });
}
const mediaTool = resolve(import.meta.dirname, "fixtures/fake-media-tool.mjs");
const toolWrapper = (mode: "ffmpeg" | "ffprobe") => {
  const path = join(
    root,
    `${mode}${process.platform === "win32" ? ".cmd" : ""}`
  );
  writeFileSync(
    path,
    process.platform === "win32"
      ? `@bun "${mediaTool}" ${mode} %*\r\n`
      : `#!/bin/sh\nexec bun "${mediaTool}" ${mode} "$@"\n`
  );
  if (process.platform !== "win32") {
    chmodSync(path, 0o755);
  }
  return path;
};
const fakeFfmpeg = toolWrapper("ffmpeg");
const fakeFfprobe = toolWrapper("ffprobe");

const bridge = process.env.OPENCUT_TEST_BRIDGE_PATH;
const headless = process.env.OPENCUT_TEST_HEADLESS_PATH;
if (!(bridge && headless)) {
  throw new Error("packaged smoke paths were not provided by the build runner");
}
const client = new Client({ name: "packaged-smoke", version: "0.1.0" });
const transport = new StdioClientTransport({
  command: bridge,
  env: {
    ...Object.fromEntries(
      Object.entries(process.env).filter(
        (entry): entry is [string, string] => entry[1] !== undefined
      )
    ),
    OPENCUT_ALLOWED_MEDIA_DIRS: directories.media,
    OPENCUT_EXPORTS_DIR: directories.exports,
    OPENCUT_FFMPEG_PATH: fakeFfmpeg,
    OPENCUT_FFPROBE_PATH: fakeFfprobe,
    OPENCUT_HEADLESS_PATH: headless,
    OPENCUT_KOKORO_MODEL_DIR: directories.model,
    OPENCUT_KOKORO_PYTHON: process.env.OPENCUT_TEST_PYTHON ?? "python",
    OPENCUT_KOKORO_WORKER: resolve(
      import.meta.dirname,
      "fixtures/fake_tts_worker.py"
    ),
    OPENCUT_PROJECTS_DIR: directories.projects,
    OPENCUT_TRANSCRIPTION_PYTHON: process.env.OPENCUT_TEST_PYTHON ?? "python",
    OPENCUT_TRANSCRIPTION_WORKER: resolve(
      import.meta.dirname,
      "fixtures/fake_transcription_worker.py"
    ),
    OPENCUT_TTS_WORK_DIR: directories.work,
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
    throw new Error(JSON.stringify(response.structuredContent));
  }
  return schema.parse(response.structuredContent);
};

beforeAll(async () => await client.connect(transport));
afterAll(async () => {
  await client.close();
  rmSync(root, { force: true, recursive: true });
});

it("completes packaged editing, draft, speech, and transcription flows", async () => {
  await expect(
    call("editor_get_status", {}, statusSchema)
  ).resolves.toMatchObject({ ready: true });
  await expect(
    call("tts_get_status", {}, ttsStatusSchema)
  ).resolves.toMatchObject({ providerId: "fake-speech", ready: true });
  await expect(
    call("transcription_get_status", {}, transcriptionStatusSchema)
  ).resolves.toMatchObject({ providerId: "fake-transcriber", ready: true });
  const project = await call(
    "project_create",
    { name: "Packaged smoke" },
    writeResultSchema
  );
  const state = await call(
    "project_get_state",
    { projectId: project.projectId },
    projectStateSchema
  );
  const trackId = state.project.tracks.find(
    (track) => track.trackType === "audio"
  )?.id;
  if (!trackId) {
    throw new Error("audio track missing");
  }
  const queued = await call(
    "tts_generate_and_insert",
    {
      expectedRevision: project.revision,
      projectId: project.projectId,
      startMs: 0,
      text: "Packaged speech",
      trackId,
    },
    jobSchema
  );
  const waitForCompletion = async (
    queuedJob: ReturnType<typeof jobSchema.parse>,
    attemptsRemaining = 200
  ): Promise<ReturnType<typeof jobSchema.parse>> => {
    const current = await call(
      "job_get_status",
      { jobId: queuedJob.jobId },
      jobSchema
    );
    if (current.status === "completed" || attemptsRemaining === 0) {
      return current;
    }
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 25));
    return await waitForCompletion(queuedJob, attemptsRemaining - 1);
  };
  const terminal = await waitForCompletion(queued);
  expect(terminal).toMatchObject({
    result: {
      assetId: expect.any(String),
      itemId: expect.any(String),
      warnings: [],
    },
    status: "completed",
  });
  expect(readdirSync(directories.work)).toEqual([]);
  if (!(terminal.result && "assetId" in terminal.result)) {
    throw new Error("speech result missing");
  }

  const draft = await call(
    "draft_create",
    {
      expectedRevision: terminal.result.revision,
      label: "Packaged draft",
      operations: [
        { name: "B-roll", operation: "create_track", trackType: "video" },
      ],
      projectId: project.projectId,
    },
    editDraftSchema
  );
  const draftCommit = await call(
    "draft_commit",
    {
      draftId: draft.id,
      expectedRevision: terminal.result.revision,
      projectId: project.projectId,
    },
    writeResultSchema
  );
  const transcriptionJob = await call(
    "transcription_preview",
    { assetId: terminal.result.assetId, projectId: project.projectId },
    jobSchema
  );
  const transcript = await waitForCompletion(transcriptionJob);
  expect(transcript.transcriptionPreview?.segments[0]?.text).toBe(
    "Packaged caption"
  );
  if (!transcript.transcriptionPreview) {
    throw new Error("transcription preview missing");
  }
  const captionCommit = await call(
    "transcription_commit_preview",
    {
      expectedRevision: draftCommit.revision,
      projectId: project.projectId,
      token: transcript.transcriptionPreview.token,
    },
    writeResultSchema
  );
  const captioned = await call(
    "project_get_state",
    { projectId: project.projectId },
    projectStateSchema
  );
  expect(captionCommit.revision).toBe(draftCommit.revision + 1);
  expect(
    captioned.project.tracks
      .flatMap((candidate) => candidate.items)
      .some(
        (item) => item.type === "caption" && item.text === "Packaged caption"
      )
  ).toBe(true);
});

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

it("completes the packaged group workflow with aliases, rollback and history", async () => {
  await verifyGroupWorkflow(client, call);
  await verifyComponentWorkflow(client, call, directories.media);
});
