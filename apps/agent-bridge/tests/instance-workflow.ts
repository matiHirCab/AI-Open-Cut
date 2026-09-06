import { readFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import {
  jobSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;

export const verifyInstanceWorkflow = async (
  client: Client,
  call: Call,
  projectsDirectory: string
) => {
  const { projectId } = await call(
    "project_create",
    { name: "Rendered instances" },
    writeResultSchema
  );
  const read = () =>
    call("project_get_state", { projectId }, projectStateSchema);
  const state = await read();
  const trackId = state.project.tracks.find(
    (track) => track.trackType === "overlay"
  )?.id;
  expect(trackId).toBeDefined();
  const fields = {
    componentId: "@definition",
    durationMs: 500,
    startMs: 100,
    timeScale: 1.5,
    trimStartMs: 0,
  };
  const created = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 0,
      operations: [
        {
          durationMs: 1000,
          height: 64,
          name: "Leaf",
          operation: "component_create",
          resultAlias: "definition",
          tracks: [
            {
              id: "local",
              items: [
                {
                  color: "#FF0000",
                  durationMs: 1000,
                  height: 20,
                  id: "box",
                  keyframes: [],
                  startMs: 0,
                  type: "rectangle",
                  width: 20,
                },
              ],
              name: "Local",
              trackType: "overlay",
            },
          ],
          width: 64,
        },
        {
          operation: "add_component_instance",
          ...fields,
          resultAlias: "instance",
          trackId,
        },
        {
          operation: "component_instance_update",
          ...fields,
          itemId: "@instance",
          startMs: 200,
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  const componentId = created.aliases.definition;
  const itemId = created.aliases.instance;
  const expected = (await read()).project.tracks;
  expect(expected.flatMap((track) => track.items)).toContainEqual(
    expect.objectContaining({
      id: itemId,
      startMs: 200,
      type: "component_instance",
    })
  );
  const snapshot = () =>
    ["project.json", "history.json"].map((name) =>
      readFileSync(join(projectsDirectory, projectId, name))
    );
  const before = snapshot();
  const failures = [
    {
      expectedRevision: 1,
      name: "component_instance_update",
      ...fields,
      componentId,
      itemId,
      timeScale: 0,
    },
    {
      expectedRevision: 1,
      name: "component_instance_update",
      ...fields,
      componentId: "missing",
      itemId,
    },
    {
      expectedRevision: 0,
      name: "component_instance_update",
      ...fields,
      componentId,
      itemId,
    },
    {
      expectedRevision: 1,
      name: "timeline_batch_edit",
      operations: [
        {
          operation: "component_instance_update",
          ...fields,
          componentId,
          itemId,
        },
        { componentId: "missing", operation: "component_delete" },
      ],
    },
  ];
  for (const { name, ...arguments_ } of failures) {
    // biome-ignore lint/performance/noAwaitInLoops: Check every failed mutation against the same committed generation.
    const failure = await client.callTool({
      arguments: { projectId, ...arguments_ },
      name,
    });
    expect(failure.isError).toBe(true);
    expect(snapshot()).toEqual(before);
  }
  const render = await call(
    "preview_render_frame",
    { expectedRevision: 1, projectId, timeMs: 300 },
    jobSchema
  );
  await expect
    .poll(async () => {
      const job = await call(
        "job_get_status",
        { jobId: render.jobId },
        jobSchema
      );
      if (job.status === "failed") {
        throw new Error(JSON.stringify(job.error));
      }
      return job;
    })
    .toMatchObject({ status: "completed" });
  const exported = await call(
    "project_export_video",
    {
      expectedRevision: 1,
      format: "mp4",
      overwrite: false,
      projectId,
      relativePath: `${projectId}.mp4`,
      resolution: "project",
    },
    jobSchema
  );
  await expect
    .poll(() => call("job_get_status", { jobId: exported.jobId }, jobSchema))
    .toMatchObject({ status: "completed" });
  expect(snapshot()).toEqual(before);
  await call(
    "project_undo",
    { expectedRevision: 1, projectId },
    writeResultSchema
  );
  expect((await read()).project.components).toEqual([]);
  await call(
    "project_redo",
    { expectedRevision: 2, projectId },
    writeResultSchema
  );
  expect((await read()).project.tracks).toEqual(expected);
  const reopened = await call(
    "project_open",
    { projectId },
    projectStateSchema
  );
  expect(reopened.project.tracks).toEqual(expected);
  await call(
    "component_instance_update",
    {
      expectedRevision: 3,
      projectId,
      ...fields,
      componentId,
      itemId,
      slotValues: {},
    },
    writeResultSchema
  );
  await call(
    "timeline_batch_edit",
    {
      expectedRevision: 4,
      operations: [{ locked: true, operation: "update_track", trackId }],
      projectId,
    },
    writeResultSchema
  );
  const locked = snapshot();
  const rejected = await client.callTool({
    arguments: {
      expectedRevision: 5,
      projectId,
      ...fields,
      componentId,
      itemId,
    },
    name: "component_instance_update",
  });
  expect(rejected.isError).toBe(true);
  expect(
    JSON.stringify(rejected.structuredContent ?? rejected.content)
  ).toContain("TRACK_LOCKED");
  expect(snapshot()).toEqual(locked);
};
