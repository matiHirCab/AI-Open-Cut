import { jobSchema, schemas } from "../schemas";
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
        "Poll a process-local job and retrieve its result or structured failure.",
      inputSchema: schemas.jobGetStatus,
      outputSchema: jobSchema,
    },
    async ({ jobId }) => {
      try {
        return await previewJobResponse(dependencies, jobId);
      } catch (error) {
        return failure(error);
      }
    }
  );
};
