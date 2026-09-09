import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import CATALOG from "../../../contracts/rich-text-documents-v1.json";
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

export async function verifyRichTextWorkflow(client: Client, call: Call) {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.capabilities).toContain(CATALOG.capability);
  const created = await call(
    "project_create",
    { name: "Rich text smoke" },
    writeResultSchema
  );
  const { projectId } = created;
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const state = await read();
  const trackId = state.project.tracks.find(
    (t) => t.trackType === "overlay"
  )?.id;
  const document = CATALOG.valid[1]?.document;
  if (!document) {
    throw new Error("Rich text fixture missing");
  }
  const added = await call(
    "timeline_add_text",
    {
      document,
      durationMs: 1000,
      expectedRevision: created.revision,
      projectId,
      startMs: 0,
      trackId,
    },
    writeResultSchema
  );
  const [itemId] = added.changedIds;
  const saved = await read();
  expect(saved.project.schemaVersion).toBe(CATALOG.schemaVersion);
  expect(
    saved.project.tracks.flatMap((t) => t.items).find((i) => i.id === itemId)
  ).toMatchObject({ document, text: CATALOG.valid[1]?.text });
  const failed = await client.callTool({
    arguments: {
      expectedRevision: added.revision,
      operations: [
        { itemId, operation: "update_item", text: "temporary" },
        { document, itemId: "missing", operation: "update_item" },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(failed.isError).toBe(true);
  expect(await read()).toEqual(saved);
  const conflict = await client.callTool({
    arguments: {
      document,
      expectedRevision: added.revision,
      itemId,
      projectId,
      text: "x",
    },
    name: "timeline_update_item",
  });
  expect(conflict.isError).toBe(true);
  expect(await read()).toEqual(saved);
  await call(
    "timeline_batch_edit",
    {
      expectedRevision: added.revision,
      operations: [
        {
          color: "#ffffff",
          document,
          durationMs: 1000,
          fontSize: 48,
          operation: "add_text",
          resultAlias: "title",
          startMs: 0,
          trackId,
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
        },
        { itemId: "@title", operation: "update_item", text: "plain" },
      ],
      projectId,
    },
    writeResultSchema
  );
  const final = await read();
  expect(final.project.tracks.flatMap((t) => t.items)).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        document: { runs: [{ text: "plain" }] },
        text: "plain",
      }),
    ])
  );
}
