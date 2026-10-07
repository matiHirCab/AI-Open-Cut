import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import contract from "../../../contracts/track-mattes-v1.json";
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
export const verifyTrackMatteWorkflow = async (client: Client, call: Call) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.projectSchemaVersion).toBe(37);
  expect(status.subsystems.editor.capabilities).toContain(
    contract.capabilities.editor
  );
  expect(status.subsystems.editor.capabilities).not.toContain(
    contract.capabilities.renderer
  );
  expect(
    status.subsystems.rendering.capabilities.includes(
      contract.capabilities.renderer
    )
  ).toBe(status.subsystems.rendering.ready);
  const { projectId } = await call(
    "project_create",
    { fps: 10, height: 64, name: "Scoped matte lifecycle", width: 64 },
    writeResultSchema
  );
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const trackId = (await read()).project.tracks[1]?.id;
  if (!trackId) {
    throw new Error("matte workflow track missing");
  }
  const write = (
    name: string,
    expectedRevision: number,
    input: Record<string, unknown>
  ) => call(name, { expectedRevision, projectId, ...input }, writeResultSchema);
  const add = (alias: string, color: string) => ({
    color,
    durationMs: 1000,
    height: 64,
    operation: "add_rectangle",
    resultAlias: alias,
    startMs: 0,
    trackId,
    transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
    width: 64,
  });
  const created = await write("timeline_batch_edit", 0, {
    operations: [
      add("provider", "#ff0000"),
      add("recipient", "#0000ff"),
      { itemId: "@provider", matteOnly: true, operation: "update_item" },
      {
        itemId: "@recipient",
        matte: { channel: "alpha", sourceId: "@provider" },
        operation: "update_item",
      },
    ],
  });
  const providerId = created.aliases.provider,
    recipientId = created.aliases.recipient;
  if (!(providerId && recipientId)) {
    throw new Error("matte aliases missing");
  }
  const item = async (id: string) =>
    (await read()).project.tracks[1]?.items.find((entry) => entry.id === id);
  expect(await item(recipientId)).toMatchObject({
    matte: { channel: "alpha", sourceId: providerId },
  });
  expect(await item(providerId)).toHaveProperty("matteOnly", true);
  await write("timeline_update_item", 1, {
    itemId: recipientId,
    matte: { channel: "luma", sourceId: providerId },
  });
  await write("timeline_update_item", 2, {
    color: "#00ff00",
    itemId: recipientId,
  });
  expect(await item(recipientId)).toHaveProperty("matte", {
    channel: "luma",
    sourceId: providerId,
  });
  await write("item_reorder", 3, { index: 1, itemId: providerId });
  expect(await item(recipientId)).toHaveProperty("matte", {
    channel: "luma",
    sourceId: providerId,
  });
  const before = await read();
  const reject = async (
    name: string,
    input: Record<string, unknown>,
    code: string,
    retryable = false
  ) => {
    const result = await client.callTool({
      arguments: { expectedRevision: 4, projectId, ...input },
      name,
    });
    expect(result.isError).toBe(true);
    expect(result.structuredContent).toMatchObject({
      error: { code, retryable },
    });
    expect(await read()).toEqual(before);
  };
  await reject(
    "timeline_update_item",
    { itemId: recipientId, matte: { channel: "alpha", sourceId: "missing" } },
    "ITEM_NOT_FOUND"
  );
  await reject(
    "timeline_update_item",
    { itemId: providerId, matte: { channel: "alpha", sourceId: recipientId } },
    "INVALID_ARGUMENT"
  );
  await reject(
    "timeline_update_item",
    { expectedRevision: 3, itemId: recipientId, matte: null },
    "REVISION_CONFLICT",
    true
  );
  await reject(
    "timeline_delete_item",
    { itemId: providerId },
    "ITEM_NOT_FOUND"
  );
  await reject(
    "timeline_batch_edit",
    {
      operations: [
        { itemId: recipientId, matte: null, operation: "update_item" },
        { itemId: "missing", operation: "delete_item" },
      ],
    },
    "ITEM_NOT_FOUND"
  );
  await reject(
    "timeline_batch_edit",
    {
      operations: [
        {
          itemId: recipientId,
          matte: { channel: "alpha", sourceId: "@future" },
          operation: "update_item",
        },
        add("future", "#ffffff"),
      ],
    },
    "VALIDATION_FAILED"
  );
  const operations = [
    { itemId: recipientId, matte: null, operation: "update_item" },
    { itemId: providerId, matteOnly: false, operation: "update_item" },
  ];
  const draft = await call(
    "draft_create",
    {
      expectedRevision: 4,
      operations: [
        {
          itemId: recipientId,
          matte: { channel: "alpha", sourceId: providerId },
          operation: "update_item",
        },
      ],
      projectId,
    },
    editDraftSchema
  );
  const getDraft = () =>
    call("draft_get", { draftId: draft.id, projectId }, editDraftSchema);
  expect(draft.operations[0]).toHaveProperty("matte", {
    channel: "alpha",
    sourceId: providerId,
  });
  expect(await read()).toEqual(before);
  await reject(
    "draft_update",
    {
      draftId: draft.id,
      operations: [
        {
          itemId: recipientId,
          matte: { channel: "alpha", sourceId: "missing" },
          operation: "update_item",
        },
      ],
    },
    "ITEM_NOT_FOUND"
  );
  expect(await getDraft()).toEqual(draft);
  const updated = await call(
    "draft_update",
    { draftId: draft.id, expectedRevision: 4, operations, projectId },
    editDraftSchema
  );
  expect(updated.operations).toEqual(operations);
  const rebased = await call(
    "draft_rebase",
    { draftId: draft.id, expectedRevision: 4, projectId },
    editDraftSchema
  );
  expect(rebased.operations).toEqual(operations);
  expect((await getDraft()).operations).toEqual(operations);
  expect(await read()).toEqual(before);
  await write("draft_commit", 4, { draftId: draft.id });
  expect(await item(recipientId)).not.toHaveProperty("matte");
  expect(await item(providerId)).not.toHaveProperty("matteOnly");
  await write("project_undo", 5, {});
  expect(await item(recipientId)).toHaveProperty("matte", {
    channel: "luma",
    sourceId: providerId,
  });
  await write("project_redo", 6, {});
  expect(await item(recipientId)).not.toHaveProperty("matte");
  await write("timeline_update_item", 7, {
    itemId: recipientId,
    matte: { channel: "alpha", sourceId: providerId },
  });
  await write("timeline_batch_edit", 8, {
    operations: [
      { itemId: providerId, operation: "delete_item" },
      { itemId: recipientId, matte: null, operation: "update_item" },
    ],
  });
  expect(await item(providerId)).toBeUndefined();
  expect(await item(recipientId)).not.toHaveProperty("matte");
};
