import { writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import catalog from "../../../contracts/timeline-audio-events-v1.json";
import {
  editDraftSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <T>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<T>
) => Promise<T>;
export const verifyAudioEventWorkflow = async (
  client: Client,
  call: Call,
  mediaDirectory: string
) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.projectSchemaVersion).toBe(catalog.projectSchemaVersion + 1);
  expect(status.subsystems.editor.capabilities).toContain(catalog.capability);
  const { projectId } = await call(
    "project_create",
    { name: "Semantic audio placement" },
    writeResultSchema
  );
  const wav = Buffer.alloc(44 + 1600);
  wav.write("RIFF", 0);
  wav.writeUInt32LE(wav.length - 8, 4);
  wav.write("WAVEfmt ", 8);
  wav.writeUInt32LE(16, 16);
  wav.writeUInt16LE(1, 20);
  wav.writeUInt16LE(1, 22);
  wav.writeUInt32LE(8000, 24);
  wav.writeUInt32LE(16_000, 28);
  wav.writeUInt16LE(2, 32);
  wav.writeUInt16LE(16, 34);
  wav.write("data", 36);
  wav.writeUInt32LE(1600, 40);
  for (let i = 0; i < 800; i += 1) {
    wav.writeInt16LE(i % 2 ? 300 : -300, 44 + i * 2);
  }
  const path = join(mediaDirectory, "semantic-audio-placement.wav");
  writeFileSync(path, wav);
  const asset = await call(
    "asset_import",
    { expectedRevision: 0, mediaType: "audio", path, projectId },
    writeResultSchema
  );
  const [assetId] = asset.changedIds;
  if (!assetId) {
    throw new Error("asset missing");
  }
  const read = () =>
    call("project_get_state", { projectId }, projectStateSchema);
  const placed = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 1,
      operations: [
        {
          busId: "sfx",
          defaultGainDb: -6,
          event: "impact",
          operation: "sound_event_register",
          resultAlias: "sound",
          variantAssetIds: [assetId],
          variantSeed: 0,
        },
        {
          name: "Events",
          operation: "create_track",
          resultAlias: "events",
          trackType: "audio",
        },
        {
          at: { type: "milliseconds", valueMs: 250 },
          durationMs: 80,
          event: "@sound",
          gainDb: -3,
          operation: "timeline_add_audio_event",
          resultAlias: "hit",
          scope: "root",
          trackId: "@events",
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  const trackId = placed.aliases.events;
  const itemId = placed.aliases.hit;
  if (!(trackId && itemId)) {
    throw new Error("placement aliases missing");
  }
  const item = () =>
    read().then(
      (state) => state.project.tracks.find((t) => t.id === trackId)?.items[0]
    );
  expect(await item()).toMatchObject({
    assetId,
    audioEvent: {
      busId: "sfx",
      defaultGainDb: -6,
      event: "impact",
      gainDb: -3,
      variantIndex: 0,
      variantSeed: 0,
    },
    durationMs: 80,
    id: itemId,
    startMs: 250,
    type: "media",
  });
  const placement = {
    at: { type: "milliseconds", valueMs: 500 },
    durationMs: 80,
    event: "impact",
    gainDb: -3,
    scope: "root",
    trackId,
  };
  const rejected = async (
    expectedRevision: number,
    event: string,
    code: string
  ) => {
    const before = await read();
    const response = await client.callTool({
      arguments: { projectId, ...placement, event, expectedRevision },
      name: catalog.operation,
    });
    expect(response.isError).toBe(true);
    expect(response.structuredContent).toMatchObject({
      error: { code, retryable: code === "REVISION_CONFLICT" },
    });
    expect(await read()).toEqual(before);
  };
  await rejected(1, "impact", "REVISION_CONFLICT");
  await rejected(2, "missing", "INVALID_ARGUMENT");
  const draft = await call(
    "draft_create",
    {
      expectedRevision: 2,
      operations: [{ operation: catalog.operation, ...placement }],
      projectId,
    },
    editDraftSchema
  );
  expect(draft.audioEventAssetIds).toEqual([assetId]);
  await call(
    "draft_commit",
    { draftId: draft.id, expectedRevision: 2, projectId },
    writeResultSchema
  );
  const committed = await read();
  expect(
    committed.project.tracks.find((t) => t.id === trackId)?.items
  ).toHaveLength(2);
  await call(
    "project_undo",
    { expectedRevision: 3, projectId },
    writeResultSchema
  );
  expect(
    (await read()).project.tracks.find((t) => t.id === trackId)?.items
  ).toHaveLength(1);
  await call(
    "project_redo",
    { expectedRevision: 4, projectId },
    writeResultSchema
  );
  await call("project_open", { projectId }, projectStateSchema);
  expect((await read()).project.tracks).toEqual(committed.project.tracks);
};
