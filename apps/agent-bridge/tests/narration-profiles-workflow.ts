import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import { type ZodType, z } from "zod/v4";
import contract from "../../../contracts/narration-profiles-v1.json";
import {
  jobSchema,
  narrationProfileSchema,
  projectStateSchema,
  resolvedNarrationRequestSchema,
  speechVoiceListResultSchema,
  ttsStatusSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;
export const verifyNarrationProfilesWorkflow = async (
  client: Client,
  call: Call
) => {
  const status = await call("tts_get_status", {}, ttsStatusSchema);
  const { voices } = await call(
    "speech_list_voices",
    {},
    speechVoiceListResultSchema
  );
  const voice = voices.find(
    (candidate) => candidate.available && candidate.id === status.defaultVoiceId
  );
  if (!voice) {
    throw new Error("Fake provider default voice missing");
  }
  const settings = {
    ...contract.settings,
    language: voice.language,
    name: "Shared explicit NP1 transport",
    speed: status.defaultSpeed,
    voiceId: voice.id,
  };
  const profile = await call(
    "narration_profile_create",
    { settings },
    narrationProfileSchema
  );
  const selection = { id: profile.id, version: profile.version };
  const application = {
    expectedVersion: 1,
    profileId: profile.id,
    text: "AI Open synthetic source",
  };
  const resolved = await call(
    "narration_profile_apply",
    { application },
    resolvedNarrationRequestSchema
  );
  expect(resolved.narrationProfile).toEqual({
    id: profile.id,
    name: settings.name,
    version: 1,
  });
  const projects = await Promise.all(
    ["Profile project A", "Profile project B"].map(async (name) => {
      const created = await call("project_create", { name }, writeResultSchema);
      const state = await call(
        "project_get_state",
        { projectId: created.projectId },
        projectStateSchema
      );
      const audio = state.project.tracks.find(
        (track) => track.trackType === "audio"
      );
      if (!audio) {
        throw new Error("Profile project audio track missing");
      }
      return { projectId: created.projectId, trackId: audio.id };
    })
  );
  const wait = async (
    jobId: string,
    attempts = 300
  ): Promise<z.infer<typeof jobSchema>> => {
    const job = await call("job_get_status", { jobId }, jobSchema);
    if (job.status === "completed" || job.status === "failed") {
      return job;
    }
    if (attempts === 0) {
      throw new Error("Narration profile workflow job timed out");
    }
    await new Promise((resolve) => setTimeout(resolve, 10));
    return await wait(jobId, attempts - 1);
  };
  await Promise.all(
    projects.map(async (project) => {
      const started = await call(
        "speech_generate_and_insert",
        {
          ...project,
          expectedRevision: 0,
          profile: selection,
          startMs: 0,
          text: application.text,
        },
        jobSchema
      );
      const generated = await wait(started.jobId);
      expect(generated.status, JSON.stringify(generated.error)).toBe(
        "completed"
      );
      const state = await call(
        "project_get_state",
        { projectId: project.projectId },
        projectStateSchema
      );
      const asset = state.project.assets.find(
        (candidate) => candidate.id === generated.result?.assetId
      );
      expect(asset?.origin).toMatchObject({
        generation: { request: resolved },
        type: "speech_synthesis",
      });
    })
  );
  const updatedSettings = {
    ...settings,
    textOptions: { ...settings.textOptions, pronunciations: [] },
  };
  const updated = await call(
    "narration_profile_update",
    {
      expectedVersion: 1,
      profileId: profile.id,
      settings: updatedSettings,
    },
    narrationProfileSchema
  );
  expect(updated.version).toBe(2);
  const stale = await client.callTool({
    arguments: { application },
    name: "narration_profile_apply",
  });
  expect(stale.isError).toBe(true);
  expect(JSON.stringify(stale)).toContain("REVISION_CONFLICT");
  const newRequest = await call(
    "narration_profile_apply",
    {
      application: {
        ...application,
        expectedVersion: 2,
        textOptions: { ...settings.textOptions, pronunciations: [] },
      },
    },
    resolvedNarrationRequestSchema
  );
  expect(newRequest.narrationProfile.version).toBe(2);
  expect(newRequest.textOptions.pronunciations).toEqual([]);
  await Promise.all(
    projects.map(async (project) => {
      const state = await call(
        "project_get_state",
        { projectId: project.projectId },
        projectStateSchema
      );
      expect(state.project.assets[0]?.origin).toMatchObject({
        generation: { request: resolved },
      });
    })
  );
  const [first] = projects;
  if (!first) {
    throw new Error("Profile project missing");
  }
  const oldState = await call(
    "project_get_state",
    { projectId: first.projectId },
    projectStateSchema
  );
  const item = oldState.project.tracks.find(
    (track) => track.id === first.trackId
  )?.items[0];
  if (!item) {
    throw new Error("Profile item missing");
  }
  const regenerate = await call(
    "speech_regenerate",
    {
      expectedRevision: oldState.project.revision,
      itemId: item.id,
      projectId: first.projectId,
    },
    jobSchema
  );
  const regenerated = await wait(regenerate.jobId);
  expect(regenerated.status, JSON.stringify(regenerated.error)).toBe(
    "completed"
  );
  const final = await call(
    "project_get_state",
    { projectId: first.projectId },
    projectStateSchema
  );
  expect(
    final.project.assets.find(
      (asset) => asset.id === regenerated.result?.assetId
    )?.origin
  ).toMatchObject({ generation: { request: resolved } });
  const unavailable = await call(
    "narration_profile_create",
    {
      settings: {
        ...settings,
        name: "Unavailable NP1 transport",
        voiceId: "synthetic-unavailable-voice",
      },
    },
    narrationProfileSchema
  );
  const rejected = await client.callTool({
    arguments: {
      application: {
        expectedVersion: 1,
        profileId: unavailable.id,
        text: "No inference",
      },
    },
    name: "narration_profile_apply",
  });
  expect(rejected.isError).toBe(true);
  expect(JSON.stringify(rejected)).toContain("VALIDATION_FAILED");
  const listed = await call(
    "narration_profile_list",
    {},
    z.object({ profiles: narrationProfileSchema.array() }).strict()
  );
  expect(
    listed.profiles.some(
      (candidate) => candidate.id === profile.id && candidate.version === 2
    )
  ).toBe(true);
};
