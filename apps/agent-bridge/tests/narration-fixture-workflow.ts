import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import recipe from "../../../contracts/narration-driven-fixture-v1.json";
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

export const verifyNarrationFixtureWorkflow = async (
  client: Client,
  call: Call
) => {
  const { projectId } = await call(
    "project_create",
    { name: "Synthetic narration transport" },
    writeResultSchema
  );
  const read = () =>
    call("project_get_state", { projectId }, projectStateSchema);
  const initial = await read();
  const audio = initial.project.tracks.find(
    (track) => track.trackType === "audio"
  );
  const visual = initial.project.tracks.find(
    (track) => track.trackType === "overlay"
  );
  if (!(audio && visual)) {
    throw new Error("Fixture tracks missing");
  }
  const job = await call(
    "speech_generate_and_insert",
    {
      expectedRevision: 0,
      projectId,
      startMs: 0,
      text: "synthetic transport source",
      trackId: audio.id,
    },
    jobSchema
  );
  const wait = async (
    attempts: number
  ): Promise<ReturnType<typeof jobSchema.parse>> => {
    const result = await call(
      "job_get_status",
      { jobId: job.jobId },
      jobSchema
    );
    if (result.status === "completed" || result.status === "failed") {
      return result;
    }
    if (attempts === 0) {
      throw new Error("Narration source job timed out");
    }
    await new Promise((resolve) => setTimeout(resolve, 10));
    return await wait(attempts - 1);
  };
  const completed = await wait(300);
  expect(completed.status).toBe("completed");
  const assetId = completed.result?.assetId;
  if (!assetId) {
    throw new Error("Fixture generated asset missing");
  }
  // The existing fake worker returns 100ms, without alignment. Keep its contract
  // unchanged; this transport proof scales the independent recipe into that source.
  const alignment = structuredClone(recipe.alignment);
  for (const segments of [
    alignment.sentences,
    alignment.words,
    alignment.phonemes,
  ]) {
    for (const segment of segments) {
      segment.startMs /= 100;
      segment.endMs /= 100;
    }
  }
  await call(
    "speech_markers_generate",
    {
      alignment,
      assetId,
      expectedRevision: 1,
      markerPolicy: { type: "sentence" },
      projectId,
      scope: "root",
      startMs: 0,
    },
    writeResultSchema
  );
  const operations: Record<string, unknown>[] = [
    {
      busId: "sfx",
      defaultGainDb: -6,
      event: "narration_accent",
      operation: "sound_event_register",
      variantAssetIds: [assetId],
      variantSeed: 0,
    },
  ];
  for (const [index, cue] of recipe.cues.entries()) {
    operations.push(
      {
        color: cue.color,
        durationMs: 4,
        height: 64,
        operation: "add_rectangle",
        resultAlias: `visual${index}`,
        startMs: 0,
        trackId: visual.id,
        transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
        width: 64,
      },
      {
        itemId: `@visual${index}`,
        operation: "apply_animation_preset",
        parameters: { ...recipe.visual.parameters, durationMs: 2 },
        presetId: "scalar_tween",
        presetVersion: 1,
      },
      {
        itemId: `@visual${index}`,
        operation: "set_item_start_time",
        scope: "root",
        time: { markerName: cue.name, offsetMs: 0, type: "marker" },
      },
      {
        at: { markerName: cue.name, offsetMs: 0, type: "marker" },
        durationMs: 3,
        event: "narration_accent",
        gainDb: -3,
        operation: "timeline_add_audio_event",
        resultAlias: `event${index}`,
        scope: "root",
        trackId: audio.id,
      }
    );
  }
  const created = await call(
    "timeline_batch_edit",
    { expectedRevision: 2, operations, projectId },
    writeResultSchema
  );
  const saved = await read();
  expect(
    saved.project.markers.map(({ name, timeMs }) => ({ name, timeMs }))
  ).toEqual(
    recipe.cues.map((cue) => ({ name: cue.name, timeMs: cue.timeMs / 100 }))
  );
  for (const [index, cue] of recipe.cues.entries()) {
    for (const alias of [`visual${index}`, `event${index}`]) {
      const item = saved.project.tracks
        .flatMap((track) => track.items)
        .find((candidate) => candidate.id === created.aliases[alias]);
      expect(item?.startMs).toBe(cue.timeMs / 100);
      expect(item?.startTime).toEqual({
        markerName: cue.name,
        offsetMs: 0,
        type: "marker",
      });
    }
  }
  const failed = await client.callTool({
    arguments: {
      expectedRevision: 3,
      operations: [
        {
          kind: "cue",
          markerId: saved.project.markers[0]?.id,
          name: "EVERY",
          operation: "marker_update",
          scope: "root",
          timeMs: 6,
        },
        { markerId: "missing", operation: "marker_delete", scope: "root" },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(failed.isError).toBe(true);
  expect(await read()).toEqual(saved);
  await call(
    "project_undo",
    { expectedRevision: 3, projectId },
    writeResultSchema
  );
  await call(
    "project_redo",
    { expectedRevision: 4, projectId },
    writeResultSchema
  );
  const reopened = await call(
    "project_open",
    { projectId },
    projectStateSchema
  );
  expect(reopened.project.tracks).toEqual(saved.project.tracks);
  expect(reopened.project.markers).toEqual(saved.project.markers);
  expect(reopened.project.assets).toEqual(saved.project.assets);
};
