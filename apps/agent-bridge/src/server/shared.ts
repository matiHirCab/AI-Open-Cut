import type { McpServer } from "@modelcontextprotocol/server";
import type { z } from "zod/v4";

import type { BridgeConfig } from "../config";
import { errorBody, type HeadlessClient } from "../headless";
import type { HeadlessRequest } from "../headless-contract";
import type { JobRegistry } from "../jobs";
import type { SpeechApplicationService } from "../speech";
import type { TranscriptionApplicationService } from "../transcription";

import { jobWithArtifactResource, readJobArtifact } from "./artifacts";

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
  jobId: string,
  includeBinary = false
) => {
  const job = jobWithArtifactResource(dependencies.jobs.get(jobId));
  const response = success(job);
  const resource = job.artifactResource;
  if (!resource) {
    return response;
  }
  const link = {
    mimeType: resource.mimeType,
    name: resource.name,
    type: "resource_link" as const,
    uri: resource.uri,
    ...(resource.sizeBytes === undefined ? {} : { size: resource.sizeBytes }),
  };
  if (
    includeBinary &&
    (resource.mimeType === "image/png" || resource.mimeType === "audio/wav")
  ) {
    const artifact = await readJobArtifact(dependencies, job);
    return {
      ...response,
      content: [
        ...response.content,
        link,
        {
          data: artifact.data.toString("base64"),
          mimeType: artifact.mimeType,
          type:
            resource.mimeType === "image/png"
              ? ("image" as const)
              : ("audio" as const),
        },
      ],
    };
  }
  return { ...response, content: [...response.content, link] };
};

export type Server = McpServer;
