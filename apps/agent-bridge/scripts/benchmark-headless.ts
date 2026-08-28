import { mkdir, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

import { loadBridgeConfig } from "../src/config";
import { HeadlessClient } from "../src/headless";
import { headlessStatusSchema } from "../src/schemas";

const root = await mkdtemp(join(tmpdir(), "opencut-headless-benchmark-"));
const directories = ["projects", "media", "exports", "model", "work"].map(
  (name) => join(root, name)
);
await Promise.all(directories.map(async (directory) => await mkdir(directory)));
const [projects, media, exportsDirectory, model, work] = directories as [
  string,
  string,
  string,
  string,
  string,
];
const suffix = process.platform === "win32" ? ".exe" : "";
const config = loadBridgeConfig({
  ...process.env,
  OPENCUT_ALLOWED_MEDIA_DIRS: media,
  OPENCUT_EXPORTS_DIR: exportsDirectory,
  OPENCUT_HEADLESS_PATH: resolve(
    import.meta.dirname,
    `../../../target/release/opencut-headless${suffix}`
  ),
  OPENCUT_KOKORO_MODEL_DIR: model,
  OPENCUT_PROJECTS_DIR: projects,
  OPENCUT_TTS_WORK_DIR: work,
});
const client = new HeadlessClient(config);
const samples: number[] = [];
try {
  for (let index = 0; index < 31; index += 1) {
    const started = performance.now();
    await client.call({ operation: "status" }, headlessStatusSchema);
    if (index > 0) {
      samples.push(performance.now() - started);
    }
  }
} finally {
  client.close();
  await rm(root, { force: true, recursive: true });
}
samples.sort((left, right) => left - right);
const percentile = (ratio: number) =>
  samples[
    Math.min(samples.length - 1, Math.ceil(samples.length * ratio) - 1)
  ] ?? 0;
process.stdout.write(
  `${JSON.stringify({
    architecture: "process-per-request-stdio",
    medianMs: Number(percentile(0.5).toFixed(2)),
    p95Ms: Number(percentile(0.95).toFixed(2)),
    samples: samples.length,
    thresholdsMs: { median: 100, p95: 250 },
  })}\n`
);
