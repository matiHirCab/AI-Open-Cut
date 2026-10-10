import { spawn } from "node:child_process";
import type { BridgeConfig } from "./config";

import { BridgeError } from "./headless-events";
import type { Job } from "./schemas";

const UUID = "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}";
const PROJECT = new RegExp(`^${UUID}$`, "u");
const FRAME = new RegExp(`^previews/preview-${UUID}\\.png$`, "u");
const RANGE = new RegExp(`^previews/preview-range-${UUID}\\.mp4$`, "u");
const unsafe = () =>
  new BridgeError(
    "VALIDATION_FAILED",
    "Preview artifact disposal is unavailable or unsafe"
  );

const validateOutput = (job: Job) => {
  const { artifact } = job;
  const frame = job.kind === "preview";
  if (
    !(
      artifact &&
      PROJECT.test(job.projectId) &&
      (frame ? FRAME : RANGE).test(artifact.relativePath)
    ) ||
    artifact.mimeType !== (frame ? "image/png" : "video/mp4") ||
    !(frame || job.kind === "preview_range")
  ) {
    throw unsafe();
  }
  return artifact;
};

// Private bounded native adapter. It is outside the public headless/MCP protocol.
export const previewDisposer = (config: BridgeConfig) => async (job: Job) => {
  const artifact = validateOutput(job);
  if (!config.projectsDirectory) {
    throw unsafe();
  }
  await new Promise<void>((resolve, reject) => {
    const child = spawn(
      config.headlessPath,
      [...config.headlessArguments, "--dispose-owned-preview"],
      {
        env: {
          ...config.environment,
          OPENCUT_PROJECTS_DIR: config.projectsDirectory,
        },
        stdio: ["pipe", "pipe", "pipe"],
        windowsHide: true,
      }
    );
    let output = "";
    let failed = false;
    const fail = () => {
      failed = true;
      child.kill("SIGKILL");
    };
    const timer = setTimeout(
      fail,
      Math.min(config.headlessRequestTimeoutMs, 10_000)
    );
    child.on("error", () => {
      failed = true;
    });
    child.stdin.on("error", fail);
    child.stdout.on("data", (chunk: Buffer) => {
      if (output.length + chunk.length > 1024) {
        fail();
        return;
      }
      output += chunk.toString();
    });
    child.stderr.on("data", fail);
    child.once("close", (code) => {
      clearTimeout(timer);
      if (
        failed ||
        code !== 0 ||
        output.trim() !== '{"type":"result","result":{"disposed":true}}'
      ) {
        reject(unsafe());
      } else {
        resolve();
      }
    });
    child.stdin.end(
      JSON.stringify({
        projectId: job.projectId,
        relativePath: artifact.relativePath,
      })
    );
  });
};
