import { runArtworkCli } from "./artwork/cli";
import { loadBridgeConfig } from "./config";
import { HeadlessClient } from "./headless";

const write = (stream: NodeJS.WritableStream, value: string): Promise<void> =>
  new Promise((resolve, reject) => {
    stream.write(value, (error) => (error ? reject(error) : resolve()));
  });
const controller = new AbortController();
const cancel = () => {
  controller.abort();
  process.stdin.destroy();
};
process.once("SIGINT", cancel);
process.once("SIGTERM", cancel);
let client: HeadlessClient | undefined;
try {
  const config = loadBridgeConfig();
  client = new HeadlessClient(config);
  process.exitCode = await runArtworkCli(
    process.stdin,
    config,
    client,
    (value) => write(process.stdout, value),
    (value) => write(process.stderr, value),
    controller.signal
  );
} catch {
  process.exitCode = 1;
  await write(
    process.stderr,
    `${JSON.stringify({ error: { code: "INVALID_CONFIGURATION", message: "Artwork CLI could not start", retryable: false }, ok: false, version: 1 })}\n`
  ).catch(() => {
    // Exit status already records failure if stderr is unavailable.
  });
} finally {
  process.stdin.destroy();
  process.removeListener("SIGINT", cancel);
  process.removeListener("SIGTERM", cancel);
  try {
    await client?.close();
  } catch {
    process.exitCode = 1;
    await write(
      process.stderr,
      `${JSON.stringify({ error: { code: "DEPENDENCY_UNAVAILABLE", message: "Artwork runtime cleanup failed; inspect the reported result and current project before retry", retryable: false }, ok: false, version: 1 })}\n`
    ).catch(() => {
      // Exit status remains unsuccessful if stderr is closed.
    });
  }
}
