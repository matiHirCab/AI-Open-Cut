import { writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import CATALOG from "../../../contracts/repeaters-v1.json";
import {
  jobSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;

export async function verifyRepeaterWorkflow(
  client: Client,
  call: Call,
  mediaDirectory: string
) {
  const [fixture] = CATALOG.valid;
  if (!fixture) {
    throw new Error("Repeater fixtures are missing");
  }
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.capabilities).toContain("repeater_items");
  expect(status.capabilities).toContain("repeater_rendering");
  const created = await call(
    "project_create",
    { name: "Repeater smoke" },
    writeResultSchema
  );
  const { projectId } = created;
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const trackId = (await read()).project.tracks.find(
    (track) => track.trackType === "overlay"
  )?.id;
  expect(trackId).toBeDefined();
  const aliasedDescriptor = structuredClone(fixture.repeater);
  aliasedDescriptor.source.id = "@source";
  let result = await call(
    "timeline_batch_edit",
    {
      expectedRevision: created.revision,
      operations: [
        {
          durationMs: 1000,
          fill: { color: { a: 1, b: 0, g: 0, r: 1 }, type: "solid" },
          geometry: { height: 20, type: "rectangle", width: 20 },
          operation: "add_shape",
          resultAlias: "source",
          startMs: 0,
          stroke: null,
          trackId,
        },
        {
          durationMs: 800,
          operation: "add_repeater",
          repeater: aliasedDescriptor,
          resultAlias: "copies",
          startMs: 100,
          trackId,
        },
        { itemId: "@copies", operation: "item_set_z_index", zIndex: 3 },
        {
          durationMs: 1000,
          fill: { color: { a: 1, b: 0, g: 1, r: 0 }, type: "solid" },
          geometry: { height: 10, type: "rectangle", width: 10 },
          operation: "add_shape",
          resultAlias: "replacement",
          startMs: 0,
          stroke: null,
          trackId,
        },
        {
          itemId: "@copies",
          operation: "update_item",
          repeater: {
            ...aliasedDescriptor,
            source: { id: "@replacement", scope: "root" },
          },
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  expect(result.revision).toBe(created.revision + 1);
  const sourceId = result.aliases.replacement;
  const repeaterId = result.aliases.copies;
  if (!(sourceId && repeaterId)) {
    throw new Error("Repeater aliases were not returned");
  }
  expect(
    (await read()).project.tracks.flatMap((track) => track.items)
  ).toContainEqual(
    expect.objectContaining({
      id: repeaterId,
      repeater: expect.objectContaining({
        source: { id: sourceId, scope: "root" },
      }),
    })
  );
  const descriptor = structuredClone(fixture.repeater);
  descriptor.opacityOffset = -0.1;
  descriptor.source = { id: sourceId, scope: "root" };
  result = await call(
    "timeline_update_item",
    {
      expectedRevision: result.revision,
      itemId: repeaterId,
      projectId,
      repeater: descriptor,
    },
    writeResultSchema
  );
  const beforeFailure = await read();
  await Promise.all(
    [false, true].map(async (forward) => {
      const operations: Record<string, unknown>[] = [
        {
          itemId: repeaterId,
          operation: "update_item",
          repeater: { ...descriptor, source: { id: "@later", scope: "root" } },
        },
      ];
      if (forward) {
        operations.push({
          durationMs: 1000,
          fill: { color: { a: 1, b: 0, g: 0, r: 1 }, type: "solid" },
          geometry: { height: 20, type: "rectangle", width: 20 },
          operation: "add_shape",
          resultAlias: "later",
          startMs: 0,
          stroke: null,
          trackId,
        });
      }
      const rejected = await client.callTool({
        arguments: { expectedRevision: result.revision, operations, projectId },
        name: "timeline_batch_edit",
      });
      expect(rejected.isError).toBe(true);
      expect(rejected.structuredContent).toMatchObject({
        error: { code: "VALIDATION_FAILED" },
      });
    })
  );
  expect(await read()).toEqual(beforeFailure);
  const failed = await client.callTool({
    arguments: {
      expectedRevision: result.revision,
      operations: [
        {
          durationMs: 1000,
          operation: "add_repeater",
          repeater: {
            ...fixture.repeater,
            source: { id: sourceId, scope: "root" },
          },
          resultAlias: "newCopies",
          startMs: 0,
          trackId,
        },
        {
          itemId: "@newCopies",
          operation: "update_item",
          repeater: descriptor,
        },
        { itemId: "missing", operation: "delete_item" },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(failed.isError).toBe(true);
  expect(failed.structuredContent).toMatchObject({
    error: { code: "ITEM_NOT_FOUND" },
  });
  expect(await read()).toEqual(beforeFailure);
  const after = await read();
  result = await call(
    "project_undo",
    { expectedRevision: result.revision, projectId },
    writeResultSchema
  );
  result = await call(
    "project_redo",
    { expectedRevision: result.revision, projectId },
    writeResultSchema
  );
  expect((await read()).project.tracks).toEqual(after.project.tracks);
  const frame = await call(
    "preview_render_frame",
    { expectedRevision: result.revision, projectId, timeMs: 250 },
    jobSchema
  );
  await expect
    .poll(() => call("job_get_status", { jobId: frame.jobId }, jobSchema))
    .toMatchObject({ status: "completed" });
  await verifyEffectiveAudio(client, call, mediaDirectory);
}

async function verifyEffectiveAudio(
  client: Client,
  call: Call,
  mediaDirectory: string
) {
  const created = await call(
    "project_create",
    { name: "Effective repeater audio" },
    writeResultSchema
  );
  const { projectId } = created;
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const trackId = (await read()).project.tracks.find(
    (track) => track.trackType === "overlay"
  )?.id;
  const silentPath = join(mediaDirectory, "repeater-silent.mp4");
  const audiblePath = join(mediaDirectory, "repeater-audible.mp4");
  writeFileSync(silentPath, "silent fixture");
  writeFileSync(audiblePath, "audible fixture");
  const silent = await call(
    "asset_import",
    { expectedRevision: 0, mediaType: "video", path: silentPath, projectId },
    writeResultSchema
  );
  const audible = await call(
    "asset_import",
    { expectedRevision: 1, mediaType: "video", path: audiblePath, projectId },
    writeResultSchema
  );
  const [silentId] = silent.changedIds;
  const [audibleId] = audible.changedIds;
  const descriptor = {
    ...CATALOG.valid[0]?.repeater,
    source: { id: "@instance", scope: "root" },
  };
  const added = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 2,
      operations: [
        {
          durationMs: 100,
          height: 100,
          name: "Leaf",
          operation: "component_create",
          resultAlias: "leaf",
          slots: [
            {
              binding: { property: "media.asset", targetLayerId: "media" },
              constraints: {},
              id: "asset",
              kind: "asset",
              name: "Asset",
              required: false,
            },
          ],
          tracks: [
            {
              id: "local",
              items: [
                {
                  assetId: silentId,
                  audio: { fadeInMs: 0, fadeOutMs: 0, muted: false, volume: 1 },
                  durationMs: 100,
                  id: "media",
                  keyframes: [],
                  sourceInMs: 0,
                  startMs: 0,
                  type: "media",
                },
              ],
              name: "Local",
              trackType: "overlay",
            },
          ],
          width: 100,
        },
        {
          componentId: "@leaf",
          durationMs: 100,
          operation: "add_component_instance",
          resultAlias: "instance",
          startMs: 0,
          timeScale: 1,
          trackId,
          trimStartMs: 0,
        },
        {
          durationMs: 100,
          operation: "add_repeater",
          repeater: descriptor,
          startMs: 0,
          trackId,
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  const before = await read();
  const update = {
    componentId: added.aliases.leaf,
    durationMs: 100,
    itemId: added.aliases.instance,
    operation: "component_instance_update",
    slotValues: {
      asset: {
        type: "asset",
        value: { id: audibleId, kind: "asset", scope: "project" },
      },
    },
    startMs: 0,
    timeScale: 1,
    trimStartMs: 0,
  };
  await Promise.all(
    ["timeline_batch_edit", "draft_create"].map(async (name) => {
      const failure = await client.callTool({
        arguments: { expectedRevision: 3, operations: [update], projectId },
        name,
      });
      expect(failure.isError).toBe(true);
      expect(failure.structuredContent).toMatchObject({
        error: { code: "INVALID_ARGUMENT" },
      });
    })
  );
  expect(await read()).toEqual(before);
  const undone = await call(
    "project_undo",
    { expectedRevision: 3, projectId },
    writeResultSchema
  );
  const redone = await call(
    "project_redo",
    { expectedRevision: undone.revision, projectId },
    writeResultSchema
  );
  expect((await read()).project.tracks).toEqual(before.project.tracks);
  const frame = await call(
    "preview_render_frame",
    { expectedRevision: redone.revision, projectId, timeMs: 50 },
    jobSchema
  );
  await expect
    .poll(() => call("job_get_status", { jobId: frame.jobId }, jobSchema))
    .toMatchObject({ status: "completed" });
}
