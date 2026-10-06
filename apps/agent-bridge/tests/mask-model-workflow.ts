import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import contract from "../../../contracts/mask-models-v1.json";
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

export const verifyMaskModelWorkflow = async (client: Client, call: Call) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.projectSchemaVersion).toBe(34);
  expect(status.subsystems.editor.capabilities).toContain(contract.capability);
  expect(status.subsystems.rendering.capabilities).not.toContain(
    contract.capability
  );
  const { projectId } = await call(
    "project_create",
    { name: "Mask metadata lifecycle" },
    writeResultSchema
  );
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const initial = await read();
  expect(initial.project.schemaVersion).toBe(34);
  const trackId = initial.project.tracks[1]?.id;
  const masks = contract.stackCases[1]?.value?.map((value) =>
    maskSchema.parse(value)
  );
  if (!(trackId && masks)) {
    throw new Error("canonical mask lifecycle input missing");
  }
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
        operation: "add_solid_color",
        resultAlias: "leaf",
        startMs: 0,
        trackId,
        transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
      },
      { itemId: "@leaf", masks, operation: "update_item" },
    ],
  });
  const itemId = created.aliases.leaf;
  const item = async () =>
    (await read()).project.tracks[1]?.items.find(
      (value) => value.id === itemId
    );
  expect((await item())?.masks).toEqual(masks);
  await write("timeline_update_item", 1, {
    itemId,
    masks: [...masks].reverse(),
  });
  expect((await item())?.masks).toEqual([...masks].reverse());
  await write("timeline_update_item", 2, { color: "#123456", itemId });
  expect((await item())?.masks).toEqual([...masks].reverse());
  const before = await read();
  for (const [expectedRevision, input, code] of [
    [2, { itemId, masks: [] }, "REVISION_CONFLICT"],
    [3, { itemId: "missing", masks }, "ITEM_NOT_FOUND"],
    [3, { itemId, masks: [masks[0], masks[0]] }, "INVALID_ARGUMENT"],
  ] as const) {
    // biome-ignore lint/performance/noAwaitInLoops: verify persisted state after each independently rejected mutation.
    const failed = await client.callTool({
      arguments: { expectedRevision, projectId, ...input },
      name: "timeline_update_item",
    });
    expect(failed.isError).toBe(true);
    expect(failed.structuredContent).toMatchObject({ error: { code } });
    expect(await read()).toEqual(before);
  }
  const failed = await client.callTool({
    arguments: {
      expectedRevision: 3,
      operations: [
        { itemId, masks: [], operation: "update_item" },
        { itemId: "missing", operation: "delete_item" },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(failed.structuredContent).toMatchObject({
    error: { code: "ITEM_NOT_FOUND" },
  });
  expect(await read()).toEqual(before);
  const malformed = await client.callTool({
    arguments: { expectedRevision: 3, itemId, masks: null, projectId },
    name: "timeline_update_item",
  });
  expect(malformed.isError).toBe(true);
  expect(await read()).toEqual(before);
  await write("timeline_update_item", 3, { itemId, masks: [] });
  expect(await item()).not.toHaveProperty("masks");
  await write("project_undo", 4, {});
  expect((await item())?.masks).toEqual([...masks].reverse());
  await write("project_redo", 5, {});
  expect(await item()).not.toHaveProperty("masks");
  const draft = await call(
    "draft_create",
    {
      expectedRevision: 6,
      operations: [{ itemId, masks, operation: "update_item" }],
      projectId,
    },
    editDraftSchema
  );
  expect(draft.operations[0]).toMatchObject({ masks });
  expect((await read()).project.revision).toBe(6);
  const updated = await call(
    "draft_update",
    {
      draftId: draft.id,
      expectedRevision: 6,
      operations: [
        { itemId, masks: [...masks].reverse(), operation: "update_item" },
      ],
      projectId,
    },
    editDraftSchema
  );
  expect(updated.operations[0]).toMatchObject({ masks: [...masks].reverse() });
  await write("draft_commit", 6, { draftId: draft.id });
  expect((await item())?.masks).toEqual([...masks].reverse());
  expect((await read()).project.revision).toBe(7);
};
