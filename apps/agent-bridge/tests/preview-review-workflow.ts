import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import {
  editDraftSchema,
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

export const verifyPreviewReviewWorkflow = async (
  client: Client,
  call: Call,
  projects: string
) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.subsystems.rendering.capabilities).toContain(
    "preview_review_presets_v1"
  );
  expect(status.capabilities).toContain("preview_review_presets_v1");
  // Portrait geometry bounds CPU work for every fixed-height review preset.
  const { projectId } = await call(
    "project_create",
    { fps: 10, height: 64, name: "Preset review", width: 16 },
    writeResultSchema
  );
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const initial = await read();
  const added = await call(
    "timeline_add_solid_color",
    {
      color: "#112233",
      durationMs: 1000,
      expectedRevision: 0,
      projectId,
      startMs: 0,
      trackId: initial.project.tracks[1]?.id,
      transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
    },
    writeResultSchema
  );
  await call(
    "draft_create",
    {
      expectedRevision: 1,
      label: "Retained review draft",
      operations: [
        {
          color: "#223344",
          itemId: added.changedIds[0],
          operation: "update_item",
        },
      ],
      projectId,
    },
    editDraftSchema
  );
  const directory = join(projects, projectId);
  const persisted = () =>
    Object.fromEntries(
      [
        "project.json",
        "history.json",
        ...readdirSync(join(directory, "drafts"))
          .filter((name) => name.endsWith(".json"))
          .map((name) => `drafts/${name}`),
      ].map((name) => [name, readFileSync(join(directory, name), "utf8")])
    );
  const diskBefore = persisted();
  const before = await read();
  const terminal = async (
    jobId: string,
    remaining = 200
  ): Promise<ReturnType<typeof jobSchema.parse>> => {
    const state = await call("job_get_status", { jobId }, jobSchema);
    if (
      ["completed", "failed", "cancelled"].includes(state.status) ||
      remaining === 0
    ) {
      return state;
    }
    await new Promise((resolve) => setTimeout(resolve, 25));
    return terminal(jobId, remaining - 1);
  };
  await Promise.all(
    [
      {},
      { resolution: "540p" },
      { resolution: "720p" },
      { resolution: "project" },
      { fps: 15, includeAudio: false, resolution: { height: 90, width: 160 } },
    ].map(async (selection) => {
      const queued = await call(
        "preview_review_range",
        {
          endMs: 1000,
          expectedRevision: added.revision,
          projectId,
          startMs: 0,
          ...selection,
        },
        jobSchema
      );
      const complete = await terminal(queued.jobId);
      expect(complete).toMatchObject({
        artifact: { mimeType: "video/mp4" },
        kind: "preview_range",
        revision: 1,
        status: "completed",
      });
      expect(complete.artifact?.sizeBytes).toBeGreaterThan(0);
      expect(complete.artifactResource).toMatchObject({
        mimeType: "video/mp4",
        uri: `opencut://jobs/${queued.jobId}/artifact`,
      });
      const metadata = await client.callTool({
        arguments: { jobId: queued.jobId },
        name: "job_get_status",
      });
      expect(
        (metadata.content as { type: string }[]).map((entry) => entry.type)
      ).toEqual(["text", "resource_link"]);
      const resource = await client.readResource({
        uri: complete.artifactResource?.uri ?? "",
      });
      const [content] = resource.contents;
      expect(content?.mimeType).toBe("video/mp4");
      const bytes =
        content && "blob" in content
          ? Buffer.from(content.blob, "base64")
          : Buffer.alloc(0);
      expect(bytes.length).toBe(complete.artifact?.sizeBytes);
      expect(bytes).toEqual(
        readFileSync(join(directory, complete.artifact?.relativePath ?? ""))
      );
    })
  );
  expect((await read()).project).toEqual(before.project);
  expect(persisted()).toEqual(diskBefore);
  const stale = await client.callTool({
    arguments: { endMs: 1000, expectedRevision: 0, projectId, startMs: 0 },
    name: "preview_review_range",
  });
  expect(stale.isError).toBe(true);
  const malformed = await client.callTool({
    arguments: {
      endMs: 1000,
      expectedRevision: 1,
      projectId,
      resolution: { width: 160 },
      startMs: 0,
    },
    name: "preview_review_range",
  });
  expect(malformed.isError).toBe(true);
  await call(
    "project_undo",
    { expectedRevision: 1, projectId },
    writeResultSchema
  );
  expect((await read()).project.tracks).toEqual(initial.project.tracks);
  await call(
    "project_redo",
    { expectedRevision: 2, projectId },
    writeResultSchema
  );
  expect((await read()).project.tracks).toEqual(before.project.tracks);
};
