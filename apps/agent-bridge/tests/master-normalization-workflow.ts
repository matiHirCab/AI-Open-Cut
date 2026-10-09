import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect, vi } from "vitest";
import type { ZodType } from "zod/v4";
import catalog from "../../../contracts/master-normalization-v1.json";
import {
  editDraftSchema,
  jobSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

const TRANSPORT_VALIDATION = /invalid|validation/i;

type Call = <T>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<T>
) => Promise<T>;
export const verifyMasterNormalizationWorkflow = async (
  client: Client,
  call: Call
) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.projectSchemaVersion).toBe(catalog.projectSchemaVersion);
  expect(status.protocolVersion).toBe(1);
  expect(status.subsystems.rendering.capabilities).toContain(
    catalog.capability
  );
  const { tools } = await client.request({ method: "tools/list" });
  expect(tools.filter((tool) => tool.name === catalog.tool)).toHaveLength(1);
  const { projectId } = await call(
    "project_create",
    { height: 32, name: "Master normalization workflow", width: 32 },
    writeResultSchema
  );
  const read = () =>
    call("project_get_state", { projectId }, projectStateSchema);
  const initial = await read();
  expect(initial.project.masterNormalization).toBeUndefined();
  const root = join(status.paths.projectsDirectory.resolvedPath, projectId);
  const files = () => {
    const values: Record<string, string> = {};
    const visit = (path: string) => {
      for (const name of readdirSync(path)) {
        const child = join(path, name);
        if (statSync(child).isDirectory()) {
          visit(child);
        } else {
          values[child] = readFileSync(child).toString("base64");
        }
      }
    };
    visit(root);
    return values;
  };
  const fail = async (
    name: string,
    args: Record<string, unknown>,
    code: string,
    transport = false
  ) => {
    const before = await read();
    const bytes = files();
    const result = await client.callTool({
      arguments: { projectId, ...args },
      name,
    });
    expect(result.isError).toBe(true);
    if (transport) {
      expect(result.structuredContent).toBeUndefined();
      expect(result.content).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            text: expect.stringMatching(TRANSPORT_VALIDATION),
            type: "text",
          }),
        ])
      );
    } else {
      expect(result.structuredContent).toMatchObject({
        error: { code, retryable: code === "REVISION_CONFLICT" },
      });
    }
    expect(await read()).toEqual(before);
    expect(files()).toEqual(bytes);
  };
  await fail(
    catalog.tool,
    { expectedRevision: 99, normalization: catalog.settingsExample },
    "REVISION_CONFLICT"
  );
  await fail(
    catalog.tool,
    {
      expectedRevision: 0,
      normalization: { ...catalog.disabledExample, targetTruePeakDbtp: 0.01 },
    },
    "VALIDATION_FAILED",
    true
  );
  await fail(
    "timeline_batch_edit",
    {
      expectedRevision: 0,
      operations: [
        catalog.input,
        { markerId: "missing", operation: "marker_delete", scope: "root" },
      ],
    },
    "ITEM_NOT_FOUND"
  );
  await fail(
    "timeline_batch_edit",
    {
      expectedRevision: 0,
      operations: [{ ...catalog.input, resultAlias: "master" }],
    },
    "VALIDATION_FAILED",
    true
  );
  const applied = await call(
    catalog.tool,
    { expectedRevision: 0, normalization: catalog.settingsExample, projectId },
    writeResultSchema
  );
  expect(applied.changedIds).toEqual(catalog.changedIds);
  expect((await read()).project.masterNormalization).toEqual(
    catalog.settingsExample
  );
  const batch = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 1,
      operations: [
        {
          color: "#112233",
          durationMs: 1000,
          operation: "add_solid_color",
          resultAlias: "background",
          startMs: 0,
          trackId: initial.project.tracks[1]?.id,
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
        },
        { ...catalog.input, normalization: catalog.disabledExample },
      ],
      projectId,
    },
    writeResultSchema
  );
  expect(batch.aliases.background).toBe(batch.changedIds[0]);
  expect(batch.changedIds.at(-1)).toBe("master");
  const draft = await call(
    "draft_create",
    { expectedRevision: 2, operations: [catalog.input], projectId },
    editDraftSchema
  );
  expect((await read()).project.masterNormalization).toEqual(
    catalog.disabledExample
  );
  expect(
    (await call("draft_get", { draftId: draft.id, projectId }, editDraftSchema))
      .operations
  ).toEqual([catalog.input]);
  await call(
    "draft_commit",
    { draftId: draft.id, expectedRevision: 2, projectId },
    writeResultSchema
  );
  await call(
    "project_undo",
    { expectedRevision: 3, projectId },
    writeResultSchema
  );
  expect((await read()).project.masterNormalization).toEqual(
    catalog.disabledExample
  );
  await call(
    "project_redo",
    { expectedRevision: 4, projectId },
    writeResultSchema
  );
  expect((await read()).project.masterNormalization).toEqual(
    catalog.settingsExample
  );
  const before = await read();
  expect(await call("project_open", { projectId }, projectStateSchema)).toEqual(
    before
  );
  await fail(
    catalog.tool,
    { expectedRevision: 4, normalization: catalog.settingsExample },
    "REVISION_CONFLICT"
  );
  const persisted = () => [
    readFileSync(join(root, "project.json")),
    readFileSync(join(root, "history.json")),
  ];
  const bytes = persisted();
  const jobs = await Promise.all([
    call(
      "audio_analyze_mix",
      {
        endMs: 1000,
        expectedRevision: 5,
        projectId,
        startMs: 0,
        waveformBins: 7,
      },
      jobSchema
    ),
    call(
      "audio_analyze_mix",
      {
        endMs: 400,
        expectedRevision: 5,
        projectId,
        startMs: 200,
        waveformBins: 2,
      },
      jobSchema
    ),
  ]);
  await Promise.all(
    jobs.map(async (queued) => {
      await vi.waitFor(
        async () => {
          const result = await call(
            "job_get_status",
            { jobId: queued.jobId },
            jobSchema
          );
          expect(result.status).toBe("completed");
          expect(result.audioAnalysis).toMatchObject({
            channels: 2,
            integratedLufs: null,
            linearSamplePeak: 0,
            sampleRateHz: 48_000,
            truePeakDbtp: null,
          });
          expect(result.artifact?.mimeType).toBe("application/json");
        },
        { timeout: 10_000 }
      );
    })
  );
  expect(persisted()).toEqual(bytes);
  const preview = await call(
    "preview_render_frame",
    { expectedRevision: 5, projectId, timeMs: 200 },
    jobSchema
  );
  await vi.waitFor(
    async () => {
      expect(
        (await call("job_get_status", { jobId: preview.jobId }, jobSchema))
          .status
      ).toBe("completed");
    },
    { timeout: 10_000 }
  );
  expect(persisted()).toEqual(bytes);
};
