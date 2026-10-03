import { ProtocolError, ResourceTemplate } from "@modelcontextprotocol/server";
import { BridgeError, errorBody } from "../headless";
import { jobSchema, schemas } from "../schemas";
import {
  ARTIFACT_RESOURCE_TEMPLATE,
  artifactUri,
  readJobArtifact,
} from "./artifacts";
import {
  failure,
  previewJobResponse,
  READ_ONLY,
  type Server,
  type ServerDependencies,
  success,
  WRITE,
} from "./shared";

export const registerJobTools = (
  server: Server,
  dependencies: ServerDependencies
) => {
  server.registerResource(
    "job-artifact",
    new ResourceTemplate(ARTIFACT_RESOURCE_TEMPLATE, { list: undefined }),
    { description: "Explicitly retrieve a retained process-local job output." },
    async (uri, variables) => {
      try {
        const jobId = String(variables.jobId);
        if (uri.href !== artifactUri(jobId)) {
          throw new BridgeError("JOB_NOT_FOUND", "Job was not found");
        }
        const artifact = await readJobArtifact(
          dependencies,
          dependencies.jobs.get(jobId)
        );
        return {
          contents: [
            {
              blob: artifact.data.toString("base64"),
              mimeType: artifact.mimeType,
              uri: uri.href,
            },
          ],
        };
      } catch (error) {
        // biome-ignore lint/style/useErrorCause: Resource RPC errors must contain only the sanitized public error.
        throw new ProtocolError(-32_602, JSON.stringify(errorBody(error)));
      }
    }
  );
  server.registerTool(
    "job_cancel",
    {
      annotations: WRITE,
      description: "Cancel queued or running work before atomic commit begins.",
      inputSchema: schemas.jobCancel,
      outputSchema: jobSchema,
    },
    ({ jobId }) => {
      try {
        return success(dependencies.jobs.cancel(jobId));
      } catch (error) {
        return failure(error);
      }
    }
  );

  server.registerTool(
    "job_get_status",
    {
      annotations: READ_ONLY,
      description:
        "Poll metadata and artifact links. Set includeBinary=true for inline PNG/WAV previews.",
      inputSchema: schemas.jobGetStatus,
      outputSchema: jobSchema,
    },
    async ({ jobId, includeBinary }) => {
      try {
        return await previewJobResponse(dependencies, jobId, includeBinary);
      } catch (error) {
        return failure(error);
      }
    }
  );
};
