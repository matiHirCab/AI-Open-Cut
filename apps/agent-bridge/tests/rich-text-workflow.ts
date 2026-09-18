import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import CATALOG from "../../../contracts/rich-text-documents-v1.json";
import STYLED from "../../../contracts/styled-text-layers-v1.json";
import LAYOUT from "../../../contracts/text-layout-v2.json";
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

export async function verifyRichTextWorkflow(client: Client, call: Call) {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.capabilities).toContain(CATALOG.capability);
  expect(status.capabilities).toContain(LAYOUT.capability);
  expect(status.capabilities).toContain(STYLED.capability);
  expect(status.textLayoutVersion).toBe(2);
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
  const baseDocument = CATALOG.valid[1]?.document;
  if (!baseDocument) {
    throw new Error("Rich text fixture missing");
  }
  const document = {
    ...baseDocument,
    spans: [
      {
        end: 1,
        start: 0,
        style: {
          bold: false,
          paintLayers: [{ color: "#123456", kind: "fill", opacity: 0.5 }],
        },
      },
    ],
  };
  const added = await call(
    "timeline_add_text",
    {
      document,
      durationMs: 1000,
      expectedRevision: created.revision,
      projectId,
      startMs: 0,
      style: { paintLayers: STYLED.valid[3]?.paintLayers },
      trackId,
    },
    writeResultSchema
  );
  const [itemId] = added.changedIds;
  const saved = await read();
  expect(saved.project.schemaVersion).toBe(20);
  expect(Object.keys(saved.project.fonts)).toHaveLength(4);
  expect(Object.keys(saved.project.fonts).sort()).toEqual(
    [
      LAYOUT.defaultFamily.regular,
      LAYOUT.defaultFamily.bold,
      LAYOUT.defaultFamily.italic,
      LAYOUT.defaultFamily.boldItalic,
    ].sort()
  );
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
  const stale = await client.callTool({
    arguments: { document, expectedRevision: 0, itemId, projectId },
    name: "timeline_update_item",
  });
  expect(stale.isError).toBe(true);
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
  const draft = await call(
    "draft_create",
    {
      expectedRevision: final.project.revision,
      operations: [{ document, itemId, operation: "update_item" }],
      projectId,
    },
    editDraftSchema
  );
  expect(draft.version).toBe(2);
  expect(Object.keys(draft.fontCatalog).sort()).toEqual(
    Object.keys(saved.project.fonts).sort()
  );
  expect(
    await call("draft_get", { draftId: draft.id, projectId }, editDraftSchema)
  ).toEqual(draft);
  expect(await read()).toEqual(final);
  await call(
    "draft_commit",
    { draftId: draft.id, expectedRevision: final.project.revision, projectId },
    writeResultSchema
  );
  expect((await read()).project.fonts).toEqual(final.project.fonts);
}
