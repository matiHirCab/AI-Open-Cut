import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import ADVANCED from "../../../contracts/advanced-text-layout-v1.json";
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

export async function verifyRichTextWorkflow(
  client: Client,
  call: Call,
  projects: string
) {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.capabilities).toContain(CATALOG.capability);
  expect(status.capabilities).toContain(CATALOG.fontSizeUpdate.capability);
  expect(status.capabilities).toContain(LAYOUT.capability);
  expect(status.capabilities).toContain(STYLED.capability);
  expect(status.capabilities).toContain(ADVANCED.capability);
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
      style: {
        layout: ADVANCED.valid[1]?.layout,
        paintLayers: STYLED.valid[3]?.paintLayers,
      },
      trackId,
    },
    writeResultSchema
  );
  const [itemId] = added.changedIds;
  const saved = await read();
  expect(saved.project.schemaVersion).toBe(22);
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
  ).toMatchObject({
    document,
    style: { layout: ADVANCED.valid[1]?.layout },
    text: CATALOG.valid[1]?.text,
  });
  const invalidLayout = await client.callTool({
    arguments: {
      expectedRevision: added.revision,
      itemId,
      projectId,
      style: {
        layout: { bounds: { widthPx: 15 } },
        padding: { bottom: 0, left: 10, right: 10, top: 0 },
      },
    },
    name: "timeline_update_item",
  });
  expect(invalidLayout.isError).toBe(true);
  expect(await read()).toEqual(saved);
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
        {
          itemId: "@title",
          operation: "update_item",
          style: { layout: ADVANCED.valid[1]?.layout },
          text: "plain",
        },
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
  const current = (await read()).project;
  const projectPath = join(projects, projectId, "project.json");
  const historyPath = join(projects, projectId, "history.json");
  const originalProject = readFileSync(projectPath);
  const originalHistory = readFileSync(historyPath);
  const invalidRoot = (layout: unknown) => ({
    ...current,
    tracks: current.tracks.map((track) => ({
      ...track,
      items: track.items.map((item) =>
        item.type === "text"
          ? { ...item, style: { ...item.style, layout } }
          : item
      ),
    })),
  });
  const expectPersistedError = async (code: string) => {
    const result = await client.callTool({
      arguments: { projectId },
      name: "project_open",
    });
    expect(result.isError).toBe(true);
    expect(result.structuredContent).toMatchObject({
      error: { code, retryable: false },
    });
    expect(JSON.stringify(result)).not.toContain("OPENCUT_LAYOUT_DECODE");
  };
  const verifyPersistedProject = async (project: unknown, code: string) => {
    const bytes = Buffer.from(JSON.stringify(project));
    try {
      writeFileSync(projectPath, bytes);
      await expectPersistedError(code);
      expect(readFileSync(projectPath)).toEqual(bytes);
      expect(readFileSync(historyPath)).toEqual(originalHistory);
    } finally {
      writeFileSync(projectPath, originalProject);
    }
  };
  await verifyPersistedProject(
    invalidRoot({ trackingPx: -1 }),
    "INVALID_ARGUMENT"
  );
  await verifyPersistedProject(invalidRoot(null), "INVALID_ARGUMENT");
  await verifyPersistedProject({ ...current, name: 42 }, "INTERNAL_ERROR");
  const retained = await call(
    "draft_create",
    {
      expectedRevision: current.revision,
      operations: [{ itemId, operation: "update_item", style: { layout: {} } }],
      projectId,
    },
    editDraftSchema
  );
  const draftPath = join(projects, projectId, "drafts", `${retained.id}.json`);
  const originalDraft = readFileSync(draftPath);
  const malformedDraft = Buffer.from(
    JSON.stringify({
      ...retained,
      operations: [
        { itemId, operation: "update_item", style: { layout: null } },
      ],
    })
  );
  try {
    writeFileSync(draftPath, malformedDraft);
    await expectPersistedError("INVALID_ARGUMENT");
    expect(readFileSync(draftPath)).toEqual(malformedDraft);
    expect(readFileSync(projectPath)).toEqual(originalProject);
    expect(readFileSync(historyPath)).toEqual(originalHistory);
  } finally {
    writeFileSync(draftPath, originalDraft);
  }
  expect((await read()).project).toEqual(current);

  const originalItem = current.tracks
    .flatMap((t) => t.items)
    .find((i) => i.id === itemId);
  const resized = await call(
    "timeline_update_item",
    {
      expectedRevision: current.revision,
      fontSize: 72,
      itemId,
      projectId,
    },
    writeResultSchema
  );
  const resizedState = await read();
  expect(
    resizedState.project.tracks
      .flatMap((t) => t.items)
      .find((i) => i.id === itemId)
  ).toEqual({ ...originalItem, fontSize: 72 });
  const failedSizes = await Promise.all(
    [
      { fontSize: 0, itemId },
      { fontSize: 1001, itemId },
      { fontSize: 80, itemId: "missing" },
    ].map((sizeInput) =>
      client.callTool({
        arguments: {
          expectedRevision: resized.revision,
          projectId,
          ...sizeInput,
        },
        name: "timeline_update_item",
      })
    )
  );
  for (const failedSize of failedSizes) {
    expect(failedSize.isError).toBe(true);
  }
  expect(await read()).toEqual(resizedState);
  const staleSize = await client.callTool({
    arguments: {
      expectedRevision: current.revision,
      fontSize: 80,
      itemId,
      projectId,
    },
    name: "timeline_update_item",
  });
  expect(staleSize.isError).toBe(true);
  expect(await read()).toEqual(resizedState);
  const failedSizeBatch = await client.callTool({
    arguments: {
      expectedRevision: resized.revision,
      operations: [
        { fontSize: 80, itemId, operation: "update_item" },
        { itemId: "missing", operation: "delete_item" },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(failedSizeBatch.isError).toBe(true);
  expect(await read()).toEqual(resizedState);
  const aliased = await call(
    "timeline_batch_edit",
    {
      expectedRevision: resized.revision,
      operations: [
        {
          color: "#ffffff",
          durationMs: 1000,
          fontSize: 48,
          operation: "add_text",
          resultAlias: "resized",
          startMs: 0,
          text: "Sizing",
          trackId,
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
        },
        { fontSize: 96, itemId: "@resized", operation: "update_item" },
      ],
      projectId,
    },
    writeResultSchema
  );
  const aliasedState = await read();
  expect(aliasedState.project.tracks.flatMap((t) => t.items)).toEqual(
    expect.arrayContaining([
      expect.objectContaining({ fontSize: 96, text: "Sizing" }),
    ])
  );
  const undone = await call(
    "project_undo",
    { expectedRevision: aliased.revision, projectId },
    writeResultSchema
  );
  expect((await read()).project.tracks.flatMap((t) => t.items)).not.toEqual(
    expect.arrayContaining([expect.objectContaining({ text: "Sizing" })])
  );
  await call(
    "project_redo",
    { expectedRevision: undone.revision, projectId },
    writeResultSchema
  );
  expect((await read()).project.tracks.flatMap((t) => t.items)).toEqual(
    aliasedState.project.tracks.flatMap((t) => t.items)
  );
}
