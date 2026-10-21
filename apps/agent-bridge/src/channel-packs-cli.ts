import { channelFailure, runChannelCli } from "./channel-packs/cli";
import { loadBridgeConfig } from "./config";
import { HeadlessClient } from "./headless";

const write = (stream: NodeJS.WritableStream, value: string): Promise<void> =>
  new Promise((resolve, reject) => {
    const failed = (error: Error) => reject(error);
    stream.once("error", failed);
    stream.write(value, (error) => {
      stream.removeListener("error", failed);
      if (error) {
        reject(error);
      } else {
        resolve();
      }
    });
  });

let headless: HeadlessClient | undefined;
let shutdown: Promise<void> | undefined;
let interrupted = false;
const requestShutdown = () => {
  interrupted = true;
  process.stdin.destroy();
  shutdown ??= Promise.resolve(headless?.close());
};
process.once("SIGINT", requestShutdown);
process.once("SIGTERM", requestShutdown);
try {
  headless = new HeadlessClient(loadBridgeConfig());
  process.exitCode = await runChannelCli(
    process.argv.slice(2),
    process.stdin,
    headless,
    (value) => write(process.stdout, value),
    (value) => write(process.stderr, value)
  );
} catch (error) {
  process.exitCode = 1;
  await write(
    process.stderr,
    `${JSON.stringify(channelFailure(error))}\n`
  ).catch(() => {
    // exitCode already reports failure if stderr is unavailable.
  });
} finally {
  // The active command above remains awaited: close() alone does not wait for
  // one-shot abort settlement. Referenced child handles keep shutdown alive.
  await shutdown;
  await headless?.close();
  process.removeListener("SIGINT", requestShutdown);
  process.removeListener("SIGTERM", requestShutdown);
  if (interrupted) {
    process.exitCode = 1;
  }
}
