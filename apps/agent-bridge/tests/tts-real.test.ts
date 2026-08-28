import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { afterAll, beforeAll, expect, it } from "vitest";
import type { ZodType } from "zod/v4";

import {
  jobSchema,
  projectStateSchema,
  ttsStatusSchema,
  writeResultSchema,
} from "../src/schemas";

const enabled = process.env.OPENCUT_REAL_TTS === "1";
const realTts = enabled ? it : it.skip;
const root = mkdtempSync(join(tmpdir(), "opencut-real-tts-"));
const projects = join(root, "projects");
const media = join(root, "media");
const exportsDirectory = join(root, "exports");
for (const directory of [projects, media, exportsDirectory]) {
  mkdirSync(directory, { recursive: true });
}

const client = new Client({ name: "opencut-real-tts", version: "0.1.0" });
const bridgeExecutable =
  process.env.OPENCUT_TEST_BRIDGE_PATH ??
  resolve(
    import.meta.dirname,
    "../dist",
    process.platform === "win32"
      ? "opencut-agent-bridge.exe"
      : "opencut-agent-bridge"
  );
const transport = new StdioClientTransport({
  args: [],
  command: bridgeExecutable,
  cwd: resolve(import.meta.dirname, "../../.."),
  env: {
    ...Object.fromEntries(
      Object.entries(process.env).filter(
        (entry): entry is [string, string] => entry[1] !== undefined
      )
    ),
    OPENCUT_ALLOWED_MEDIA_DIRS: media,
    OPENCUT_EXPORTS_DIR: exportsDirectory,
    OPENCUT_HEADLESS_PATH: resolve(
      import.meta.dirname,
      "../../../target/release",
      process.platform === "win32" ? "opencut-headless.exe" : "opencut-headless"
    ),
    OPENCUT_PROJECTS_DIR: projects,
  },
  stderr: "inherit",
});

const call = async <Output>(
  name: string,
  arguments_: Record<string, unknown>,
  schema: ZodType<Output>
) => {
  const response = await client.callTool({ arguments: arguments_, name });
  if (response.isError) {
    throw new Error(JSON.stringify(response.structuredContent));
  }
  return schema.parse(response.structuredContent);
};

const waitForJob = async (
  jobId: string,
  attemptsRemaining = 600
): Promise<ReturnType<typeof jobSchema.parse>> => {
  const job = await call("job_get_status", { jobId }, jobSchema);
  if (job.status === "completed") {
    return job;
  }
  if (job.status === "failed") {
    throw new Error(JSON.stringify(job.error));
  }
  if (attemptsRemaining === 0) {
    throw new Error("real TTS job timed out");
  }
  await new Promise((resolvePromise) => setTimeout(resolvePromise, 250));
  return waitForJob(jobId, attemptsRemaining - 1);
};

beforeAll(async () => {
  if (enabled) {
    await client.connect(transport);
  }
});

afterAll(async () => {
  if (enabled) {
    await client.close();
  }
  rmSync(root, { force: true, recursive: true });
});

realTts(
  "generates offline Kokoro speech, inserts it, and exports audible media",
  async () => {
    const ttsStatus = await call("tts_get_status", {}, ttsStatusSchema);
    expect(ttsStatus).toMatchObject({
      device: "cpu",
      modelCached: true,
      providerId: "kokoro",
      ready: true,
      version: "0.9.4",
    });
    const created = await call(
      "project_create",
      { name: "Real Kokoro" },
      writeResultSchema
    );
    const initial = await call(
      "project_get_state",
      { projectId: created.projectId },
      projectStateSchema
    );
    const audioTrack = initial.project.tracks.find(
      (track) => track.trackType === "audio"
    );
    if (!audioTrack) {
      throw new Error("canonical audio track was not created");
    }
    const queued = await call(
      "tts_generate_and_insert",
      {
        expectedRevision: 0,
        projectId: created.projectId,
        startMs: 0,
        text: "OpenCut now generates speech locally with Kokoro.",
        trackId: audioTrack.id,
        voice: "af_heart",
      },
      jobSchema
    );
    const generated = await waitForJob(queued.jobId);
    expect(generated.result?.durationMs).toBeGreaterThan(0);
    const revision = generated.result?.revision;
    if (revision === undefined) {
      throw new Error("TTS job returned no revision");
    }
    const state = await call(
      "project_get_state",
      { projectId: created.projectId },
      projectStateSchema
    );
    const asset = state.project.assets.find(
      (candidate) => candidate.id === generated.result?.assetId
    );
    expect(asset?.mediaType).toBe("audio");
    expect(asset?.origin).toMatchObject({
      generation: {
        modelId: ttsStatus.modelId,
        modelVersion: ttsStatus.modelVersion,
        providerId: ttsStatus.providerId,
        request: {
          language: "en-US",
          speed: ttsStatus.defaultSpeed,
          text: "OpenCut now generates speech locally with Kokoro.",
          voiceId: "af_heart",
        },
        sampleRateHz: ttsStatus.sampleRateHz,
      },
      type: "speech_synthesis",
    });
    if (!asset) {
      throw new Error("generated asset was not persisted");
    }
    const wav = readFileSync(
      join(projects, created.projectId, asset.projectRelativePath)
    );
    expect(wav.subarray(0, 4).toString("ascii")).toBe("RIFF");
    expect(wav.subarray(8, 12).toString("ascii")).toBe("WAVE");
    expect(wav.readUInt16LE(22)).toBe(1);
    expect(wav.readUInt32LE(24)).toBe(24_000);

    const exportJob = await call(
      "project_export_video",
      {
        expectedRevision: revision,
        format: "mp4",
        overwrite: false,
        projectId: created.projectId,
        relativePath: "real-kokoro.mp4",
        resolution: "720p",
      },
      jobSchema
    );
    await waitForJob(exportJob.jobId);
    expect(existsSync(join(exportsDirectory, "real-kokoro.mp4"))).toBe(true);
  },
  180_000
);
