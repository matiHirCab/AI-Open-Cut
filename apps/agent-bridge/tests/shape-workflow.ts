import { readFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import CATALOG from "../../../contracts/shape-items-v1.json";
import {
  jobSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;
export async function verifyShapeWorkflow(
  client: Client,
  call: Call,
  projects: string
) {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.capabilities).toContain("shape_items");
  expect(status.capabilities).toContain("shape_rendering");
  const created = await call(
    "project_create",
    { name: "Shape smoke" },
    writeResultSchema
  );
  const { projectId } = created;
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const state = await read();
  const trackId = state.project.tracks.find(
    (t) => t.trackType === "overlay"
  )?.id;
  expect(trackId).toBeDefined();
  let revision = await CATALOG.valid.reduce(async (pending, fixture) => {
    const expectedRevision = await pending;
    const { operation: _operation, ...fields } = fixture.value;
    const { revision: next } = await call(
      "timeline_add_shape",
      { ...fields, expectedRevision, projectId, trackId },
      writeResultSchema
    );
    return next;
  }, Promise.resolve(created.revision));
  const files = () =>
    ["project.json", "history.json"].map((n) =>
      readFileSync(join(projects, projectId, n))
    );
  const waitForRender = async (
    jobId: string,
    remaining = 200
  ): Promise<void> => {
    const job = await call("job_get_status", { jobId }, jobSchema);
    if (job.status === "completed") {
      expect(job.artifact).toBeDefined();
      return;
    }
    if (job.status === "failed" || remaining === 0) {
      throw new Error(JSON.stringify(job.error ?? "shape render timed out"));
    }
    await new Promise((resolve) => setTimeout(resolve, 25));
    await waitForRender(jobId, remaining - 1);
  };
  const preview = await call(
    "preview_render_frame",
    { expectedRevision: revision, projectId, timeMs: 500 },
    jobSchema
  );
  await waitForRender(preview.jobId);
  const exported = await call(
    "project_export_video",
    {
      expectedRevision: revision,
      format: "mp4",
      overwrite: false,
      projectId,
      relativePath: "shapes.mp4",
      resolution: "project",
    },
    jobSchema
  );
  await waitForRender(exported.jobId);
  await Promise.all(
    CATALOG.invalid.map(async (fixture) => {
      const { operation: _operation, ...fields } = fixture.value;
      const before = files();
      const failed = await client.callTool({
        arguments: {
          ...fields,
          expectedRevision: revision,
          projectId,
          trackId,
        },
        name: "timeline_add_shape",
      });
      expect(failed.isError, fixture.id).toBe(true);
      expect(files()).toEqual(before);
    })
  );
  const result = await call(
    "timeline_batch_edit",
    {
      expectedRevision: revision,
      operations: [
        { ...CATALOG.valid[0]?.value, resultAlias: "shape", trackId },
        { itemId: "@shape", operation: "item_set_z_index", zIndex: 2 },
      ],
      projectId,
    },
    writeResultSchema
  );
  ({ revision } = result);
  const before = files();
  const failed = await client.callTool({
    arguments: {
      expectedRevision: revision,
      operations: [
        { ...CATALOG.valid[0]?.value, trackId },
        { itemId: "missing", operation: "delete_item" },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(failed.structuredContent).toMatchObject({
    error: { code: "ITEM_NOT_FOUND" },
  });
  expect(files()).toEqual(before);
  const conflict = await client.callTool({
    arguments: {
      ...CATALOG.valid[0]?.value,
      expectedRevision: 0,
      operation: undefined,
      projectId,
      trackId,
    },
    name: "timeline_add_shape",
  });
  expect(conflict.structuredContent).toMatchObject({
    error: { code: "REVISION_CONFLICT" },
  });
  expect(files()).toEqual(before);
  const {
    project: { tracks },
  } = await read();
  let r = await call(
    "project_undo",
    { expectedRevision: revision, projectId },
    writeResultSchema
  );
  r = await call(
    "project_redo",
    { expectedRevision: r.revision, projectId },
    writeResultSchema
  );
  expect((await read()).project.tracks).toEqual(tracks);
  r = await call(
    "track_update",
    { expectedRevision: r.revision, locked: true, projectId, trackId },
    writeResultSchema
  );
  const [first] = CATALOG.valid;
  if (!first) {
    throw new Error("shape catalog is empty");
  }
  const { operation: _operation, ...fields } = first.value;
  const locked = await client.callTool({
    arguments: { ...fields, expectedRevision: r.revision, projectId, trackId },
    name: "timeline_add_shape",
  });
  expect(locked.structuredContent).toMatchObject({
    error: { code: "TRACK_LOCKED" },
  });
}
