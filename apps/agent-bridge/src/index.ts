import { spawn } from "node:child_process";

import { serveStdio } from "@modelcontextprotocol/server/stdio";
import type { z } from "zod/v4";

import { type BridgeConfig, loadBridgeConfig } from "./config";
import { runDoctor } from "./doctor";
import { BridgeError, errorBody, HeadlessClient } from "./headless";
import { serveHttp } from "./http";
import { JobRegistry } from "./jobs";
import { JsonLineLogger, type Logger } from "./logger";
import { headlessStatusSchema, type statusSchema } from "./schemas";
import { createServer } from "./server";
import { SpeechApplicationService, type SpeechSynthesizer } from "./speech";
import {
  FasterWhisperTranscriber,
  TranscriptionApplicationService,
} from "./transcription";
import { KokoroSpeechSynthesizer } from "./tts";

const createSpeechProvider = (
  config: BridgeConfig,
  logger: Logger
): SpeechSynthesizer => {
  const providerId = config.speechProviderId;
  if (providerId === "kokoro") {
    return new KokoroSpeechSynthesizer(config, logger);
  }
  throw new BridgeError(
    "TTS_UNAVAILABLE",
    `Unsupported speech provider configuration: ${providerId}`
  );
};

const run = async () => {
  const config = loadBridgeConfig();
  const logger = new JsonLineLogger(config.logLevel);
  const headless = new HeadlessClient(config, logger);
  if (process.argv.includes("--doctor")) {
    const speech = new SpeechApplicationService(
      createSpeechProvider(config, logger),
      headless,
      config.generatedArtifactTtlMs,
      Date.now,
      logger
    );
    const transcription = new TranscriptionApplicationService(
      new FasterWhisperTranscriber(config, logger),
      headless,
      config.generatedArtifactTtlMs
    );
    try {
      const report = await runDoctor(
        config,
        headless,
        speech,
        undefined,
        transcription
      );
      process.stdout.write(`${JSON.stringify(report)}\n`);
      if (!report.ready) {
        process.exitCode = 1;
      }
    } finally {
      headless.close();
      await speech.close();
      await transcription.close();
    }
    return;
  }
  if (process.argv.includes("--health")) {
    const status = await headless.call(
      { operation: "status" },
      headlessStatusSchema
    );
    let provider: SpeechSynthesizer | null = null;
    let speechSubsystem: z.infer<typeof statusSchema>["subsystems"]["speech"];
    let transcriptionSubsystem: z.infer<
      typeof statusSchema
    >["subsystems"]["transcription"];
    try {
      provider = createSpeechProvider(config, logger);
      const speech = new SpeechApplicationService(
        provider,
        headless,
        config.generatedArtifactTtlMs,
        Date.now,
        logger
      );
      const speechStatus = await speech.status();
      speechSubsystem = {
        capabilities: speechStatus.ready ? ["tts"] : [],
        error: null,
        modelId: speechStatus.modelId,
        providerId: speechStatus.providerId,
        queue: speechStatus.queue,
        ready: speechStatus.ready,
      };
      await speech.close();
    } catch (error) {
      await provider?.close().catch(() => undefined);
      speechSubsystem = {
        capabilities: [],
        error: errorBody(error),
        modelId: null,
        providerId: null,
        queue: null,
        ready: false,
      };
    }
    try {
      const transcription = new TranscriptionApplicationService(
        new FasterWhisperTranscriber(config, logger),
        headless,
        config.generatedArtifactTtlMs
      );
      const transcriptionStatus = await transcription.status();
      transcriptionSubsystem = {
        capabilities: transcriptionStatus.ready
          ? ["transcription", "captions"]
          : [],
        error: null,
        modelId: transcriptionStatus.modelId,
        providerId: transcriptionStatus.providerId,
        queue: transcriptionStatus.queue,
        ready: transcriptionStatus.ready,
      };
      await transcription.close();
    } catch (error) {
      transcriptionSubsystem = {
        capabilities: [],
        error: errorBody(error),
        modelId: null,
        providerId: null,
        queue: null,
        ready: false,
      };
    }
    process.stdout.write(
      `${JSON.stringify({
        ...status,
        activeProjectId: null,
        capabilities: [
          ...status.capabilities,
          ...(speechSubsystem.ready ? ["tts"] : []),
          ...(transcriptionSubsystem.ready
            ? ["transcription", "captions"]
            : []),
        ],
        subsystems: {
          ...status.subsystems,
          speech: speechSubsystem,
          transcription: transcriptionSubsystem,
        },
      })}\n`
    );
    return;
  }

  if (process.argv.includes("--with-desktop")) {
    const desktop = spawn("cargo", ["run", "-p", "opencut-desktop"], {
      cwd: new URL("../../..", import.meta.url),
      detached: true,
      stdio: ["ignore", "ignore", "inherit"],
      windowsHide: true,
    });
    desktop.unref();
  }

  const speech = new SpeechApplicationService(
    createSpeechProvider(config, logger),
    headless,
    config.generatedArtifactTtlMs,
    Date.now,
    logger
  );
  const jobs = new JobRegistry({
    headless,
    logger,
    maxCount: config.jobMaxCount,
    ttlMs: config.jobTtlMs,
  });
  const transcription = new TranscriptionApplicationService(
    new FasterWhisperTranscriber(config, logger),
    headless,
    config.generatedArtifactTtlMs
  );
  const handle =
    config.transport === "http"
      ? serveHttp(speech, transcription, headless, jobs, config, async () => {
          const [editor, speechState, transcriptionState] =
            await Promise.allSettled([
              headless.call({ operation: "status" }, headlessStatusSchema),
              speech.status(),
              transcription.status(),
            ]);
          return {
            ready: editor.status === "fulfilled",
            subsystems: {
              editor: { ready: editor.status === "fulfilled" },
              rendering: {
                ready:
                  editor.status === "fulfilled" &&
                  editor.value.subsystems.rendering.ready,
              },
              speech: {
                ready:
                  speechState.status === "fulfilled" && speechState.value.ready,
              },
              transcription: {
                ready:
                  transcriptionState.status === "fulfilled" &&
                  transcriptionState.value.ready,
              },
            },
            version:
              editor.status === "fulfilled" ? editor.value.version : "0.1.0",
          };
        })
      : serveStdio(() =>
          createServer(speech, headless, jobs, config, transcription)
        );
  let shutdownPromise: Promise<void> | undefined;
  const shutdown = () => {
    shutdownPromise ??= (async () => {
      await jobs.close();
      headless.close();
      await speech.close();
      await transcription.close();
      await handle.close();
    })();
    return shutdownPromise;
  };
  const requestShutdown = () => {
    shutdown()
      .then(() => {
        if (config.transport === "http") {
          process.exit(0);
        }
      })
      .catch((error: unknown) => {
        const message =
          error instanceof Error ? error.message : "shutdown failed";
        process.stderr.write(`${message}\n`);
      });
  };
  process.stdin.once("end", requestShutdown);
  process.once("SIGINT", requestShutdown);
  process.once("SIGTERM", requestShutdown);
};

run().catch((error: unknown) => {
  const message =
    error instanceof Error ? error.message : "OpenCut bridge failed";
  process.stderr.write(`${message}\n`);
  process.exitCode = 1;
});
