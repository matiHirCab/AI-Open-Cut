import {
  type ProductionIntent,
  prepareProduction,
} from "../production-requests/prepare";
import { audioAnalysisResultSchema, jobSchema, schemas } from "../schemas";
import {
  DESTRUCTIVE,
  failure,
  type Server,
  type ServerDependencies,
  success,
  WRITE,
} from "./shared";

export const registerRenderTools = (
  server: Server,
  { headless, jobs }: ServerDependencies
) => {
  const queueProduction = async (intent: ProductionIntent) => {
    try {
      const prepared = await prepareProduction(headless, intent);
      return success(
        jobs.start(
          prepared.kind,
          intent.request.projectId,
          intent.request.expectedRevision,
          prepared.request
        )
      );
    } catch (error) {
      return failure(error);
    }
  };
  server.registerTool(
    "audio_analyze_mix",
    {
      annotations: WRITE,
      description:
        "Queue original48kHz stereo waveform, sample peak, true peak and LUFS analysis of a committed root range. Processing retains full root clocks; polling returns a bounded summary and an explicit JSON waveform resource.",
      inputSchema: schemas.audioAnalyzeMix,
      outputSchema: jobSchema,
    },
    ({ projectId, expectedRevision, startMs, endMs, waveformBins }) => {
      try {
        return success(
          jobs.startTask(
            "audio_analysis",
            projectId,
            expectedRevision,
            async ({ signal, onProgress }) => {
              const result = await headless.call(
                {
                  endMs,
                  expectedRevision,
                  operation: "analyze_audio",
                  projectId,
                  startMs,
                  waveformBins,
                },
                audioAnalysisResultSchema,
                { onProgress, signal }
              );
              return {
                artifact: result.artifact,
                audioAnalysis: result.summary,
              };
            }
          )
        );
      } catch (error) {
        return failure(error);
      }
    }
  );
  server.registerTool(
    "preview_review_range",
    {
      annotations: WRITE,
      description:
        "Queue an audio-enabled MP4 review using 540p, 720p, project, or custom dimensions. Set includeAudio=false for silent review.",
      inputSchema: schemas.previewReviewRange,
      outputSchema: jobSchema,
    },
    (request) => queueProduction({ kind: "review_range", request })
  );
  server.registerTool(
    "preview_render_frame",
    {
      annotations: WRITE,
      description: "Queue a PNG preview render without changing project state.",
      inputSchema: schemas.previewRenderFrame,
      outputSchema: jobSchema,
    },
    (request) => queueProduction({ kind: "preview_frame", request })
  );

  server.registerTool(
    "project_export_video",
    {
      annotations: DESTRUCTIVE,
      description:
        "Queue a local MP4 export for an immutable project revision.",
      inputSchema: schemas.projectExportVideo,
      outputSchema: jobSchema,
    },
    (request) => queueProduction({ kind: "export_video", request })
  );

  server.registerTool(
    "preview_render_range",
    {
      annotations: WRITE,
      description:
        "Queue a cancellable lightweight MP4 preview for an immutable project revision using final-export rendering behavior.",
      inputSchema: schemas.previewRenderRange,
      outputSchema: jobSchema,
    },
    (request) => queueProduction({ kind: "preview_range", request })
  );
};
