import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import models from "../../../contracts/mask-models-v1.json";
import rendering from "../../../contracts/mask-rendering-v1.json";
import {
  editDraftSchema,
  maskSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;
export const verifyMaskRenderingWorkflow = async (
  client: Client,
  call: Call
) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.projectSchemaVersion).toBe(35);
  expect(status.subsystems.editor.capabilities).toContain("mask_animation_v1");
  expect(status.subsystems.editor.capabilities).not.toContain(
    "mask_rendering_v1"
  );
  expect(
    status.subsystems.rendering.capabilities.includes("mask_rendering_v1")
  ).toBe(status.subsystems.rendering.ready);
  const { projectId } = await call(
    "project_create",
    { fps: 10, height: 64, name: "Animated mask lifecycle", width: 64 },
    writeResultSchema
  );
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const base = await read();
  const trackId = base.project.tracks[1]?.id;
  if (!trackId) {
    throw new Error("mask workflow track missing");
  }
  const mask = maskSchema.parse(models.cases[0]?.value);
  mask.id = "@paint";
  mask.source.path = {
    commands: [
      { to: { x: 0, y: 0 }, type: "moveTo" },
      { to: { x: 4, y: 0 }, type: "lineTo" },
      { to: { x: 4, y: 4 }, type: "lineTo" },
      { to: { x: 0, y: 4 }, type: "lineTo" },
      { type: "close" },
    ],
    fillRule: "evenodd",
  };
  mask.transform = {
    anchor: { x: 0, y: 0 },
    opacity: 1,
    position: { unit: "pixels", x: 0, y: 0 },
    rotationDeg: 0,
    scaleX: 1,
    scaleY: 1,
    skewXDeg: 0,
    skewYDeg: 0,
  };
  mask.featherPx = 0;
  mask.expansionPx = 0;
  const gradient = maskSchema.parse({
    ...mask,
    id: "gradient",
    source: {
      ...mask.source,
      paint: {
        end: { x: 4, y: 0 },
        start: { x: 0, y: 0 },
        stops: [
          { color: { a: 1, b: 0, g: 0, r: 1 }, offset: 0 },
          { color: { a: 1, b: 1, g: 0, r: 0 }, offset: 1 },
        ],
        type: "linearGradient",
      },
    },
  });
  const animationChannels = rendering.channelCases.map(({ channel }) => ({
    ...channel,
    target: {
      ...channel.target,
      id: channel.property === "mask.gradient_stops" ? gradient.id : mask.id,
    },
  }));
  const write = (
    name: string,
    expectedRevision: number,
    input: Record<string, unknown>
  ) => call(name, { expectedRevision, projectId, ...input }, writeResultSchema);
  const created = await write("timeline_batch_edit", 0, {
    operations: [
      {
        color: "#ff0000",
        durationMs: 1000,
        height: 64,
        operation: "add_rectangle",
        resultAlias: "leaf",
        startMs: 0,
        trackId,
        transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
        width: 64,
      },
      { itemId: "@leaf", masks: [mask, gradient], operation: "update_item" },
      {
        animationChannels,
        itemId: "@leaf",
        operation: "set_animation_channels",
      },
    ],
  });
  const itemId = created.aliases.leaf;
  if (!itemId) {
    throw new Error("mask workflow alias missing");
  }
  const item = async () =>
    (await read()).project.tracks[1]?.items.find((x) => x.id === itemId);
  expect((await item())?.animationChannels).toEqual(animationChannels);
  await write("timeline_update_item", 1, { itemId, masks: [gradient, mask] });
  expect((await item())?.animationChannels).toEqual(animationChannels);
  const before = await read();
  const reject = async (input: Record<string, unknown>, code: string) => {
    const error = await client.callTool({
      arguments: { expectedRevision: 2, projectId, ...input },
      name: "timeline_set_animation_channels",
    });
    expect(error.isError).toBe(true);
    expect(error.structuredContent).toMatchObject({
      error: { code, retryable: false },
    });
    expect(await read()).toEqual(before);
  };
  await reject(
    {
      animationChannels: [
        {
          ...animationChannels[0],
          target: { id: "missing", kind: "mask", scope: "root" },
        },
      ],
      itemId,
    },
    "ITEM_NOT_FOUND"
  );
  await reject(
    { animationChannels: [animationChannels[0], animationChannels[0]], itemId },
    "INVALID_ARGUMENT"
  );
  const dangling = await client.callTool({
    arguments: { expectedRevision: 2, itemId, masks: [], projectId },
    name: "timeline_update_item",
  });
  expect(dangling.isError).toBe(true);
  expect(dangling.structuredContent).toMatchObject({
    error: { code: "ITEM_NOT_FOUND", retryable: false },
  });
  expect(await read()).toEqual(before);
  const sameProperty = [
    ...animationChannels,
    {
      ...animationChannels.at(-1),
      target: { id: gradient.id, kind: "mask", scope: "root" },
    },
  ];
  await write("timeline_set_animation_channels", 2, {
    animationChannels: sameProperty,
    itemId,
  });
  expect((await item())?.animationChannels).toEqual(sameProperty);
  const draft = await call(
    "draft_create",
    {
      expectedRevision: 3,
      operations: [
        { animationChannels: [], itemId, operation: "set_animation_channels" },
      ],
      projectId,
    },
    editDraftSchema
  );
  expect(draft.operations).toEqual([
    { animationChannels: [], itemId, operation: "set_animation_channels" },
  ]);
  await write("draft_commit", 3, { draftId: draft.id });
  expect(await item()).not.toHaveProperty("animationChannels");
  await write("project_undo", 4, {});
  expect((await item())?.animationChannels).toEqual(sameProperty);
  await write("project_redo", 5, {});
  expect(await item()).not.toHaveProperty("animationChannels");
  await write("timeline_update_item", 6, { itemId, masks: [] });
  expect(await item()).not.toHaveProperty("masks");
};
