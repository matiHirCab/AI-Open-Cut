import { z } from "zod/v4";

import {
  jobSchema,
  schemas,
  speechEstimateSchema,
  speechRegenerateResultSchema,
  speechVoiceListResultSchema,
  ttsResultSchema,
  ttsStatusSchema,
} from "../schemas";
import {
  failure,
  READ_ONLY,
  type Server,
  type ServerDependencies,
  success,
  WRITE,
} from "./shared";

export const registerSpeechTools = (
  server: Server,
  { jobs, speech }: ServerDependencies
) => {
  const startGeneration = (
    input: z.infer<(typeof schemas)["speechGenerateAndInsert"]>
  ) => {
    try {
      return success(
        jobs.startTask(
          "tts",
          input.projectId,
          input.expectedRevision,
          async (context) => ({
            result: await speech.generateAndInsert(input, context),
          })
        )
      );
    } catch (error) {
      return failure(error);
    }
  };
  server.registerTool(
    "speech_list_voices",
    {
      annotations: READ_ONLY,
      description: "List discoverable speech voices and current availability.",
      inputSchema: schemas.speechListVoices,
      outputSchema: speechVoiceListResultSchema,
    },
    async ({ language }) => {
      try {
        return success({ voices: await speech.listVoices(language) });
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "speech_estimate",
    {
      annotations: READ_ONLY,
      description:
        "Estimate local speech duration, queue, cost, and CPU/RAM requirements.",
      inputSchema: schemas.speechEstimate,
      outputSchema: speechEstimateSchema,
    },
    async ({ source }) => {
      try {
        return success(await speech.estimate(source));
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "speech_preview",
    {
      annotations: WRITE,
      description: "Queue a playable speech preview without editing a project.",
      inputSchema: schemas.speechPreview,
      outputSchema: jobSchema,
    },
    ({ source }) => {
      try {
        const projectId =
          source.type === "item" ? source.projectId : "speech-preview";
        return success(
          jobs.startTask("speech_preview", projectId, 0, async (context) => ({
            speechPreview: await speech.preview(source, context),
          }))
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "speech_commit_preview",
    {
      annotations: WRITE,
      description: "Insert or replace speech using an existing preview token.",
      inputSchema: schemas.speechCommitPreview,
      outputSchema: z.union([ttsResultSchema, speechRegenerateResultSchema]),
    },
    async ({ token, projectId, expectedRevision, placement }) => {
      try {
        return success(
          await speech.commitPreview(
            token,
            projectId,
            expectedRevision,
            placement
          )
        );
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "speech_discard_preview",
    {
      annotations: WRITE,
      description: "Discard a retained speech preview immediately.",
      inputSchema: schemas.speechDiscardPreview,
      outputSchema: z
        .object({ discarded: z.literal(true), token: z.string() })
        .strict(),
    },
    async ({ token }) => {
      try {
        return success(await speech.discardPreview(token));
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "speech_regenerate",
    {
      annotations: WRITE,
      description:
        "Regenerate an existing speech item in place using persisted intent.",
      inputSchema: schemas.speechRegenerate,
      outputSchema: jobSchema,
    },
    ({ projectId, expectedRevision, ...input }) => {
      try {
        return success(
          jobs.startTask(
            "speech_regenerate",
            projectId,
            expectedRevision,
            async (context) => ({
              result: await speech.regenerate(
                { ...input, expectedRevision, projectId },
                context
              ),
            })
          )
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "tts_get_status",
    {
      annotations: READ_ONLY,
      description:
        "Check local speech readiness, voices, limits, FIFO queue health, configured path diagnostics, and actionable startup errors.",
      inputSchema: schemas.ttsGetStatus,
      outputSchema: ttsStatusSchema,
    },
    async () => {
      try {
        return success(await speech.status());
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "speech_generate_and_insert",
    {
      annotations: WRITE,
      description:
        "Queue speech synthesis and atomically insert one generated audio item.",
      inputSchema: schemas.speechGenerateAndInsert,
      outputSchema: jobSchema,
    },
    startGeneration
  );

  server.registerTool(
    "tts_generate_and_insert",
    {
      annotations: WRITE,
      description:
        "Queue speech synthesis and atomically insert the generated audio.",
      inputSchema: schemas.ttsGenerateAndInsert,
      outputSchema: jobSchema,
    },
    startGeneration
  );

  server.registerTool(
    "tts_commit_generated_artifact",
    {
      annotations: WRITE,
      description:
        "Commit retained speech after a revision conflict without synthesizing again.",
      inputSchema: schemas.ttsCommitGeneratedArtifact,
      outputSchema: ttsResultSchema,
    },
    async ({ artifactToken, expectedRevision }) => {
      try {
        return success(
          await speech.commitGeneratedArtifact(artifactToken, expectedRevision)
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
};
