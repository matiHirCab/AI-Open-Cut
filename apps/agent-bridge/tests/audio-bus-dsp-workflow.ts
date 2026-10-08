import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import catalog from "../../../contracts/audio-bus-dsp-v1.json";
import {
  editDraftSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <T>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<T>
) => Promise<T>;
export const verifyAudioBusDspWorkflow = async (client: Client, call: Call) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.projectSchemaVersion).toBe(43);
  const { tools } = await client.request({ method: "tools/list" });
  expect(tools.filter((t) => t.name === catalog.operation)).toHaveLength(1);
  const { projectId } = await call(
    "project_create",
    { name: "Normalized bus DSP" },
    writeResultSchema
  );
  const { operation: _operation, ...input } = catalog.input;
  const applied = await call(
    catalog.operation,
    { ...input, expectedRevision: 0, projectId },
    writeResultSchema
  );
  expect(applied.changedIds).toEqual(["music"]);
  const read = () =>
    call("project_get_state", { projectId }, projectStateSchema);
  expect((await read()).project.audioBuses[1]?.dsp).toEqual(input.dsp);
  const draft = await call(
    "draft_create",
    {
      expectedRevision: 1,
      operations: [
        { busId: "music", dsp: catalog.identity, operation: catalog.operation },
      ],
      projectId,
    },
    editDraftSchema
  );
  expect((await read()).project.audioBuses[1]?.dsp).toEqual(input.dsp);
  const persisted = await call(
    "draft_get",
    { draftId: draft.id, projectId },
    editDraftSchema
  );
  expect(persisted.operations).toEqual([
    { busId: "music", dsp: catalog.identity, operation: catalog.operation },
  ]);
  await call(
    "draft_commit",
    { draftId: draft.id, expectedRevision: 1, projectId },
    writeResultSchema
  );
  expect((await read()).project.audioBuses[1]?.dsp).toEqual(catalog.identity);
  await call(
    "timeline_batch_edit",
    {
      expectedRevision: 2,
      operations: [
        catalog.input,
        {
          busId: "master",
          dsp: catalog.identity,
          operation: catalog.operation,
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  expect((await read()).project.audioBuses[1]?.dsp).toEqual(input.dsp);
  await call(
    "project_undo",
    { expectedRevision: 3, projectId },
    writeResultSchema
  );
  expect((await read()).project.audioBuses[1]?.dsp).toEqual(catalog.identity);
  await call(
    "project_redo",
    { expectedRevision: 4, projectId },
    writeResultSchema
  );
  expect((await read()).project.audioBuses[1]?.dsp).toEqual(input.dsp);
};
