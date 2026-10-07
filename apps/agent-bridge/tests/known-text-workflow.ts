import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import {
  jobSchema,
  projectStateSchema,
  transcriptionEstimateSchema,
  transcriptionStatusSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;
export const verifyKnownTextWorkflow = async (client: Client, call: Call) => {
  const status = await call(
    "transcription_get_status",
    {},
    transcriptionStatusSchema
  );
  expect(status.knownTextAlignment).toMatchObject({
    phoneme: false,
    sentence: false,
    supported: true,
    word: true,
  });
  const { projectId } = await call(
    "project_create",
    { name: "Known-text alignment" },
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
    if (attempts > 0) {
      const job = await call("job_get_status", { jobId }, jobSchema);
      if (job.status === "completed") {
        return job;
      }
      if (job.status === "failed") {
        throw new Error(JSON.stringify(job.error));
      }
      await new Promise((resolve) => setTimeout(resolve, 10));
      return await wait(jobId, attempts - 1);
    }
    throw new Error("Known-text test job timed out");
  };
  const speech = await call(
    "tts_generate_and_insert",
    {
      expectedRevision: 0,
      projectId,
      startMs: 0,
      text: "Hello world",
      trackId: audio.id,
    },
    jobSchema
  );
  const generated = (await wait(speech.jobId)).result;
  if (!generated) {
    throw new Error("Generated speech missing");
  }
  const input = {
    assetId: generated.assetId,
    knownText: "Hello world",
    projectId,
  };
  const before = await read();
  await call("transcription_estimate", input, transcriptionEstimateSchema);
  const job = await call("transcription_preview", input, jobSchema);
  const preview = (await wait(job.jobId)).transcriptionPreview;
  if (!preview) {
    throw new Error("Known-text preview missing");
  }
  expect(preview.alignment).toMatchObject({
    providerId: "fake-transcriber",
    quality: "forced",
  });
  expect(preview.segments[0]?.text).toBe("Hello world");
  expect(JSON.stringify(preview)).not.toContain(".wav");
  expect(await read()).toEqual(before);
  const rejected = await client.callTool({
    arguments: { expectedRevision: 0, projectId, token: preview.token },
    name: "transcription_commit_preview",
  });
  expect(rejected.isError).toBe(true);
  expect(rejected.structuredContent).toMatchObject({
    error: { code: "REVISION_CONFLICT" },
  });
  expect(await read()).toEqual(before);
  const committed = await call(
    "transcription_commit_preview",
    { expectedRevision: generated.revision, projectId, token: preview.token },
    writeResultSchema
  );
  const aligned = await read();
  const captions = aligned.project.tracks
    .flatMap((track) => track.items)
    .filter((item) => item.type === "caption");
  expect(captions).toHaveLength(1);
  expect(captions[0]).toMatchObject({
    source: { assetId: generated.assetId },
    text: "Hello world",
  });
  const undone = await call(
    "project_undo",
    { expectedRevision: committed.revision, projectId },
    writeResultSchema
  );
  expect(
    (await read()).project.tracks
      .flatMap((track) => track.items)
      .filter((item) => item.type === "caption")
  ).toHaveLength(0);
  const redone = await call(
    "project_redo",
    { expectedRevision: undone.revision, projectId },
    writeResultSchema
  );
  expect(redone.revision).toBe(undone.revision + 1);
  expect(
    (await read()).project.tracks
      .flatMap((track) => track.items)
      .filter((item) => item.type === "caption")
  ).toEqual(captions);
  await call("project_open", { projectId }, projectStateSchema);
  expect(
    (await read()).project.tracks
      .flatMap((track) => track.items)
      .filter((item) => item.type === "caption")
  ).toEqual(captions);
};
