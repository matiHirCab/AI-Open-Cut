import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import masterNormalization from "../../../contracts/master-normalization-v1.json";
import catalog from "../../../contracts/semantic-sound-events-v1.json";
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

export const verifySoundEventWorkflow = async (
  client: Client,
  call: Call,
  mediaDirectory: string
) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(catalog.projectSchemaVersion).toBe(40);
  expect(status.projectSchemaVersion).toBe(
    masterNormalization.projectSchemaVersion
  );
  expect(status.subsystems.editor.capabilities).toContain(catalog.capability);
  expect(
    (await client.listTools()).tools.filter(
      (tool) => tool.name === catalog.operation
    )
  ).toHaveLength(1);
  const { projectId } = await call(
    "project_create",
    { name: "Semantic sound definitions" },
    writeResultSchema
  );
  const read = () =>
    call("project_get_state", { projectId }, projectStateSchema);
  const initial = await read();
  expect(initial.project.soundDefinitions).toEqual([]);
  // Valid independent PCM files; smoke's explicit fake probe is transport coverage,
  // with real FFmpeg registration/no-output parity covered by the native suites.
  const importVariant = async (index: number, amplitude: number) => {
    const wav = Buffer.alloc(44 + 1600);
    wav.write("RIFF", 0);
    wav.writeUInt32LE(wav.length - 8, 4);
    wav.write("WAVEfmt ", 8);
    wav.writeUInt32LE(16, 16);
    wav.writeUInt16LE(1, 20);
    wav.writeUInt16LE(1, 22);
    wav.writeUInt32LE(8000, 24);
    wav.writeUInt32LE(16_000, 28);
    wav.writeUInt16LE(2, 32);
    wav.writeUInt16LE(16, 34);
    wav.write("data", 36);
    wav.writeUInt32LE(1600, 40);
    for (let sample = 0; sample < 800; sample += 1) {
      wav.writeInt16LE(sample % 2 ? amplitude : -amplitude, 44 + sample * 2);
    }
    const path = join(mediaDirectory, `semantic-sound-${index}.wav`);
    writeFileSync(path, wav);
    const imported = await call(
      "asset_import",
      { expectedRevision: index, mediaType: "audio", path, projectId },
      writeResultSchema
    );
    const [asset] = imported.changedIds;
    if (!asset) {
      throw new Error("Imported sound ID missing");
    }
    return asset;
  };
  const assets = [await importVariant(0, 300), await importVariant(1, 700)];
  const fields = {
    busId: "sfx",
    defaultGainDb: -3,
    event: "impact",
    variantAssetIds: assets,
    variantSeed: 42,
  };
  const projectRoot = join(
    status.paths.projectsDirectory.resolvedPath,
    projectId
  );
  const files = () => {
    const result: Record<string, string> = {};
    const visit = (path: string) => {
      for (const name of readdirSync(path)) {
        const child = join(path, name);
        if (statSync(child).isDirectory()) {
          visit(child);
        } else {
          result[child] = readFileSync(child).toString("base64");
        }
      }
    };
    visit(projectRoot);
    return result;
  };
  const rejected = async (
    name: string,
    args: Record<string, unknown>,
    code: string
  ) => {
    const before = await read();
    const bytes = files();
    const response = await client.callTool({
      arguments: { projectId, ...args },
      name,
    });
    expect(response.isError).toBe(true);
    expect(response.structuredContent).toMatchObject({
      error: { code, retryable: code === "REVISION_CONFLICT" },
    });
    expect(await read()).toEqual(before);
    expect(files()).toEqual(bytes);
  };
  await rejected(
    catalog.operation,
    { ...fields, expectedRevision: 1 },
    "REVISION_CONFLICT"
  );
  await rejected(
    catalog.operation,
    { ...fields, expectedRevision: 2, variantAssetIds: ["absent"] },
    "ASSET_NOT_FOUND"
  );
  await rejected(
    catalog.operation,
    { ...fields, busId: "absent", expectedRevision: 2 },
    "INVALID_ARGUMENT"
  );
  await rejected(
    "timeline_batch_edit",
    {
      expectedRevision: 2,
      operations: [
        { operation: catalog.operation, ...fields },
        {
          operation: catalog.operation,
          ...fields,
          event: "late",
          variantAssetIds: ["absent"],
        },
      ],
    },
    "ASSET_NOT_FOUND"
  );
  await call(
    catalog.operation,
    { expectedRevision: 2, projectId, ...fields },
    writeResultSchema
  );
  expect((await read()).project.soundDefinitions).toEqual([fields]);
  await rejected(
    "asset_delete",
    { assetId: assets[0], expectedRevision: 3 },
    "ASSET_IN_USE"
  );
  const batch = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 3,
      operations: [
        { operation: catalog.operation, ...fields, resultAlias: "hit" },
        {
          operation: catalog.operation,
          ...fields,
          defaultGainDb: -6,
          event: "@hit",
          variantAssetIds: assets.slice(1),
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  expect(batch.aliases).toEqual({ hit: "impact" });
  const replaced = (await read()).project;
  expect(replaced.soundDefinitions).toEqual([
    { ...fields, defaultGainDb: -6, variantAssetIds: assets.slice(1) },
  ]);
  expect(replaced.tracks).toEqual(initial.project.tracks);
  const draft = await call(
    "draft_create",
    {
      expectedRevision: 4,
      operations: [
        {
          operation: catalog.operation,
          ...fields,
          event: "draft_hit",
          variantAssetIds: assets.slice(0, 1),
        },
      ],
      projectId,
    },
    editDraftSchema
  );
  expect((await read()).project).toEqual(replaced);
  await rejected(
    "asset_delete",
    { assetId: assets[0], expectedRevision: 4 },
    "ASSET_IN_USE"
  );
  await call(
    "draft_update",
    {
      draftId: draft.id,
      expectedRevision: 4,
      operations: [
        {
          operation: catalog.operation,
          ...fields,
          event: "draft_hit",
          variantSeed: 1,
        },
      ],
      projectId,
    },
    editDraftSchema
  );
  await rejected(
    "draft_commit",
    { draftId: draft.id, expectedRevision: 3 },
    "REVISION_CONFLICT"
  );
  await call(
    "draft_rebase",
    { draftId: draft.id, expectedRevision: 4, projectId },
    editDraftSchema
  );
  await call(
    "draft_commit",
    { draftId: draft.id, expectedRevision: 4, projectId },
    writeResultSchema
  );
  const committed = (await read()).project;
  expect(committed.soundDefinitions).toHaveLength(2);
  await call(
    "project_undo",
    { expectedRevision: 5, projectId },
    writeResultSchema
  );
  expect((await read()).project.soundDefinitions).toEqual(
    replaced.soundDefinitions
  );
  await call(
    "project_redo",
    { expectedRevision: 6, projectId },
    writeResultSchema
  );
  expect((await read()).project.soundDefinitions).toEqual(
    committed.soundDefinitions
  );
  await call("project_open", { projectId }, projectStateSchema);
  expect((await read()).project.soundDefinitions).toEqual(
    committed.soundDefinitions
  );
};
