import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import contract from "../../../contracts/blend-modes-v1.json";
import audioEvents from "../../../contracts/timeline-audio-events-v1.json";
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
export const verifyBlendModeWorkflow = async (client: Client, call: Call) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.projectSchemaVersion).toBe(audioEvents.projectSchemaVersion);
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
    { fps: 10, height: 64, name: "Blend public lifecycle", width: 64 },
    writeResultSchema
  );
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const initial = await read();
  const trackId = initial.project.tracks[1]?.id;
  const created = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 0,
      operations: [
        {
          color: "#ff0000",
          durationMs: 1000,
          height: 16,
          operation: "add_rectangle",
          resultAlias: "leaf",
          startMs: 0,
          trackId,
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
          width: 16,
        },
        { blendMode: "multiply", itemId: "@leaf", operation: "update_item" },
      ],
      projectId,
    },
    writeResultSchema
  );
  const itemId = created.aliases.leaf;
  if (!itemId) {
    throw new Error("blend creation alias missing");
  }
  const item = async () =>
    (await read()).project.tracks[1]?.items.find(
      (value) => value.id === itemId
    );
  expect(await item()).toHaveProperty("blendMode", "multiply");
  await contract.fields.blendMode.values.reduce(async (previous, mode) => {
    await previous;
    const {
      project: { revision },
    } = await read();
    await call(
      "timeline_update_item",
      { blendMode: mode, expectedRevision: revision, itemId, projectId },
      writeResultSchema
    );
    if (mode === "normal") {
      expect(await item()).not.toHaveProperty("blendMode");
    } else {
      expect(await item()).toHaveProperty("blendMode", mode);
    }
  }, Promise.resolve());
  const before = await read();
  const { revision } = before.project;
  const reject = async (
    name: string,
    input: Record<string, unknown>,
    code: string,
    retryable: boolean
  ) => {
    const failed = await client.callTool({
      arguments: { expectedRevision: revision, projectId, ...input },
      name,
    });
    expect(failed.isError).toBe(true);
    expect(failed.structuredContent).toMatchObject({
      error: { code, retryable },
    });
    expect((await read()).project).toEqual(before.project);
  };
  await [null, "PLUS"].reduce(async (previous, blendMode) => {
    await previous;
    const failed = await client.callTool({
      arguments: { blendMode, expectedRevision: revision, itemId, projectId },
      name: "timeline_update_item",
    });
    expect(failed.isError).toBe(true);
    expect(failed.structuredContent).toBeUndefined();
    expect(
      failed.content.some(
        (content) =>
          content.type === "text" && content.text.includes("blendMode")
      )
    ).toBe(true);
    expect((await read()).project).toEqual(before.project);
  }, Promise.resolve());
  await reject(
    "timeline_update_item",
    { blendMode: "multiply", itemId: "missing" },
    "ITEM_NOT_FOUND",
    false
  );
  await reject(
    "timeline_update_item",
    { blendMode: "multiply", expectedRevision: revision - 1, itemId },
    "REVISION_CONFLICT",
    true
  );
  await reject(
    "timeline_batch_edit",
    {
      operations: [
        { blendMode: "screen", itemId, operation: "update_item" },
        { itemId: "missing", operation: "delete_item" },
      ],
    },
    "ITEM_NOT_FOUND",
    false
  );
  const draft = await call(
    "draft_create",
    {
      expectedRevision: revision,
      operations: [{ blendMode: "multiply", itemId, operation: "update_item" }],
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
    { blendMode: "multiply", itemId, operation: "update_item" },
  ]);
  expect((await read()).project).toEqual(before.project);
  await call(
    "draft_commit",
    { draftId: draft.id, expectedRevision: revision, projectId },
    writeResultSchema
  );
  expect(await item()).toHaveProperty("blendMode", "multiply");
  await call(
    "project_undo",
    { expectedRevision: revision + 1, projectId },
    writeResultSchema
  );
  expect(await item()).toHaveProperty("blendMode", "lighten");
  await call(
    "project_redo",
    { expectedRevision: revision + 2, projectId },
    writeResultSchema
  );
  expect(await item()).toHaveProperty("blendMode", "multiply");
};
