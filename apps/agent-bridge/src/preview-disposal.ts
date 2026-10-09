import { lstat, realpath, unlink } from "node:fs/promises";
import { join, resolve } from "node:path";

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

// Disposable job outputs only. Never recursively remove project/media inventory.
export const previewDisposer =
  (projectsDirectory: string | undefined) => async (job: Job) => {
    const artifact = validateOutput(job);
    if (!projectsDirectory) {
      throw unsafe();
    }
    try {
      const root = resolve(projectsDirectory);
      const rootStat = await lstat(root);
      if (!rootStat.isDirectory() || rootStat.isSymbolicLink()) {
        throw unsafe();
      }
      const canonicalRoot = await realpath(root);
      const project = join(canonicalRoot, job.projectId);
      const previews = join(project, "previews");
      for (const directory of [project, previews]) {
        // biome-ignore lint/performance/noAwaitInLoops: Validate ancestors before descending.
        const stat = await lstat(directory);
        if (!stat.isDirectory() || stat.isSymbolicLink()) {
          throw unsafe();
        }
      }
      const path = join(project, artifact.relativePath);
      const stat = await lstat(path);
      if (
        !stat.isFile() ||
        stat.isSymbolicLink() ||
        (await realpath(path)) !== path
      ) {
        throw unsafe();
      }
      await unlink(path);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") {
        return;
      }
      throw unsafe();
    }
  };
