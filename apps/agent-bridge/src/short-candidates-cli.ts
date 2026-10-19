import { loadBridgeConfig } from "./config";
import { HeadlessClient } from "./headless";
import { planningFailure, runPlanningCli } from "./short-candidates/cli";
import { ShortCandidatePlanner } from "./short-candidates/service";

const write = (stream: NodeJS.WritableStream, value: string): Promise<void> =>
  new Promise((resolve, reject) => {
    const failed = (error: Error) => reject(error);
    stream.once("error", failed);
    stream.write(value, (error) => {
      if (error) {
        reject(error);
      } else {
        stream.removeListener("error", failed);
        resolve();
      }
    });
  });

let headless: HeadlessClient | undefined;
try {
  headless = new HeadlessClient(loadBridgeConfig());
  process.exitCode = await runPlanningCli(
    process.stdin,
    new ShortCandidatePlanner(headless),
    (value) => write(process.stdout, value),
    (value) => write(process.stderr, value)
  );
} catch (error) {
  process.exitCode = 1;
  await write(
    process.stderr,
    `${JSON.stringify(planningFailure(error))}\n`
  ).catch(() => {
    // stderr is unavailable; exitCode already records unsuccessful execution.
  });
} finally {
  await headless?.close();
}
