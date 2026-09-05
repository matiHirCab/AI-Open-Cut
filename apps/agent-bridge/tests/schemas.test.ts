import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";
import ERROR_CATALOG from "../../../contracts/error-codes-v1.json";
import {
  normalizeProviderErrorCode,
  publicDescriptionFor,
} from "../src/errors";
import { BridgeError, errorBody } from "../src/headless";
import { SERVER_INSTRUCTIONS } from "../src/instructions";
import {
  generatedAssetOriginSchema,
  jobSchema,
  projectStateSchema,
  schemas,
  speechVoiceListSchema,
  synthesizedSpeechMetadataSchema,
  ttsStatusSchema,
} from "../src/schemas";
import { DESTRUCTIVE, READ_ONLY, WRITE } from "../src/server/shared";

describe("MCP contracts", () => {
  it("requires schema-v7 common visual properties on returned items", () => {
    const transform = { opacity: 1, positionX: 0, positionY: 0, scale: 1 };
    const state = {
      durationMs: 100,
      project: {
        assets: [],
        components: [],
        createdAtMs: 1,
        id: "project",
        name: "Visual properties",
        revision: 0,
        schemaVersion: 12,
        settings: { fps: 30, height: 1080, width: 1920 },
        tracks: [
          {
            audioRole: "unassigned",
            ducking: null,
            hidden: false,
            id: "video",
            items: [
              {
                durationMs: 100,
                fromItemId: "source",
                hidden: false,
                id: "transition",
                stackOrder: 0,
                startMs: 0,
                toItemId: null,
                transform,
                transitionType: "fade",
                type: "transition",
                zIndex: 0,
              },
            ],
            locked: false,
            muted: false,
            name: "Video",
            trackType: "video",
          },
        ],
        updatedAtMs: 1,
      },
    };

    expect(projectStateSchema.safeParse(state).success).toBe(true);
    expect(
      projectStateSchema.safeParse({
        ...state,
        project: { ...state.project, schemaVersion: 6 },
      }).success
    ).toBe(false);
    const itemWithoutTransform = structuredClone(state);
    const transition = itemWithoutTransform.project.tracks.at(0)?.items.at(0);
    expect(transition).toBeDefined();
    if (!transition) {
      throw new Error("transition fixture is missing");
    }
    (transition as { transform?: unknown }).transform = undefined;
    expect(projectStateSchema.safeParse(itemWithoutTransform).success).toBe(
      false
    );
  });

  it("rejects unknown fields and invalid time ranges", () => {
    expect(
      schemas.projectCreate.safeParse({ name: "Intro", unexpected: true })
        .success
    ).toBe(false);
    expect(
      schemas.projectGetState.safeParse({
        projectId: "project",
        timeRange: { endMs: 1000, startMs: 1000 },
      }).success
    ).toBe(false);
  });

  it("validates keyframe values by property", () => {
    const result = schemas.timelineSetKeyframes.safeParse({
      expectedRevision: 2,
      itemId: "item",
      keyframes: [
        {
          easing: "linear",
          property: "position",
          timeMs: 0,
          value: { type: "scalar", value: 1 },
        },
      ],
      projectId: "project",
    });
    expect(result.success).toBe(false);
  });

  it("classifies read, write, and destructive tools", () => {
    expect(READ_ONLY).toMatchObject({
      destructiveHint: false,
      readOnlyHint: true,
    });
    expect(WRITE).toMatchObject({
      destructiveHint: false,
      readOnlyHint: false,
    });
    expect(DESTRUCTIVE).toMatchObject({
      destructiveHint: true,
      readOnlyHint: false,
    });
  });

  it("keeps the essential workflow inside the first 512 characters", () => {
    const prefix = SERVER_INSTRUCTIONS.slice(0, 512);
    for (const phrase of [
      "editor_get_status",
      "project_get_state",
      "expectedRevision",
      "REVISION_CONFLICT",
      "Preview",
      "Poll",
      "overwrite=true",
    ]) {
      expect(prefix).toContain(phrase);
    }
  });

  it("preserves stable core errors without leaking absolute paths", () => {
    const result = errorBody(
      new BridgeError(
        "PATH_NOT_ALLOWED",
        "C:\\Users\\person\\private\\clip.mov is outside the allowed roots"
      )
    );
    expect(result.code).toBe("PATH_NOT_ALLOWED");
    expect(result.message).not.toContain("C:\\Users\\person");
  });

  it("redacts spaced paths and preserves the diagnostic stderr tail", () => {
    const result = errorBody(
      new BridgeError("FFMPEG_FAILED", "render failed", false, undefined, {
        failedStage: "render",
        ffmpegExitCode: 1,
        ffmpegStderrExcerpt: `${"early ".repeat(900)}input=C:\\Users\\Jane Doe\\private clip.mov: Invalid argument\nfinal diagnostic`,
      })
    );
    expect(result.ffmpegStderrExcerpt).not.toContain("Jane Doe");
    expect(result.ffmpegStderrExcerpt).not.toContain("private clip.mov");
    expect(result.ffmpegStderrExcerpt).toContain("[path]: Invalid argument");
    expect(result.ffmpegStderrExcerpt?.endsWith("final diagnostic")).toBe(true);
    expect([...(result.ffmpegStderrExcerpt ?? "")].length).toBeLessThanOrEqual(
      4096
    );
  });

  it("derives retryability and provider mappings from the shared error catalog", () => {
    expect(new BridgeError("HEADLESS_TIMEOUT", "timeout").retryable).toBe(true);
    expect(new BridgeError("ASSET_IN_USE", "used").retryable).toBe(false);
    expect(normalizeProviderErrorCode("TTS_SYNTHESIS_FAILED")).toBe(
      "TTS_SYNTHESIS_FAILED"
    );
    expect(normalizeProviderErrorCode("PRIVATE_BACKEND_TRACE")).toBe(
      "TTS_PROVIDER_FAILED"
    );
    expect(publicDescriptionFor("TTS_PROVIDER_FAILED")).toBe(
      "tts provider failed"
    );
    expect(publicDescriptionFor("PRIVATE_BACKEND_TRACE")).not.toContain(
      "PRIVATE_BACKEND_TRACE"
    );
    expect(ERROR_CATALOG.version).toBe(1);
  });

  it("requires the explicit MVP export extension", () => {
    expect(
      schemas.projectExportVideo.safeParse({
        expectedRevision: 1,
        format: "mp4",
        overwrite: false,
        projectId: "project",
        relativePath: "movie.webm",
        resolution: "project",
      }).success
    ).toBe(false);
  });

  it("keeps the TTS request boundary provider-neutral", () => {
    expect(
      schemas.ttsGenerateAndInsert.safeParse({
        expectedRevision: 1,
        projectId: "project",
        startMs: 0,
        text: "Hello from OpenCut",
        trackId: "audio",
      }).success
    ).toBe(true);
    expect(
      schemas.ttsGenerateAndInsert.safeParse({
        expectedRevision: 1,
        language: "de-DE",
        projectId: "project",
        speed: 3,
        startMs: 0,
        text: "Hello",
        trackId: "audio",
        voice: "ef_dora",
      }).success
    ).toBe(true);
    expect(
      schemas.ttsGenerateAndInsert.safeParse({
        expectedRevision: 1,
        projectId: "project",
        startMs: 0,
        text: "Hello",
        trackId: "audio",
        voice: "",
      }).success
    ).toBe(false);
  });

  it("defines cancellation, retention, and artifact retry contracts", () => {
    expect(schemas.jobCancel.safeParse({ jobId: "job" }).success).toBe(true);
    expect(
      schemas.ttsCommitGeneratedArtifact.safeParse({
        artifactToken: "artifact",
        expectedRevision: 4,
      }).success
    ).toBe(true);
    expect(
      jobSchema.safeParse({
        createdAtMs: 1,
        error: {
          code: "REVISION_CONFLICT",
          message: "conflict",
          retryable: true,
        },
        expiresAtMs: 100,
        generatedArtifact: { expiresAtMs: 50, token: "artifact" },
        jobId: "job",
        kind: "tts",
        persistence: "process",
        progress: 0.7,
        projectId: "project",
        revision: 3,
        status: "failed",
        updatedAtMs: 2,
      }).success
    ).toBe(true);
  });

  it("parses the shared speech provider contract without catalog drift", () => {
    const contract = JSON.parse(
      readFileSync(
        resolve(
          import.meta.dirname,
          "../../../contracts/speech-provider-v1.json"
        ),
        "utf8"
      )
    ) as Record<string, unknown>;
    const status = ttsStatusSchema.parse(contract.status);
    const voices = speechVoiceListSchema.parse(contract.voices);
    const synthesis = synthesizedSpeechMetadataSchema.parse(contract.synthesis);
    const origin = generatedAssetOriginSchema.parse(contract.origin);

    expect(voices.map((voice) => voice.id)).toEqual(status.voices);
    expect(synthesis).toMatchObject({
      modelId: status.modelId,
      providerId: status.providerId,
      sampleRateHz: status.sampleRateHz,
      voiceId: status.defaultVoiceId,
    });
    expect(origin.generation).toMatchObject({
      modelId: synthesis.modelId,
      providerId: synthesis.providerId,
      sampleRateHz: synthesis.sampleRateHz,
    });
  });
});
