import { McpServer } from "@modelcontextprotocol/server";

import type { BridgeConfig } from "./config";
import type { HeadlessClient } from "./headless";
import { SERVER_INSTRUCTIONS } from "./instructions";
import type { JobRegistry } from "./jobs";
import {
  registerContextResources,
  registerWorkflowPrompts,
} from "./server/context";
import { registerDraftTools } from "./server/drafts";
import { registerJobTools } from "./server/jobs";
import { registerProjectTools } from "./server/projects";
import { registerRenderTools } from "./server/render";
import type { ServerDependencies } from "./server/shared";
import { registerSpeechTools } from "./server/speech";
import { registerTimelineTools } from "./server/timeline";
import { registerTranscriptionTools } from "./server/transcription";
import type { SpeechApplicationService } from "./speech";
import type { TranscriptionApplicationService } from "./transcription";

export const createServer = (
  speech: SpeechApplicationService,
  headless: HeadlessClient,
  jobs: JobRegistry,
  config: BridgeConfig,
  transcription: TranscriptionApplicationService
) => {
  const server = new McpServer(
    { name: "opencut-agent-bridge", version: "0.1.0" },
    { instructions: SERVER_INSTRUCTIONS }
  );
  const dependencies: ServerDependencies = {
    config,
    headless,
    jobs,
    session: { activeProjectId: null },
    speech,
    transcription,
  };

  registerProjectTools(server, dependencies);
  registerTimelineTools(server, dependencies);
  registerRenderTools(server, dependencies);
  registerSpeechTools(server, dependencies);
  registerJobTools(server, dependencies);
  registerDraftTools(server, dependencies);
  registerTranscriptionTools(server, dependencies);
  registerContextResources(server, dependencies);
  registerWorkflowPrompts(server);
  return server;
};
