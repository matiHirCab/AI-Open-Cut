import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import CATALOG from "../../../contracts/procedural-grids-v1.json";
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
export async function verifyGridWorkflow(client: Client, call: Call) {
  const [first, second] = CATALOG.valid;
  if (!(first && second)) {
    throw new Error("Grid fixtures are missing");
  }
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.capabilities).toContain("grid_items");
  expect(status.capabilities).toContain("grid_rendering");
  const created = await call(
    "project_create",
    { name: "Grid smoke" },
    writeResultSchema
  );
  const { projectId } = created;
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const state = await read();
  const trackId = state.project.tracks.find(
    (t) => t.trackType === "overlay"
  )?.id;
  const fields = {
    durationMs: 1000,
    grid: first.grid,
    projectId,
    startMs: 0,
    trackId,
  };
  let result = await call(
    "timeline_add_grid",
    { ...fields, expectedRevision: created.revision },
    writeResultSchema
  );
  result = await CATALOG.valid.slice(2).reduce(async (previous, fixture) => {
    const current = await previous;
    return call(
      "timeline_add_grid",
      { ...fields, expectedRevision: current.revision, grid: fixture.grid },
      writeResultSchema
    );
  }, Promise.resolve(result));
  const before = await read();
  const itemId = before.project.tracks
    .flatMap((track) => track.items)
    .find((item) => item.type === "grid")?.id;
  expect(itemId).toBeDefined();
  await Promise.all(
    CATALOG.invalid.map(async (fixture) => {
      const failed = await client.callTool({
        arguments: {
          ...fields,
          expectedRevision: result.revision,
          grid: fixture.grid,
        },
        name: "timeline_add_grid",
      });
      expect(failed.isError, fixture.id).toBe(true);
    })
  );
  expect(await read()).toEqual(before);
  result = await CATALOG.valid.reduce(async (previous, fixture) => {
    const current = await previous;
    const updated = await call(
      "timeline_update_item",
      {
        expectedRevision: current.revision,
        grid: fixture.grid,
        itemId,
        projectId,
      },
      writeResultSchema
    );
    expect(
      (await read()).project.tracks
        .flatMap((track) => track.items)
        .find((item) => item.id === itemId)
    ).toMatchObject({ grid: fixture.grid });
    return updated;
  }, Promise.resolve(result));
  const unchanged = await read();
  await [
    {
      expectedRevision: created.revision,
      operations: [{ grid: first.grid, itemId, operation: "update_item" }],
      projectId,
    },
    {
      expectedRevision: result.revision,
      operations: [
        {
          durationMs: 1000,
          grid: first.grid,
          operation: "add_grid",
          resultAlias: "rollback",
          startMs: 0,
          trackId,
        },
        { itemId: "missing-grid", operation: "delete_item" },
      ],
      projectId,
    },
  ].reduce(async (previous, arguments_) => {
    await previous;
    const failed = await client.callTool({
      arguments: arguments_,
      name: "timeline_batch_edit",
    });
    expect(failed.isError).toBe(true);
    const stale = arguments_.expectedRevision === created.revision;
    expect(failed.structuredContent).toMatchObject({
      error: {
        code: stale ? "REVISION_CONFLICT" : "ITEM_NOT_FOUND",
        retryable: stale,
      },
    });
    expect(await read()).toEqual(unchanged);
  }, Promise.resolve());
  result = await call(
    "timeline_batch_edit",
    {
      expectedRevision: result.revision,
      operations: [
        {
          durationMs: 1000,
          grid: second.grid,
          operation: "add_grid",
          resultAlias: "icon",
          startMs: 0,
          trackId,
        },
        { itemId: "@icon", operation: "item_set_z_index", zIndex: 3 },
      ],
      projectId,
    },
    writeResultSchema
  );
  const after = await read();
  result = await call(
    "project_undo",
    { expectedRevision: result.revision, projectId },
    writeResultSchema
  );
  result = await call(
    "project_redo",
    { expectedRevision: result.revision, projectId },
    writeResultSchema
  );
  expect((await read()).project.tracks).toEqual(after.project.tracks);
  const frame = await call(
    "preview_render_frame",
    { expectedRevision: result.revision, projectId, timeMs: 0 },
    jobSchema
  );
  await expect
    .poll(() => call("job_get_status", { jobId: frame.jobId }, jobSchema))
    .toMatchObject({ status: "completed" });
}
