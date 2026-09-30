import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import GROUPS from "../../../contracts/group-parent-v1.json";

import {
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;

export const verifyGroupWorkflow = async (client: Client, call: Call) => {
  const { projectId } = await call(
    "project_create",
    { name: "Ungroup smoke" },
    writeResultSchema
  );
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const initial = await read();
  const trackId = initial.project.tracks[1]?.id;
  const edit = (
    name: string,
    expectedRevision: number,
    input: Record<string, unknown>
  ) => call(name, { expectedRevision, projectId, ...input }, writeResultSchema);
  const created = await edit("add_group", 0, {
    durationMs: 1000,
    startMs: 0,
    trackId,
  });
  const [groupId] = created.changedIds;
  const visual = await edit("timeline_add_rectangle", 1, {
    color: "#ff0000",
    durationMs: 1000,
    height: 10,
    startMs: 0,
    trackId,
    transform: { opacity: 1, positionX: 7, positionY: 9, scale: 1 },
    width: 20,
  });
  const [itemId] = visual.changedIds;
  await edit("item_set_parent", 2, {
    itemId,
    parent: { id: groupId, scope: "root" },
  });
  await edit("item_set_z_index", 3, { itemId, zIndex: -7 });
  const before = await read();
  await Promise.all(
    (
      [
        [0, groupId, "REVISION_CONFLICT"],
        ...GROUPS.ungroupFailures.map(
          (fixture) =>
            [
              4,
              fixture.groupId === "child" ? itemId : fixture.groupId,
              fixture.error,
            ] as const
        ),
      ] as const
    ).map(async ([expectedRevision, target, code]) => {
      const failed = await client.callTool({
        arguments: { expectedRevision, groupId: target, projectId },
        name: "group_ungroup",
      });
      expect(failed.isError).toBe(true);
      expect(failed.structuredContent).toMatchObject({
        error: { code, retryable: code === "REVISION_CONFLICT" },
      });
    })
  );
  const failed = await client.callTool({
    arguments: {
      expectedRevision: 4,
      operations: [
        { groupId, operation: "group_ungroup" },
        { groupId, operation: "group_ungroup" },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(failed.isError).toBe(true);
  expect(failed.structuredContent).toMatchObject({
    error: { code: "ITEM_NOT_FOUND" },
  });
  expect(await read()).toEqual(before);
  const malformedBatch = await client.callTool({
    arguments: {
      expectedRevision: 4,
      operations: [
        { itemId, operation: "item_set_z_index", zIndex: 99 },
        { groupId, operation: "group_ungroup", resultAlias: null },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(malformedBatch.isError).toBe(true);
  expect(await read()).toEqual(before);
  await edit("track_update", 4, { locked: true, trackId });
  const locked = await client.callTool({
    arguments: { expectedRevision: 5, groupId, projectId },
    name: "group_ungroup",
  });
  expect(locked.structuredContent).toMatchObject({
    error: { code: "TRACK_LOCKED" },
  });
  await edit("track_update", 5, { locked: false, trackId });
  const removed = await edit("group_ungroup", 6, { groupId });
  expect(removed.changedIds).toEqual([groupId, itemId]);
  const after = await read();
  expect(after.project.tracks[1]?.items).toHaveLength(1);
  expect(after.project.tracks[1]?.items[0]?.parent).toBeUndefined();
  expect(after.project.tracks[1]?.items[0]?.zIndex).toBe(-7);
  await edit("project_undo", 7, {});
  expect((await read()).project.tracks).toEqual(before.project.tracks);
  await edit("project_redo", 8, {});
  expect((await read()).project.tracks).toEqual(after.project.tracks);
  const operations = [
    {
      durationMs: 1000,
      operation: "add_group",
      resultAlias: "g",
      startMs: 0,
      trackId,
    },
    {
      itemId,
      operation: "item_set_parent",
      parent: { id: "@g", scope: "root" },
    },
    { itemId: "@g", operation: "item_set_z_index", zIndex: 5 },
    { groupId: "@g", operation: "group_ungroup" },
  ];
  const aliased = await edit("timeline_batch_edit", 9, { operations });
  expect(aliased.revision).toBe(10);
  expect(aliased.aliases.g).toBeTypeOf("string");
  expect((await read()).project.tracks).toEqual(after.project.tracks);
  const aliasFailure = await client.callTool({
    arguments: {
      expectedRevision: 10,
      operations: [
        ...operations,
        { groupId: "@g", operation: "group_ungroup" },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(aliasFailure.structuredContent).toMatchObject({
    error: { code: "ITEM_NOT_FOUND" },
  });
  expect((await read()).project.revision).toBe(10);
  const malformed = await client.callTool({
    arguments: { expectedRevision: 10, groupId: 42, projectId },
    name: "group_ungroup",
  });
  expect(malformed.isError).toBe(true);
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.capabilities).toContain(GROUPS.ungroupCapability);
  const tools = await client.listTools();
  expect(
    tools.tools.find((tool) => tool.name === "group_ungroup")?.annotations
      ?.destructiveHint
  ).toBe(true);
  const timing = await edit("timeline_batch_edit", 10, {
    operations: [
      {
        durationMs: 1000,
        operation: "add_group",
        resultAlias: "timed",
        staggerMs: 125,
        startMs: 0,
        trackId,
      },
      { itemId: "@timed", operation: "update_item", transform2d: null },
      {
        animationChannels: [
          {
            keyframes: [
              {
                curve: "linear",
                timeMs: 0,
                value: { type: "scalar", value: 1 },
              },
              {
                curve: "hold",
                timeMs: 500,
                value: { type: "scalar", value: 0.5 },
              },
            ],
            property: "transform.opacity",
          },
        ],
        itemId: "@timed",
        operation: "set_animation_channels",
      },
    ],
  });
  expect(timing.revision).toBe(11);
  const timedState = await read();
  expect(
    timedState.project.tracks[1]?.items.find(
      (item) => item.id === timing.aliases.timed
    )
  ).toMatchObject({
    animationChannels: [{ property: "transform.opacity" }],
    staggerMs: 125,
  });
  const timingFailure = await client.callTool({
    arguments: {
      expectedRevision: 11,
      operations: [
        {
          itemId: timing.aliases.timed,
          operation: "update_item",
          staggerMs: 250,
        },
        { itemId: "missing", operation: "update_item", staggerMs: 5 },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(timingFailure.structuredContent).toMatchObject({
    error: { code: "ITEM_NOT_FOUND" },
  });
  expect(await read()).toEqual(timedState);
  const boundsFailure = await client.callTool({
    arguments: {
      expectedRevision: 11,
      operations: [
        {
          color: "#ff0000",
          durationMs: 1000,
          height: 10,
          operation: "add_rectangle",
          resultAlias: "wide",
          startMs: 0,
          trackId,
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
          width: 500,
        },
        {
          itemId: "@wide",
          operation: "item_set_parent",
          parent: { id: timing.aliases.timed, scope: "root" },
        },
        {
          animationChannels: [
            {
              keyframes: [
                {
                  curve: "linear",
                  timeMs: 0,
                  value: { type: "scalar", value: 1 },
                },
                {
                  curve: "hold",
                  timeMs: 900,
                  value: { type: "scalar", value: 100 },
                },
              ],
              property: "transform.scale_x",
            },
          ],
          itemId: timing.aliases.timed,
          operation: "set_animation_channels",
        },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(boundsFailure.structuredContent).toMatchObject({
    error: { code: "INVALID_ARGUMENT", retryable: false },
  });
  expect(await read()).toEqual(timedState);
  expect(status.capabilities).toContain("inherited_animation_timing_v1");
};
