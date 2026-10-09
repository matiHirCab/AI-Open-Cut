import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import catalog from "../../../contracts/audio-buses-v1.json";
import masterNormalization from "../../../contracts/master-normalization-v1.json";
import {
  editDraftSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;
export const verifyAudioBusWorkflow = async (client: Client, call: Call) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(catalog.projectSchemaVersion).toBe(39);
  expect(status.projectSchemaVersion).toBe(
    masterNormalization.projectSchemaVersion
  );
  expect(status.subsystems.editor.capabilities).toContain(catalog.capability);
  const { tools } = await client.listTools();
  for (const name of catalog.operations) {
    expect(tools.filter((tool) => tool.name === name)).toHaveLength(1);
  }
  const { projectId } = await call(
    "project_create",
    { name: "Bus routing workflow" },
    writeResultSchema
  );
  const read = () =>
    call("project_get_state", { projectId }, projectStateSchema);
  const initial = await read();
  expect(initial.project.audioBuses).toEqual(catalog.defaultBuses);
  const audio = initial.project.tracks.find(
    (track) => track.trackType === "audio"
  );
  const overlay = initial.project.tracks.find(
    (track) => track.trackType === "overlay"
  );
  if (!(audio && overlay)) {
    throw new Error("Canonical initial tracks missing");
  }
  const root = join(status.paths.projectsDirectory.resolvedPath, projectId);
  const files = () => {
    const result: Record<string, string> = {};
    const visit = (path: string) => {
      for (const name of readdirSync(path)) {
        const child = join(path, name);
        if (statSync(child).isDirectory()) {
          visit(child);
        } else {
          result[child] = readFileSync(child).toString("base64");
        }
      }
    };
    visit(root);
    return result;
  };
  const rejected = async (
    name: string,
    args: Record<string, unknown>,
    code: string
  ) => {
    const before = await read();
    const bytes = files();
    const result = await client.callTool({
      arguments: { projectId, ...args },
      name,
    });
    expect(result.isError).toBe(true);
    expect(result.structuredContent).toMatchObject({
      error: { code, retryable: code === "REVISION_CONFLICT" },
    });
    expect(await read()).toEqual(before);
    expect(files()).toEqual(bytes);
  };
  const route = { busId: "music", scope: "root", trackId: audio.id };
  await rejected(
    "audio_track_route",
    { ...route, expectedRevision: 9 },
    "REVISION_CONFLICT"
  );
  await rejected(
    "audio_track_route",
    { ...route, expectedRevision: 0, trackId: "absent" },
    "TRACK_NOT_FOUND"
  );
  await rejected(
    "audio_track_route",
    { ...route, busId: "absent", expectedRevision: 0 },
    "INVALID_ARGUMENT"
  );
  await rejected(
    "audio_track_route",
    { ...route, expectedRevision: 0, trackId: overlay.id },
    "INVALID_ARGUMENT"
  );
  await rejected(
    "audio_bus_set_route",
    { busId: "music", expectedRevision: 0, outputBusId: "music" },
    "INVALID_ARGUMENT"
  );
  await call(
    "audio_bus_set_route",
    { busId: "music", expectedRevision: 0, outputBusId: "sfx", projectId },
    writeResultSchema
  );
  const batched = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 1,
      operations: [
        {
          name: "Bus track",
          operation: "create_track",
          resultAlias: "bus_track",
          trackType: "audio",
        },
        {
          busId: "music",
          operation: "audio_track_route",
          scope: "root",
          trackId: "@bus_track",
        },
        {
          durationMs: 1000,
          height: 64,
          name: "Bus component",
          operation: "component_create",
          resultAlias: "card",
          tracks: [
            { id: "local", items: [], name: "Local audio", trackType: "audio" },
          ],
          width: 64,
        },
        {
          busId: "voiceover",
          operation: "audio_track_route",
          scope: "component:@card",
          trackId: "local",
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  const trackId = batched.aliases.bus_track;
  const componentId = batched.aliases.card;
  if (!(trackId && componentId)) {
    throw new Error("Routing aliases missing");
  }
  const routed = await read();
  expect(
    routed.project.tracks.find((track) => track.id === trackId)?.audioBusId
  ).toBe("music");
  expect(
    routed.project.components.find((component) => component.id === componentId)
      ?.tracks[0]?.audioBusId
  ).toBe("voiceover");
  expect(routed.project.audioBuses[1]?.outputBusId).toBe("sfx");
  expect(
    routed.project.tracks.find((track) => track.id === trackId)?.audioRole
  ).toBe("unassigned");
  await rejected(
    "timeline_batch_edit",
    {
      expectedRevision: 2,
      operations: [
        {
          busId: "voiceover",
          operation: "audio_track_route",
          scope: "root",
          trackId,
        },
        {
          busId: "sfx",
          operation: "audio_bus_set_route",
          outputBusId: "music",
        },
      ],
    },
    "INVALID_ARGUMENT"
  );
  await call(
    "track_update",
    { expectedRevision: 2, locked: true, projectId, trackId },
    writeResultSchema
  );
  await rejected(
    "audio_track_route",
    { busId: "sfx", expectedRevision: 3, scope: "root", trackId },
    "TRACK_LOCKED"
  );
  await call(
    "track_update",
    { expectedRevision: 3, locked: false, projectId, trackId },
    writeResultSchema
  );
  const beforeDraft = await read();
  const draft = await call(
    "draft_create",
    {
      expectedRevision: 4,
      operations: [
        {
          busId: "sfx",
          operation: "audio_track_route",
          scope: "root",
          trackId,
        },
      ],
      projectId,
    },
    editDraftSchema
  );
  expect(await read()).toEqual(beforeDraft);
  await rejected(
    "draft_commit",
    { draftId: draft.id, expectedRevision: 3 },
    "REVISION_CONFLICT"
  );
  await call(
    "draft_commit",
    { draftId: draft.id, expectedRevision: 4, projectId },
    writeResultSchema
  );
  expect(
    (await read()).project.tracks.find((track) => track.id === trackId)
      ?.audioBusId
  ).toBe("sfx");
  await call(
    "project_undo",
    { expectedRevision: 5, projectId },
    writeResultSchema
  );
  expect(
    (await read()).project.tracks.find((track) => track.id === trackId)
      ?.audioBusId
  ).toBe("music");
  await call(
    "project_redo",
    { expectedRevision: 6, projectId },
    writeResultSchema
  );
  expect(
    (await read()).project.tracks.find((track) => track.id === trackId)
      ?.audioBusId
  ).toBe("sfx");
  await call(
    "audio_track_route",
    { busId: null, expectedRevision: 7, projectId, scope: "root", trackId },
    writeResultSchema
  );
  const cleared = await read();
  const clearedTrack = cleared.project.tracks.find(
    (track) => track.id === trackId
  );
  expect(clearedTrack?.audioBusId).toBeUndefined();
  expect(clearedTrack?.audioRole).toBe("unassigned");
  const bytes = files();
  expect(await call("project_open", { projectId }, projectStateSchema)).toEqual(
    cleared
  );
  expect(await read()).toEqual(cleared);
  expect(files()).toEqual(bytes);
};
