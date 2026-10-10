import { spawnSync } from "node:child_process";
import { copyFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import {
  jobSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";

// Genuine source/default-package MCP exports; clocks are literal authored times.
export async function verifyOrdinaryAudioTiming(
  client: Client,
  ffmpeg: string,
  audio: string,
  exports: string,
  evidence: string,
  mode: string
) {
  const call = async <T>(
    name: string,
    input: Record<string, unknown>,
    schema: ZodType<T>
  ) => {
    const result = await client.callTool({ arguments: input, name });
    expect(result.isError, JSON.stringify(result)).not.toBe(true);
    return schema.parse(result.structuredContent);
  };
  const created = await call(
    "project_create",
    { fps: 10, height: 32, name: "Ordinary audio timing", width: 32 },
    writeResultSchema
  );
  const { projectId } = created;
  const read = () =>
    call("project_get_state", { projectId }, projectStateSchema);
  const track = (await read()).project.tracks.find(
    (value) => value.trackType === "audio"
  );
  if (!track) {
    throw new Error("Audio track missing");
  }
  const imported = await call(
    "asset_import",
    { expectedRevision: 0, mediaType: "audio", path: audio, projectId },
    writeResultSchema
  );
  const added = await call(
    "timeline_add_media",
    {
      assetId: imported.changedIds[0],
      durationMs: 500,
      expectedRevision: imported.revision,
      projectId,
      sourceInMs: 0,
      startMs: 2000,
      trackId: track.id,
    },
    writeResultSchema
  );
  const [itemId] = added.changedIds;
  const records: unknown[] = [];
  const terminal = async (
    jobId: string,
    attempts = 1200
  ): Promise<ReturnType<typeof jobSchema.parse>> => {
    const job = await call("job_get_status", { jobId }, jobSchema);
    if (["completed", "failed", "cancelled"].includes(job.status)) {
      return job;
    }
    if (attempts === 0) {
      throw new Error("Ordinary timing job exceeded two minutes");
    }
    await new Promise((done) => setTimeout(done, 100));
    return await terminal(jobId, attempts - 1);
  };
  const render = async (label: string, start: number) => {
    const before = await read();
    const relativePath = `${mode}-ordinary-${label}.mp4`;
    const submitted = await call(
      "project_export_video",
      {
        expectedRevision: before.project.revision,
        format: "mp4",
        overwrite: false,
        projectId,
        relativePath,
        resolution: "project",
      },
      jobSchema
    );
    const job = await terminal(submitted.jobId);
    expect(job.status, JSON.stringify(job.error)).toBe("completed");
    expect(await read()).toEqual(before);
    const path = join(exports, relativePath);
    const decoded = spawnSync(
      ffmpeg,
      [
        "-v",
        "error",
        "-i",
        path,
        "-map",
        "0:a:0",
        "-f",
        "f32le",
        "-ar",
        "48000",
        "-ac",
        "1",
        "pipe:1",
      ],
      { maxBuffer: 4 * 1024 * 1024 }
    );
    expect(decoded.status, decoded.stderr.toString()).toBe(0);
    const rms = (from: number, to: number) => {
      let sum = 0;
      for (let index = from * 48; index < to * 48; index += 1) {
        sum += decoded.stdout.readFloatLE(index * 4) ** 2;
      }
      return Math.sqrt(sum / ((to - from) * 48));
    };
    const silence = rms(100, 300),
      tone = rms(start + 100, start + 300);
    expect(silence).toBeLessThan(0.0001);
    expect(tone).toBeGreaterThan(0.04);
    copyFileSync(path, join(evidence, relativePath));
    records.push({
      job,
      label,
      silenceRms: silence,
      startMs: start,
      toneRms: tone,
    });
  };
  await render("absent", 2000);
  await call(
    "audio_track_route",
    {
      busId: "voiceover",
      expectedRevision: (await read()).project.revision,
      projectId,
      scope: "root",
      trackId: track.id,
    },
    writeResultSchema
  );
  await [0, -1].reduce(async (previous, gainDb) => {
    await previous;
    await call(
      "audio_bus_set_dsp",
      {
        busId: "voiceover",
        dsp: { compressor: null, eq: [], gainDb, pan: 0 },
        expectedRevision: (await read()).project.revision,
        projectId,
      },
      writeResultSchema
    );
    await render(gainDb === 0 ? "identity" : "gain", 2000);
  }, Promise.resolve());
  await call(
    "timeline_move_item",
    {
      expectedRevision: (await read()).project.revision,
      itemId,
      projectId,
      startMs: 1000,
      trackId: track.id,
    },
    writeResultSchema
  );
  await render("moved", 1000);
  await call(
    "project_undo",
    { expectedRevision: (await read()).project.revision, projectId },
    writeResultSchema
  );
  await render("undone", 2000);
  await call(
    "project_redo",
    { expectedRevision: (await read()).project.revision, projectId },
    writeResultSchema
  );
  await render("redone", 1000);
  const state = await read();
  expect(await call("project_open", { projectId }, projectStateSchema)).toEqual(
    state
  );
  await render("reopened", 1000);
  return { kind: "ordinary-audio-timing", projectId, records, state };
}
