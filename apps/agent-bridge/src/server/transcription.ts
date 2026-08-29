import { z } from "zod/v4";

import {
  jobSchema,
  schemas,
  transcriptionEstimateSchema,
  transcriptionStatusSchema,
  writeResultSchema,
} from "../schemas";
import {
  failure,
  READ_ONLY,
  type Server,
  type ServerDependencies,
  success,
  WRITE,
} from "./shared";

export const registerTranscriptionTools = (
  server: Server,
  { jobs, transcription }: ServerDependencies
) => {
  server.registerTool(
    "transcription_get_status",
    {
      annotations: READ_ONLY,
      description: "Check local faster-whisper readiness and queue health.",
      inputSchema: schemas.transcriptionGetStatus,
      outputSchema: transcriptionStatusSchema,
    },
    async () => {
      try {
        return success(await transcription.status());
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "transcription_estimate",
    {
      annotations: READ_ONLY,
      description:
        "Estimate a zero-cost local transcription without exposing media paths.",
      inputSchema: schemas.transcriptionEstimate,
      outputSchema: transcriptionEstimateSchema,
    },
    async (input) => {
      try {
        return success(await transcription.estimate(input));
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "transcription_preview",
    {
      annotations: WRITE,
      description:
        "Queue transcription and retain an editable caption preview without changing the project.",
      inputSchema: schemas.transcriptionPreview,
      outputSchema: jobSchema,
    },
    (input) => {
      try {
        return success(
          jobs.startTask(
            "transcription_preview",
            input.projectId,
            0,
            async (context) => ({
              transcriptionPreview: await transcription.preview(input, context),
            })
          )
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "transcription_commit_preview",
    {
      annotations: WRITE,
      description:
        "Atomically commit accepted transcript segments as first-class captions.",
      inputSchema: schemas.transcriptionCommitPreview,
      outputSchema: writeResultSchema,
    },
    async (input) => {
      try {
        return success(await transcription.commitPreview(input));
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "transcription_discard_preview",
    {
      annotations: WRITE,
      description: "Discard a retained transcription preview.",
      inputSchema: schemas.transcriptionDiscardPreview,
      outputSchema: z
        .object({ discarded: z.literal(true), token: z.string() })
        .strict(),
    },
    async ({ token }) => {
      try {
        return success(await transcription.discardPreview(token));
      } catch (error) {
        return failure(error);
      }
    }
  );
};
