/** Test-only process barrier around the unchanged actual rendering backend. */

import { spawn } from "node:child_process";
import { appendFileSync, existsSync, writeFileSync } from "node:fs";
import { setTimeout } from "node:timers/promises";

const backend = process.env.OPENCUT_DURABLE_TEST_BACKEND;
if (!backend) {
  throw new Error("Missing controlled test backend");
}
const args = process.argv.slice(2);
if (args.includes("-i")) {
  const marker = process.env.OPENCUT_DURABLE_TEST_INVOCATIONS;
  if (marker) {
    appendFileSync(marker, "encode\n");
  }
  const gate = process.env.OPENCUT_DURABLE_TEST_GATE;
  const entered = process.env.OPENCUT_DURABLE_TEST_ENTERED;
  if (gate && entered && existsSync(gate)) {
    writeFileSync(entered, "entered");
    const wait = async (): Promise<void> => {
      if (!existsSync(gate)) {
        return;
      }
      await setTimeout(10);
      return wait();
    };
    await wait();
  }
}
const child = spawn(backend, args, { stdio: "inherit", windowsHide: true });
const code = await new Promise<number>((resolveCode, reject) => {
  child.once("error", reject);
  child.once("close", (status) => resolveCode(status ?? 1));
});
process.exit(code);
