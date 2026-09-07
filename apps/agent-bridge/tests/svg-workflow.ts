import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import CATALOG from "../../../contracts/svg-ingestion-v1.json";
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
export async function verifySvgWorkflow(client: Client, call: Call) {
  const [first, second] = CATALOG.valid;
  if (!(first && second)) {
    throw new Error("SVG fixtures are missing");
  }
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.capabilities).toContain("svg_items");
  expect(status.capabilities).toContain("svg_rendering");
  const created = await call(
    "project_create",
    { name: "SVG smoke" },
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
    projectId,
    startMs: 0,
    svg: first.svg,
    trackId,
  };
  let result = await call(
    "timeline_add_svg",
    { ...fields, expectedRevision: created.revision },
    writeResultSchema
  );
  result = await CATALOG.valid.slice(2).reduce(async (previous, fixture) => {
    const current = await previous;
    return call(
      "timeline_add_svg",
      { ...fields, expectedRevision: current.revision, svg: fixture.svg },
      writeResultSchema
    );
  }, Promise.resolve(result));
  const before = await read();
  await Promise.all(
    CATALOG.invalid.map(async (fixture) => {
      const failed = await client.callTool({
        arguments: {
          ...fields,
          expectedRevision: result.revision,
          svg: fixture.svg,
        },
        name: "timeline_add_svg",
      });
      expect(failed.isError, fixture.id).toBe(true);
    })
  );
  expect(await read()).toEqual(before);
  result = await call(
    "timeline_batch_edit",
    {
      expectedRevision: result.revision,
      operations: [
        {
          durationMs: 1000,
          operation: "add_svg",
          resultAlias: "icon",
          startMs: 0,
          svg:
            CATALOG.valid.find((f) => f.id === "post-close-continuation")
              ?.svg ?? second.svg,
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
  await [
    '<svg width="100" height="100" viewBox="0 0 0.0001 0.0001"><rect x="-5000" y="-5000" width="10000" height="10000"/></svg>',
    '<svg width="100" height="100" viewBox="0 0 .001 .001"><polygon points="-5000,-5000 5000,5000 5000,5000.0001 -5000,-4999.9999" fill="#f00"/></svg>',
  ].reduce(async (previous, svg) => {
    const current = await previous;
    const added = await call(
      "timeline_add_svg",
      { ...fields, expectedRevision: current.revision, svg },
      writeResultSchema
    );
    const invalidState = await read();
    const render = await call(
      "preview_render_frame",
      { expectedRevision: added.revision, projectId, timeMs: 0 },
      jobSchema
    );
    await expect
      .poll(() => call("job_get_status", { jobId: render.jobId }, jobSchema))
      .toMatchObject({ error: { code: "INVALID_ARGUMENT" }, status: "failed" });
    expect(await read()).toEqual(invalidState);
    return call(
      "project_undo",
      { expectedRevision: added.revision, projectId },
      writeResultSchema
    );
  }, Promise.resolve(result));
}
