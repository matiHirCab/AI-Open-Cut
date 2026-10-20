import { loadBridgeConfig } from "./config";
import { BridgeError, HeadlessClient } from "./headless";
import { JobRuntimeGuard } from "./job-runtime";
import { JsonLineLogger } from "./logger";
import { runNarrationCli } from "./narration-blocks/cli";
import { closeNarrationSession } from "./narration-blocks/lifecycle";
import {
  NarrationBlocksWorkflow,
  narrationFailure,
} from "./narration-blocks/workflow";
import { SpeechApplicationService } from "./speech";
import { KokoroSpeechSynthesizer } from "./tts";

const write = (stream: NodeJS.WritableStream, text: string): Promise<void> =>
  new Promise((resolve, reject) => {
    const failed = (error: Error) => reject(error);
    stream.once("error", failed);
    stream.write(text, (error) => {
      stream.removeListener("error", failed);
      if (error) {
        reject(error);
      } else {
        resolve();
      }
    });
  });
const abort = new AbortController();
let speech: SpeechApplicationService | undefined;
let headless: HeadlessClient | undefined;
let workflow: NarrationBlocksWorkflow | undefined;
let guard: JobRuntimeGuard | undefined;
const interrupt = () => {
  abort.abort();
  speech?.cancel();
  process.stdin.destroy();
};
process.once("SIGINT", interrupt);
process.once("SIGTERM", interrupt);
try {
  if (process.argv.length > 2) {
    throw new BridgeError(
      "INVALID_ARGUMENT",
      "Use the narration CLI with bounded JSON-lines commands on stdin"
    );
  }
  const config = loadBridgeConfig();
  const logger = new JsonLineLogger(config.logLevel);
  if (config.speechProviderId !== "kokoro") {
    throw new BridgeError(
      "TTS_UNAVAILABLE",
      "Configured speech provider is unavailable"
    );
  }
  headless = new HeadlessClient(config, logger);
  guard = new JobRuntimeGuard(config.ttsWorkDirectory);
  guard.acquire();
  speech = new SpeechApplicationService(
    new KokoroSpeechSynthesizer(config, logger),
    headless,
    config.generatedArtifactTtlMs,
    Date.now,
    logger
  );
  workflow = new NarrationBlocksWorkflow(headless, speech);
  process.exitCode = await runNarrationCli(
    process.stdin,
    workflow,
    (value) => write(process.stdout, value),
    () => ({
      markNonCancellable: () => {
        /* Publication is awaited to settlement before shutdown. */
      },
      onProgress: () => {
        /* Results are emitted as bounded command responses. */
      },
      signal: abort.signal,
    })
  );
} catch (error) {
  process.exitCode = 1;
  await write(
    process.stderr,
    `${JSON.stringify({ error: narrationFailure(error) })}\n`
  ).catch(() => {
    /* A closed stderr cannot receive a fallback report. */
  });
} finally {
  // The input loop awaits its active command. Preserve core ownership until its
  // publication settles, then dispose previews before closing the core transport.
  const closed = await closeNarrationSession({
    guard,
    headless,
    speech,
    workflow,
  });
  if (!closed.cleanupConfirmed) {
    process.exitCode = 1;
    await write(
      process.stderr,
      `${JSON.stringify({ error: { code: "INTERNAL_ERROR", message: "Narration cleanup is unconfirmed; runtime admission remains held for inspection", retryable: false } })}\n`
    ).catch(() => {
      /* A closed stderr cannot receive a fallback report. */
    });
  }
  if (closed.publicationUnknown) {
    await write(
      process.stderr,
      `${JSON.stringify({ message: "Session closed with an unresolved narration publication; inspect the project before new work", publicationUnknown: true })}\n`
    ).catch(() => {
      /* A closed stderr cannot receive a fallback report. */
    });
  }

  process.removeListener("SIGINT", interrupt);
  process.removeListener("SIGTERM", interrupt);
  if (abort.signal.aborted) {
    process.exitCode = 1;
  }
}
