import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import catalog from "../../../contracts/audio-bus-ducking-v1.json";
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
const TRANSPORT_VALIDATION = /invalid|validation/i;
export const verifyAudioBusDuckingWorkflow = async (
  client: Client,
  call: Call
) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.projectSchemaVersion).toBe(43);
  const { tools } = await client.request({ method: "tools/list" });
  expect(tools.filter((t) => t.name === catalog.operation)).toHaveLength(1);
  const { projectId } = await call(
    "project_create",
    { name: "Narration bus ducking" },
    writeResultSchema
  );
  const { operation: _operation, ...input } = catalog.input;
  const read = () =>
    call("project_get_state", { projectId }, projectStateSchema);
  const root = join(status.paths.projectsDirectory.resolvedPath, projectId);
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
    visit(root);
    return result;
  };
  const rejected = async (
    name: string,
    args: Record<string, unknown>,
    code: string,
    transportInput = false
  ) => {
    const before = await read();
    const bytes = files();
    const response = await client.callTool({
      arguments: { projectId, ...args },
      name,
    });
    expect(response.isError).toBe(true);
    if (transportInput) {
      expect(response.structuredContent).toBeUndefined();
      expect(response.content).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            text: expect.stringMatching(TRANSPORT_VALIDATION),
            type: "text",
          }),
        ])
      );
    } else {
      expect(response.structuredContent).toMatchObject({
        error: { code, retryable: code === "REVISION_CONFLICT" },
      });
    }
    expect(await read()).toEqual(before);
    expect(files()).toEqual(bytes);
  };
  await rejected(
    catalog.operation,
    { ...input, expectedRevision: 99 },
    "REVISION_CONFLICT"
  );
  await rejected(
    catalog.operation,
    { ...input, busId: "missing", expectedRevision: 0 },
    "INVALID_ARGUMENT"
  );
  await rejected(
    catalog.operation,
    {
      ...input,
      ducking: { ...input.ducking, sourceBusId: "music" },
      expectedRevision: 0,
    },
    "INVALID_ARGUMENT"
  );
  await rejected(
    "timeline_batch_edit",
    {
      expectedRevision: 0,
      operations: [catalog.input, { ...catalog.input, busId: "missing" }],
    },
    "INVALID_ARGUMENT"
  );
  await rejected(
    "timeline_batch_edit",
    {
      expectedRevision: 0,
      operations: [{ ...catalog.input, resultAlias: "bus" }],
    },
    "VALIDATION_FAILED",
    true
  );

  const applied = await call(
    catalog.operation,
    { ...input, expectedRevision: 0, projectId },
    writeResultSchema
  );
  expect(applied.changedIds).toEqual(["music"]);
  expect((await read()).project.audioBuses[1]?.ducking).toEqual(input.ducking);
  const draft = await call(
    "draft_create",
    {
      expectedRevision: 1,
      operations: [
        {
          busId: "music",
          ducking: catalog.identity,
          operation: catalog.operation,
        },
      ],
      projectId,
    },
    editDraftSchema
  );
  expect((await read()).project.audioBuses[1]?.ducking).toEqual(input.ducking);
  const persisted = await call(
    "draft_get",
    { draftId: draft.id, projectId },
    editDraftSchema
  );
  expect(persisted.operations).toEqual([
    { busId: "music", ducking: catalog.identity, operation: catalog.operation },
  ]);
  await call(
    "draft_commit",
    { draftId: draft.id, expectedRevision: 1, projectId },
    writeResultSchema
  );
  expect((await read()).project.audioBuses[1]?.ducking).toEqual(
    catalog.identity
  );
  await call(
    "timeline_batch_edit",
    {
      expectedRevision: 2,
      operations: [
        catalog.input,
        {
          busId: "master",
          ducking: catalog.identity,
          operation: catalog.operation,
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  expect((await read()).project.audioBuses[1]?.ducking).toEqual(input.ducking);
  await call(
    "project_undo",
    { expectedRevision: 3, projectId },
    writeResultSchema
  );
  expect((await read()).project.audioBuses[1]?.ducking).toEqual(
    catalog.identity
  );
  await call(
    "project_redo",
    { expectedRevision: 4, projectId },
    writeResultSchema
  );
  expect((await read()).project.audioBuses[1]?.ducking).toEqual(input.ducking);
  const beforeReopen = await read();
  const opened = await call("project_open", { projectId }, projectStateSchema);
  expect(opened).toEqual(beforeReopen);
  await rejected(
    catalog.operation,
    { ...input, expectedRevision: 4 },
    "REVISION_CONFLICT"
  );
};
