import { constants } from "node:fs";
import { lstat, open, realpath } from "node:fs/promises";
import { isAbsolute, join, relative, resolve, sep } from "node:path";

import type { BridgeConfig } from "../config";
import { BridgeError } from "../headless";
import type { Job } from "../schemas";
import type { ServerDependencies } from "./shared";

export const ARTIFACT_RESOURCES_CAPABILITY = "artifact_resources_v2";
export const ARTIFACT_RESOURCE_TEMPLATE = "opencut://jobs/{jobId}/artifact";
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/u;
const RENDER_KINDS = new Set(["preview", "preview_range", "export"]);
const UNSAFE_PATH = /[\\:%]/u;
const unavailable = () =>
  new BridgeError("VALIDATION_FAILED", "Job artifact is unavailable or unsafe");

const safeRelativePath = (path: string) => {
  if (
    !path ||
    isAbsolute(path) ||
    UNSAFE_PATH.test(path) ||
    [...path].some(
      (char) => char.charCodeAt(0) < 32 || char.charCodeAt(0) === 127
    ) ||
    path.split("/").some((part) => !part || part === "." || part === "..")
  ) {
    throw unavailable();
  }
  return path;
};

export const artifactUri = (jobId: string) => {
  if (!UUID.test(jobId)) {
    throw new BridgeError("JOB_NOT_FOUND", "Job was not found");
  }
  return `opencut://jobs/${jobId}/artifact`;
};

export const jobWithArtifactResource = (job: Job): Job => {
  if (job.status !== "completed") {
    return job;
  }
  let mimeType: "image/png" | "audio/wav" | "video/mp4";
  let name: string;
  let sizeBytes: number | undefined;
  let { expiresAtMs } = job;
  if (job.kind === "speech_preview" && job.speechPreview) {
    mimeType = "audio/wav";
    name = "Speech preview";
    expiresAtMs = Math.min(
      expiresAtMs ?? Number.POSITIVE_INFINITY,
      job.speechPreview.expiresAtMs
    );
  } else if (job.artifact && RENDER_KINDS.has(job.kind)) {
    const path = safeRelativePath(job.artifact.relativePath);
    if (job.kind !== "export" && !path.startsWith("previews/")) {
      throw unavailable();
    }
    const artifactMime = job.artifact.mimeType;
    if (
      (job.kind === "preview" && artifactMime !== "image/png") ||
      (job.kind !== "preview" && artifactMime !== "video/mp4")
    ) {
      throw unavailable();
    }
    mimeType = job.kind === "preview" ? "image/png" : "video/mp4";
    name = job.kind === "export" ? "Video export" : "Render preview";
    ({ sizeBytes } = job.artifact);
  } else {
    return job;
  }
  return {
    ...job,
    artifactResource: {
      expiresAtMs,
      mimeType,
      name,
      ...(sizeBytes === undefined ? {} : { sizeBytes }),
      uri: artifactUri(job.jobId),
    },
  };
};

// Resource I/O confinement only: project/media semantics remain core-owned.
const readOwnedFile = async (root: string, path: string) => {
  const base = resolve(root);
  const suffix = relative(base, resolve(path));
  if (!suffix || isAbsolute(suffix)) {
    throw unavailable();
  }
  safeRelativePath(suffix.split(sep).join("/"));
  if ((await realpath(base)) !== base) {
    throw unavailable();
  }
  let current = base;
  const rootStat = await lstat(current);
  if (rootStat.isSymbolicLink() || !rootStat.isDirectory()) {
    throw unavailable();
  }
  const segments = suffix.split(sep);
  for (const [index, segment] of segments.entries()) {
    current = join(current, segment);
    // biome-ignore lint/performance/noAwaitInLoops: Validate each ancestor before descending to the next.
    const stat = await lstat(current);
    if (
      stat.isSymbolicLink() ||
      (index < segments.length - 1 ? !stat.isDirectory() : !stat.isFile())
    ) {
      throw unavailable();
    }
  }
  // biome-ignore lint/suspicious/noBitwiseOperators: Combine filesystem open flags to refuse following the final symlink.
  const file = await open(current, constants.O_RDONLY | constants.O_NOFOLLOW);
  try {
    const stat = await file.stat();
    const pathStat = await lstat(current);
    if (
      !stat.isFile() ||
      stat.ino !== pathStat.ino ||
      stat.dev !== pathStat.dev ||
      (await realpath(current)) !== current
    ) {
      throw unavailable();
    }
    return await file.readFile();
  } finally {
    await file.close();
  }
};

const renderLocation = (config: BridgeConfig, job: Job) => {
  if (!job.artifact) {
    throw unavailable();
  }
  const path = safeRelativePath(job.artifact.relativePath);
  if (job.kind === "export") {
    if (!config.exportsDirectory) {
      throw unavailable();
    }
    return {
      path: join(config.exportsDirectory, path),
      root: config.exportsDirectory,
    };
  }
  if (!(config.projectsDirectory && UUID.test(job.projectId))) {
    throw unavailable();
  }
  return {
    path: join(config.projectsDirectory, job.projectId, path),
    root: config.projectsDirectory,
  };
};

export const readJobArtifact = async (
  dependencies: ServerDependencies,
  job: Job
) => {
  const resource = jobWithArtifactResource(job).artifactResource;
  if (!resource) {
    throw unavailable();
  }
  try {
    if (job.kind === "speech_preview" && job.speechPreview) {
      const preview = await dependencies.speech.previewAudio(
        job.speechPreview.token
      );
      if (preview.mimeType !== "audio/wav") {
        throw unavailable();
      }
      const root = dependencies.config.generatedMediaDirectories.find(
        (candidate) => {
          const suffix = relative(resolve(candidate), resolve(preview.path));
          return (
            suffix && !isAbsolute(suffix) && !suffix.split(sep).includes("..")
          );
        }
      );
      if (!root) {
        throw unavailable();
      }
      return {
        data: await readOwnedFile(root, preview.path),
        mimeType: resource.mimeType,
      };
    }
    const location = renderLocation(dependencies.config, job);
    return {
      data: await readOwnedFile(location.root, location.path),
      mimeType: resource.mimeType,
    };
  } catch (error) {
    if (
      error instanceof BridgeError &&
      error.code === "GENERATED_ARTIFACT_NOT_FOUND"
    ) {
      // biome-ignore lint/style/useErrorCause: Do not expose provider paths or tokens through error causes.
      throw new BridgeError(
        "GENERATED_ARTIFACT_NOT_FOUND",
        "Generated speech artifact was not found or has expired"
      );
    }
    throw unavailable();
  }
};
