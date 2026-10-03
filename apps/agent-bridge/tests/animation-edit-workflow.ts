import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import CONTRACT from "../../../contracts/animation-channels-v1.json";
import { projectStateSchema, writeResultSchema } from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;

export const verifyAnimationEditWorkflow = async (
  client: Client,
  call: Call
) => {
  const { projectId } = await call(
    "project_create",
    { name: "Animation edit preservation" },
    writeResultSchema
  );
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const edit = (
    name: string,
    expectedRevision: number,
    input: Record<string, unknown>
  ) => call(name, { expectedRevision, projectId, ...input }, writeResultSchema);
  const trackId = (await read()).project.tracks[1]?.id;
  const source = CONTRACT.examples.validBezier;
  const created = await edit("timeline_batch_edit", 0, {
    operations: [
      {
        color: "#ffffff",
        durationMs: 1000,
        height: 32,
        operation: "add_rectangle",
        resultAlias: "seed",
        startMs: 0,
        trackId,
        transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
        width: 32,
      },
      {
        animationChannels: [source],
        itemId: "@seed",
        operation: "set_animation_channels",
      },
      { itemId: "@seed", operation: "split_item", splitMs: 175 },
    ],
  });
  const first = await read();
  const items = first.project.tracks[1]?.items ?? [];
  expect(items).toHaveLength(2);
  const left = items.find((item) => item.id === created.aliases.seed);
  const right = items.find((item) => item.startMs === 175);
  expect(left?.animationChannels).toEqual([
    { ...source, clock: { offsetMs: 0, sourceDurationMs: 1000 } },
  ]);
  expect(right?.animationChannels).toEqual([
    { ...source, clock: { offsetMs: 175, sourceDurationMs: 1000 } },
  ]);
  const rightId = right?.id;
  expect(rightId).toBeDefined();
  await edit("timeline_trim_item", 1, {
    durationMs: 500,
    itemId: rightId,
    startMs: 250,
  });
  const trimmed = await read();
  expect(
    trimmed.project.tracks[1]?.items.find((item) => item.id === rightId)
      ?.animationChannels
  ).toEqual([{ ...source, clock: { offsetMs: 250, sourceDurationMs: 1000 } }]);
  await edit("timeline_duplicate_items", 2, {
    itemIds: [rightId],
    offsetMs: 1000,
  });
  const duplicated = await read();
  const copy = duplicated.project.tracks[1]?.items.find(
    (item) => item.startMs === 1250
  );
  expect(copy?.animationChannels).toEqual([
    { ...source, clock: { offsetMs: 250, sourceDurationMs: 1000 } },
  ]);
  const rejected = await client.callTool({
    arguments: {
      expectedRevision: 3,
      operations: [
        {
          color: "#ffffff",
          durationMs: 1000,
          height: 32,
          operation: "add_rectangle",
          resultAlias: "discarded",
          startMs: 2000,
          trackId,
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
          width: 32,
        },
        {
          animationChannels: [source],
          itemId: "@discarded",
          operation: "set_animation_channels",
        },
        { itemId: "@discarded", operation: "split_item", splitMs: 2000 },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(rejected.isError).toBe(true);
  expect(rejected.structuredContent).toMatchObject({
    error: { code: "VALIDATION_FAILED", retryable: false },
  });
  expect(await read()).toEqual(duplicated);
  await edit("project_undo", 3, {});
  expect((await read()).project.tracks[1]?.items).toHaveLength(2);
  await edit("project_redo", 4, {});
  const reopened = await read();
  expect(reopened.project.tracks[1]?.items).toEqual(
    duplicated.project.tracks[1]?.items
  );
  expect(reopened.project.schemaVersion).toBe(31);
};
