import { constants } from "node:fs";
import { lstat, open, realpath } from "node:fs/promises";
import { isAbsolute, join, relative, resolve, sep } from "node:path";
import { TextDecoder } from "node:util";

import type { BridgeConfig } from "../config";
import { BridgeError } from "../headless";
import { audioAnalysisArtifactSchema, type Job } from "../schemas";
import type { ServerDependencies } from "./shared";

export const ARTIFACT_RESOURCES_CAPABILITY = "artifact_resources_v2";
export const ARTIFACT_RESOURCE_TEMPLATE = "opencut://jobs/{jobId}/artifact";
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/u;
const RENDER_ARTIFACTS: Partial<
  Record<
    Job["kind"],
    { mimeType: "image/png" | "video/mp4" | "application/json"; name: string }
  >
> = {
  audio_analysis: { mimeType: "application/json", name: "Audio analysis" },
  export: { mimeType: "video/mp4", name: "Video export" },
  preview: { mimeType: "image/png", name: "Render preview" },
  preview_range: { mimeType: "video/mp4", name: "Render preview" },
};
const ANALYSIS_JSON_BYTES = 4 * 1024 * 1024;
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
  let mimeType: "image/png" | "audio/wav" | "video/mp4" | "application/json";
  let name: string;
  let sizeBytes: number | undefined;
  let { expiresAtMs } = job;
  const render = RENDER_ARTIFACTS[job.kind];
  if (job.kind === "speech_preview" && job.speechPreview) {
    mimeType = "audio/wav";
    name = "Speech preview";
    expiresAtMs = Math.min(
      expiresAtMs ?? Number.POSITIVE_INFINITY,
      job.speechPreview.expiresAtMs
    );
  } else if (job.artifact && render) {
    const path = safeRelativePath(job.artifact.relativePath);
    if (job.kind !== "export" && !path.startsWith("previews/")) {
      throw unavailable();
    }
    if (job.artifact.mimeType !== render.mimeType) {
      throw unavailable();
    }
    ({ mimeType, name } = render);
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
const readOwnedFile = async (
  root: string,
  path: string,
  byteLimit?: number
) => {
  const base = resolve(root);
  const suffix = relative(base, resolve(path));
  if (!suffix || isAbsolute(suffix)) {
    throw unavailable();
  }
  safeRelativePath(suffix.split(sep).join("/"));
  const rootStat = await lstat(base);
  if (rootStat.isSymbolicLink() || !rootStat.isDirectory()) {
    throw unavailable();
  }
  // Trusted parent aliases (for example macOS /var) may precede the owned root.
  // Traverse its canonical location while rejecting every symlink beneath it.
  const canonicalBase = await realpath(base);
  let current = canonicalBase;
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
  const file = await open(
    current,
    // biome-ignore lint/suspicious/noBitwiseOperators: Combine filesystem open flags to refuse following the final symlink where supported.
    constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0)
  );
  try {
    const stat = await file.stat();
    const pathStat = await lstat(current);
    if (
      !stat.isFile() ||
      stat.ino !== pathStat.ino ||
      stat.dev !== pathStat.dev ||
      relative(join(canonicalBase, ...segments), await realpath(current)) !== ""
    ) {
      throw unavailable();
    }
    if (byteLimit === undefined) {
      return await file.readFile();
    }
    // Bound the actual opened-handle read, including a byte that detects growth
    // beyond admission after stat. No unbounded readFile fallback for JSON.
    const bytes = Buffer.alloc(byteLimit + 1);
    let count = 0;
    while (count < bytes.length) {
      // biome-ignore lint/performance/noAwaitInLoops: Stream one bounded opened handle without racing concurrent reads.
      const chunk = await file.read(bytes, count, bytes.length - count, null);
      if (chunk.bytesRead === 0) {
        break;
      }
      count += chunk.bytesRead;
    }
    if (count > byteLimit) {
      throw unavailable();
    }
    return bytes.subarray(0, count);
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
    if (job.kind === "audio_analysis") {
      const data = await readOwnedFile(
        location.root,
        location.path,
        ANALYSIS_JSON_BYTES
      );
      const text = new TextDecoder("utf-8", { fatal: true }).decode(data);
      const analysis = audioAnalysisArtifactSchema.parse(JSON.parse(text));
      if (
        !job.audioAnalysis ||
        JSON.stringify(analysis.summary) !== JSON.stringify(job.audioAnalysis)
      ) {
        throw unavailable();
      }
      return { data, mimeType: resource.mimeType };
    }
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
