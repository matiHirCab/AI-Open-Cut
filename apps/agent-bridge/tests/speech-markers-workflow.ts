import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import contract from "../../../contracts/speech-alignment-markers-v1.json";
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
export const verifySpeechMarkerWorkflow = async (
  client: Client,
  call: Call
) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.subsystems.editor.capabilities).toContain(contract.capability);
  const { projectId } = await call(
    "project_create",
    { name: "Speech marker workflow" },
    writeResultSchema
  );
  const read = () =>
    call("project_get_state", { projectId }, projectStateSchema);
  const audio = (await read()).project.tracks.find(
    (track) => track.trackType === "audio"
  );
  if (!audio) {
    throw new Error("Audio track missing");
  }
  const wait = async (
    jobId: string,
    attempts = 300
  ): Promise<ReturnType<typeof jobSchema.parse>> => {
    const job = await call("job_get_status", { jobId }, jobSchema);
    if (job.status === "completed" || job.status === "failed") {
      return job;
    }
    if (attempts <= 0) {
      throw new Error("Speech marker workflow job timed out");
    }
    await new Promise((resolve) => setTimeout(resolve, 10));
    return await wait(jobId, attempts - 1);
  };
  // This explicitly fake worker has no timestamps: requested cues must fail atomically.
  const before = await read();
  const badJob = await call(
    "speech_generate_and_insert",
    {
      expectedRevision: 0,
      markerPolicy: { type: "all_word" },
      projectId,
      startMs: 1000,
      text: "marker source",
      trackId: audio.id,
    },
    jobSchema
  );
  const rejected = await wait(badJob.jobId);
  expect(rejected.status).toBe("failed");
  expect(rejected.error?.code).toBe("VALIDATION_FAILED");
  expect(await read()).toEqual(before);
  const job = await call(
    "tts_generate_and_insert",
    {
      expectedRevision: 0,
      projectId,
      startMs: 1000,
      text: "marker source",
      trackId: audio.id,
    },
    jobSchema
  );
  const generated = (await wait(job.jobId)).result;
  if (!generated) {
    throw new Error("Speech result missing");
  }
  // Canonical timing scaled into the explicitly fake 100-ms source.
  const alignment = structuredClone(contract.alignment);
  for (const segments of [alignment.words, alignment.sentences]) {
    for (const segment of segments) {
      segment.startMs /= 10;
      segment.endMs /= 10;
    }
  }
  const edit = {
    alignment,
    assetId: generated.assetId,
    markerPolicy: { indices: [2, 0], type: "selected_word" },
    operation: "speech_markers_generate",
    scope: "root",
    startMs: 1000,
  };
  const { operation: unusedOperation, ...input } = edit;
  expect(unusedOperation).toBe(contract.operation);
  const inserted = await read();
  const stale = await client.callTool({
    arguments: { ...input, expectedRevision: 0, projectId },
    name: "speech_markers_generate",
  });
  expect(stale.isError).toBe(true);
  expect(stale.structuredContent).toMatchObject({
    error: { code: "REVISION_CONFLICT" },
  });
  expect(await read()).toEqual(inserted);
  const markerEdit = await call(
    "speech_markers_generate",
    { ...input, expectedRevision: generated.revision, projectId },
    writeResultSchema
  );
  const marked = await read();
  expect(
    marked.project.markers.map(({ name, timeMs }) => ({ name, timeMs }))
  ).toEqual([
    { name: "EVERY", timeMs: 1010 },
    { name: "ONE", timeMs: 1070 },
  ]);
  expect(markerEdit.changedIds).toEqual(
    marked.project.markers.map((marker) => marker.id)
  );
  const invalid = await client.callTool({
    arguments: {
      expectedRevision: markerEdit.revision,
      operations: [
        edit,
        { markerId: "missing", operation: "marker_delete", scope: "root" },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(invalid.isError).toBe(true);
  expect(invalid.structuredContent).toMatchObject({
    error: { code: "ITEM_NOT_FOUND" },
  });
  expect(await read()).toEqual(marked);
  const undone = await call(
    "project_undo",
    { expectedRevision: markerEdit.revision, projectId },
    writeResultSchema
  );
  expect((await read()).project.markers).toHaveLength(0);
  await call(
    "project_redo",
    { expectedRevision: undone.revision, projectId },
    writeResultSchema
  );
  expect((await read()).project.markers).toEqual(marked.project.markers);
  await call("project_open", { projectId }, projectStateSchema);
  expect((await read()).project.markers).toEqual(marked.project.markers);
};
