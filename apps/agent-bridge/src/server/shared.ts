import { readFile } from "node:fs/promises";
import { join } from "node:path";

import type { McpServer } from "@modelcontextprotocol/server";
import type { z } from "zod/v4";

import type { BridgeConfig } from "../config";
import { BridgeError, errorBody, type HeadlessClient } from "../headless";
import type { HeadlessRequest } from "../headless-contract";
import type { JobRegistry } from "../jobs";
import type { SpeechApplicationService } from "../speech";
import type { TranscriptionApplicationService } from "../transcription";

export const READ_ONLY = {
  destructiveHint: false,
  idempotentHint: true,
  openWorldHint: false,
  readOnlyHint: true,
} as const;
export const WRITE = {
  destructiveHint: false,
  idempotentHint: false,
  openWorldHint: false,
  readOnlyHint: false,
} as const;
export const DESTRUCTIVE = { ...WRITE, destructiveHint: true } as const;

export interface SessionState {
  activeProjectId: string | null;
}
export interface ServerDependencies {
  config: BridgeConfig;
  headless: HeadlessClient;
  jobs: JobRegistry;
  session: SessionState;
  speech: SpeechApplicationService;
  transcription: TranscriptionApplicationService;
}

export const success = (value: Record<string, unknown>) => ({
  content: [{ text: JSON.stringify(value), type: "text" as const }],
  structuredContent: value,
});
export const failure = (error: unknown) => {
  const body = errorBody(error);
  return {
    content: [{ text: JSON.stringify({ error: body }), type: "text" as const }],
    isError: true,
    structuredContent: { error: body },
  };
};
export const invoke = async <Output extends Record<string, unknown>>(
  headless: HeadlessClient,
  request: HeadlessRequest,
  schema: z.ZodType<Output>
) => success(await headless.call(request, schema));

export const previewJobResponse = async (
  dependencies: ServerDependencies,
  jobId: string
) => {
  const job = dependencies.jobs.get(jobId);
  if (job.status !== "completed" || job.kind !== "preview" || !job.artifact) {
    if (
      job.status === "completed" &&
      job.kind === "speech_preview" &&
      job.speechPreview
    ) {
      const preview = await dependencies.speech.previewAudio(
        job.speechPreview.token
      );
      const data = await readFile(preview.path);
      return {
        ...success(job),
        content: [
          { text: JSON.stringify(job), type: "text" as const },
          {
            data: data.toString("base64"),
            mimeType: preview.mimeType,
            type: "audio" as const,
          },
        ],
      };
    }
    return success(job);
  }
  if (!dependencies.config.projectsDirectory) {
    throw new BridgeError(
      "INTERNAL_ERROR",
      "Projects directory is unavailable"
    );
  }
  const data = await readFile(
    join(
      dependencies.config.projectsDirectory,
      job.projectId,
      job.artifact.relativePath
    )
  );
  return {
    ...success(job),
    content: [
      { text: JSON.stringify(job), type: "text" as const },
      {
        data: data.toString("base64"),
        mimeType: job.artifact.mimeType,
        type: "image" as const,
      },
    ],
  };
};

export type Server = McpServer;
