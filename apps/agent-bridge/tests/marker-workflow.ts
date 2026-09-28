import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";

import { projectStateSchema, writeResultSchema } from "../src/schemas";

type Call = <Output>(
  name: string,
  arguments_: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;

export const verifyMarkerWorkflow = async (client: Client, call: Call) => {
  const names = (await client.listTools()).tools.map((tool) => tool.name);
  for (const name of [
    "marker_create",
    "marker_update",
    "marker_delete",
    "set_item_start_time",
  ]) {
    expect(names).toContain(name);
  }

  const { projectId } = await call(
    "project_create",
    { name: "Marker timing" },
    writeResultSchema
  );
  const read = async () =>
    await call("project_get_state", { projectId }, projectStateSchema);
  const trackId = (await read()).project.tracks[1]?.id;
  if (!trackId) {
    throw new Error("overlay track missing");
  }
  const added = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 0,
      operations: [
        {
          color: "#ff0000",
          durationMs: 100,
          height: 20,
          operation: "add_rectangle",
          startMs: 0,
          trackId,
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
          width: 20,
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  const [itemId] = added.changedIds;
  const item = async () =>
    (await read()).project.tracks
      .flatMap((track) => track.items)
      .find((candidate) => candidate.id === itemId);
  expect(await item()).toMatchObject({ startMs: 0 });
  expect((await item())?.startTime).toBeUndefined();

  const marker = await call(
    "marker_create",
    {
      expectedRevision: 1,
      kind: "cue",
      name: "beat",
      projectId,
      scope: "root",
      timeMs: 300,
    },
    writeResultSchema
  );
  const [markerId] = marker.changedIds;
  expect((await read()).project.markers[0]).toMatchObject({
    id: markerId,
    name: "beat",
  });
  await expect(
    call(
      "set_item_start_time",
      {
        expectedRevision: 2,
        itemId,
        projectId,
        scope: "root",
        time: { markerName: "absent", offsetMs: 0, type: "marker" },
      },
      writeResultSchema
    )
  ).rejects.toThrow("ITEM_NOT_FOUND");
  expect((await read()).project.revision).toBe(2);

  await call(
    "set_item_start_time",
    {
      expectedRevision: 2,
      itemId,
      projectId,
      scope: "root",
      time: { markerName: "beat", offsetMs: -50, type: "marker" },
    },
    writeResultSchema
  );
  expect(await item()).toMatchObject({
    startMs: 250,
    startTime: { markerName: "beat", offsetMs: -50, type: "marker" },
  });
  await call(
    "marker_update",
    {
      expectedRevision: 3,
      kind: "cue",
      markerId,
      name: "beat",
      projectId,
      scope: "root",
      timeMs: 500,
    },
    writeResultSchema
  );
  expect(await item()).toMatchObject({ startMs: 450 });
  await expect(
    call(
      "marker_delete",
      { expectedRevision: 4, markerId, projectId, scope: "root" },
      writeResultSchema
    )
  ).rejects.toThrow("ITEM_NOT_FOUND");
  await expect(
    call(
      "marker_update",
      {
        expectedRevision: 3,
        kind: "cue",
        markerId,
        name: "beat",
        projectId,
        scope: "root",
        timeMs: 600,
      },
      writeResultSchema
    )
  ).rejects.toThrow("REVISION_CONFLICT");
  await expect(
    call(
      "timeline_batch_edit",
      {
        expectedRevision: 4,
        operations: [
          {
            kind: "cue",
            name: "beat",
            operation: "marker_create",
            scope: "root",
            timeMs: 600,
          },
        ],
        projectId,
      },
      writeResultSchema
    )
  ).rejects.toThrow("INVALID_ARGUMENT");
  expect((await read()).project.markers).toHaveLength(1);
  expect((await read()).project.revision).toBe(4);

  await call(
    "set_item_start_time",
    {
      expectedRevision: 4,
      itemId,
      projectId,
      scope: "root",
      time: { type: "milliseconds", valueMs: 100 },
    },
    writeResultSchema
  );
  expect(await item()).toMatchObject({ startMs: 100 });
  expect((await item())?.startTime).toBeUndefined();
  await call(
    "marker_create",
    {
      expectedRevision: 5,
      kind: "cue",
      name: "beat",
      projectId,
      scope: "root",
      timeMs: 700,
    },
    writeResultSchema
  );
  await expect(
    call(
      "set_item_start_time",
      {
        expectedRevision: 6,
        itemId,
        projectId,
        scope: "root",
        time: { markerName: "beat", offsetMs: 0, type: "marker" },
      },
      writeResultSchema
    )
  ).rejects.toThrow("INVALID_ARGUMENT");
};
