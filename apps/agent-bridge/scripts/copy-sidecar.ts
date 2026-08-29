import { existsSync } from "node:fs";
import { resolve } from "node:path";

import { assembleRuntimePackage } from "./package-runtime";

const executable =
  process.platform === "win32" ? "opencut-headless.exe" : "opencut-headless";
const source = resolve(
  import.meta.dirname,
  "../../../target/release",
  executable
);
const destinationDirectory = resolve(import.meta.dirname, "../dist");
const bridge = resolve(
  destinationDirectory,
  `opencut-agent-bridge${process.platform === "win32" ? ".exe" : ""}`
);
if (!existsSync(source)) {
  throw new Error(`Missing release headless executable: ${source}`);
}
if (!existsSync(bridge)) {
  throw new Error(`Missing compiled bridge executable: ${bridge}`);
}
await assembleRuntimePackage(destinationDirectory, {
  bridge,
  headless: source,
  transcriptionWorker: resolve(
    import.meta.dirname,
    "../../faster-whisper/worker.py"
  ),
  worker: resolve(import.meta.dirname, "../../kokoro-tts/worker.py"),
});
