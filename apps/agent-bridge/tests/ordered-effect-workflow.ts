import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import catalog from "../../../contracts/extended-visual-animation-v1.json";
import {
  editDraftSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;
export const verifyOrderedEffectWorkflow = async (
  client: Client,
  call: Call
) => {
  const f = catalog.orderedEffectCases;
  const { projectId } = await call(
    "project_create",
    {
      fps: f.project.fps,
      height: f.project.height,
      name: "Ordered-effect public lifecycle",
      width: f.project.width,
    },
    writeResultSchema
  );
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const initial = await read();
  const created = await call(
    "timeline_batch_edit",
    {
      expectedRevision: initial.project.revision,
      operations: [
        {
          durationMs: 800,
          operation: "add_shape",
          resultAlias: "leaf",
          startMs: 0,
          trackId: initial.project.tracks[1]?.id,
          ...f.source,
        },
        {
          effects: f.orders.shadeThenWash,
          itemId: "@leaf",
          operation: "update_item",
        },
        {
          effects: f.orders.washThenShade,
          itemId: "@leaf",
          operation: "update_item",
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  const itemId = created.aliases.leaf;
  if (!itemId) {
    throw new Error("ordered-effect creation alias missing");
  }
  const item = async () => {
    const leaf = (await read()).project.tracks[1]?.items.find(
      (value) => value.id === itemId
    );
    if (leaf?.type !== "shape") {
      throw new Error("eligible effect leaf missing");
    }
    return leaf;
  };
  expect((await item()).effects).toEqual(f.orders.washThenShade);
  const update = async (extra: Record<string, unknown>) => {
    const state = await read();
    return call(
      "timeline_update_item",
      { expectedRevision: state.project.revision, itemId, projectId, ...extra },
      writeResultSchema
    );
  };
  await update({ effects: f.orders.shadeThenWash });
  await update({ fill: f.source.fill });
  expect((await item()).effects).toEqual(f.orders.shadeThenWash);
  await update({ effects: [] });
  expect((await item()).effects ?? []).toEqual([]);
  await update({ effects: f.orders.shadeThenWash });
  const before = await read();
  const reject = async (
    name: string,
    extra: Record<string, unknown>,
    code?: string
  ) => {
    const response = await client.callTool({
      arguments: {
        expectedRevision: before.project.revision,
        projectId,
        ...extra,
      },
      name,
    });
    expect(response.isError).toBe(true);
    if (code) {
      expect(response.structuredContent).toMatchObject({
        error: { code, retryable: code === "REVISION_CONFLICT" },
      });
    } else {
      expect(response.content).toEqual(
        expect.arrayContaining([expect.objectContaining({ type: "text" })])
      );
    }
    expect((await read()).project).toEqual(before.project);
  };
  await f.invalidStacks.reduce(async (previous, { value }) => {
    await previous;
    await reject("timeline_update_item", { effects: value, itemId });
    await reject("timeline_batch_edit", {
      operations: [
        { effects: f.orders.washThenShade, itemId, operation: "update_item" },
        { effects: value, itemId, operation: "update_item" },
      ],
    });
    await reject("draft_create", {
      operations: [
        { effects: f.orders.washThenShade, itemId, operation: "update_item" },
        { effects: value, itemId, operation: "update_item" },
      ],
    });
  }, Promise.resolve());
  await reject(
    "timeline_update_item",
    { effects: [], itemId: "missing" },
    "ITEM_NOT_FOUND"
  );
  await reject(
    "timeline_update_item",
    { effects: [], expectedRevision: before.project.revision - 1, itemId },
    "REVISION_CONFLICT"
  );
  await reject(
    "timeline_batch_edit",
    {
      operations: [
        { effects: f.orders.washThenShade, itemId, operation: "update_item" },
        { itemId: "missing", operation: "delete_item" },
      ],
    },
    "ITEM_NOT_FOUND"
  );
  const draft = await call(
    "draft_create",
    {
      expectedRevision: before.project.revision,
      operations: [
        { effects: f.orders.washThenShade, itemId, operation: "update_item" },
      ],
      projectId,
    },
    editDraftSchema
  );
  const retained = await call(
    "draft_get",
    { draftId: draft.id, projectId },
    editDraftSchema
  );
  expect(retained.operations).toEqual([
    { effects: f.orders.washThenShade, itemId, operation: "update_item" },
  ]);
  expect((await read()).project).toEqual(before.project);
  await call(
    "draft_commit",
    { draftId: draft.id, expectedRevision: before.project.revision, projectId },
    writeResultSchema
  );
  expect((await item()).effects).toEqual(f.orders.washThenShade);
  const undoState = await read();
  await call(
    "project_undo",
    { expectedRevision: undoState.project.revision, projectId },
    writeResultSchema
  );
  expect((await item()).effects).toEqual(f.orders.shadeThenWash);
  const redoState = await read();
  await call(
    "project_redo",
    { expectedRevision: redoState.project.revision, projectId },
    writeResultSchema
  );
  expect((await item()).effects).toEqual(f.orders.washThenShade);
  const channelState = await read();
  await call(
    "timeline_set_animation_channels",
    {
      animationChannels: [
        {
          keyframes: [
            { curve: "hold", timeMs: 0, value: { type: "scalar", value: 0.5 } },
          ],
          property: "effect.vignette_amount",
          target: { id: "shade", kind: "effect", scope: "root" },
        },
      ],
      expectedRevision: channelState.project.revision,
      itemId,
      projectId,
    },
    writeResultSchema
  );
  const targeted = await read();
  const checkTarget = async (effects: unknown, code: string) => {
    const failed = await client.callTool({
      arguments: {
        effects,
        expectedRevision: targeted.project.revision,
        itemId,
        projectId,
      },
      name: "timeline_update_item",
    });
    expect(failed.isError).toBe(true);
    expect(failed.structuredContent).toMatchObject({
      error: { code, retryable: false },
    });
    expect((await read()).project).toEqual(targeted.project);
  };
  await checkTarget([], "ITEM_NOT_FOUND");
  await checkTarget(
    [{ id: "shade", radiusPx: 1, type: "gaussian_blur" }],
    "INVALID_ARGUMENT"
  );
};
