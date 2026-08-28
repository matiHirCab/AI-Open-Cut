import { spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import { mkdir, rm, statfs, writeFile } from "node:fs/promises";
import { join } from "node:path";

import type { BridgeConfig } from "./config";
import { errorBody, type HeadlessClient } from "./headless";
import { headlessStatusSchema } from "./schemas";
import type { SpeechApplicationService } from "./speech";
import type { TranscriptionApplicationService } from "./transcription";

export interface DoctorCheck {
  details?: Record<string, boolean | number | string | null>;
  id: string;
  message: string;
  status: "error" | "ok" | "warning";
}

export interface DoctorReport {
  checks: DoctorCheck[];
  ready: boolean;
  version: 1;
}

export interface DoctorPlatform {
  capture: typeof capture;
  freeBytes: (path: string) => Promise<number>;
  writable: (path: string) => Promise<void>;
}

const FIVE_GIB = 5 * 1024 * 1024 * 1024;

const capture = (command: string, arguments_: string[]) =>
  new Promise<{ code: number | null; stderr: string; stdout: string }>(
    (resolvePromise) => {
      const child = spawn(command, arguments_, {
        stdio: ["ignore", "pipe", "pipe"],
        windowsHide: true,
      });
      let stdout = "";
      let stderr = "";
      child.stdout.setEncoding("utf8");
      child.stderr.setEncoding("utf8");
      child.stdout.on("data", (chunk: string) => {
        stdout += chunk;
      });
      child.stderr.on("data", (chunk: string) => {
        stderr += chunk;
      });
      child.on("error", () => resolvePromise({ code: null, stderr, stdout }));
      child.on("close", (code) => resolvePromise({ code, stderr, stdout }));
    }
  );

const writableCheck = async (directory: string) => {
  await mkdir(directory, { recursive: true });
  const path = join(directory, `.opencut-doctor-${randomUUID()}.tmp`);
  await writeFile(path, "doctor");
  await rm(path);
};

const DEFAULT_PLATFORM: DoctorPlatform = {
  capture,
  freeBytes: async (path) => {
    const facts = await statfs(path);
    return Number(facts.bavail) * Number(facts.bsize);
  },
  writable: writableCheck,
};

const checkTranscription = async (
  transcription: TranscriptionApplicationService,
  config: BridgeConfig,
  platform: DoctorPlatform
): Promise<DoctorCheck> => {
  const work = join(config.configRoot, "local-data", "transcription", "work");
  const fixture = join(work, `.opencut-doctor-${randomUUID()}.wav`);
  try {
    const status = await transcription.status();
    if (!status.ready) {
      return {
        details: {
          modelCached: status.modelCached,
          modelLoaded: status.modelLoaded,
        },
        id: "transcription",
        message: "Transcription model is not prepared",
        status: "error",
      };
    }
    await mkdir(work, { recursive: true });
    const generated = await platform.capture(
      Object.hasOwn(config.environment, "OPENCUT_FFMPEG_PATH")
        ? config.environment.OPENCUT_FFMPEG_PATH
        : "ffmpeg",
      [
        "-f",
        "lavfi",
        "-i",
        "anullsrc=r=16000:cl=mono",
        "-t",
        "0.5",
        "-y",
        fixture,
      ]
    );
    if (generated.code !== 0) {
      throw new Error("fixture generation failed");
    }
    await transcription.doctorTranscribe(fixture, 500);
    return {
      details: {
        cleanupVerified: true,
        modelCached: status.modelCached,
        modelLoaded: status.modelLoaded,
        shortTranscription: true,
      },
      id: "transcription",
      message: "Short local transcription and cleanup succeeded",
      status: "ok",
    };
  } catch (error) {
    return {
      details: { code: errorBody(error).code },
      id: "transcription",
      message: "Transcription worker startup failed",
      status: "error",
    };
  } finally {
    await rm(fixture, { force: true }).catch(() => undefined);
  }
};

export const runDoctor = async (
  config: BridgeConfig,
  headless: HeadlessClient,
  speech: SpeechApplicationService,
  platform: DoctorPlatform = DEFAULT_PLATFORM,
  transcription?: TranscriptionApplicationService
): Promise<DoctorReport> => {
  const checks: DoctorCheck[] = [];
  const python = await platform.capture(config.ttsPythonPath, ["--version"]);
  const pythonVersion = `${python.stdout} ${python.stderr}`.trim();
  checks.push({
    details: { executableConfigured: true, version: pythonVersion || null },
    id: "python",
    message:
      python.code === 0
        ? "Configured Python executable is available"
        : "Configured Python executable could not be started",
    status: python.code === 0 ? "ok" : "error",
  });

  try {
    const freeBytes = await platform.freeBytes(config.configRoot);
    checks.push({
      details: { freeBytes },
      id: "disk",
      message:
        freeBytes < FIVE_GIB
          ? "Less than 5 GiB of free disk space is available"
          : "Free disk space is sufficient",
      status: freeBytes < FIVE_GIB ? "warning" : "ok",
    });
  } catch {
    checks.push({
      id: "disk",
      message: "Free disk space could not be determined",
      status: "error",
    });
  }

  const writableDirectories = [
    config.projectsDirectory ??
      join(config.configRoot, "local-data", "projects"),
    config.exportsDirectory ?? join(config.configRoot, "local-data", "exports"),
    ...config.generatedMediaDirectories,
  ];
  try {
    await Promise.all([...new Set(writableDirectories)].map(platform.writable));
    checks.push({
      details: { directories: new Set(writableDirectories).size },
      id: "writeability",
      message: "Project, export, and generated-media directories are writable",
      status: "ok",
    });
  } catch {
    checks.push({
      id: "writeability",
      message: "A required runtime directory is not writable",
      status: "error",
    });
  }

  try {
    const status = await headless.call(
      { operation: "status" },
      headlessStatusSchema
    );
    checks.push({
      details: { editorReady: status.subsystems.editor.ready },
      id: "editor",
      message: status.subsystems.editor.ready
        ? "Editor subsystem is ready"
        : "Editor subsystem is unavailable",
      status: status.subsystems.editor.ready ? "ok" : "error",
    });
    checks.push({
      details: { renderingReady: status.subsystems.rendering.ready },
      id: "rendering",
      message: status.subsystems.rendering.ready
        ? "FFmpeg and FFprobe are ready"
        : "FFmpeg or FFprobe is unavailable",
      status: status.subsystems.rendering.ready ? "ok" : "error",
    });
  } catch (error) {
    checks.push({
      details: { code: errorBody(error).code },
      id: "editor",
      message: "Headless diagnostics failed",
      status: "error",
    });
  }

  if (transcription) {
    checks.push(await checkTranscription(transcription, config, platform));
  }

  try {
    const status = await speech.status();
    checks.push({
      details: {
        active: status.queue.active,
        modelCached: status.modelCached,
        modelLoaded: status.modelLoaded,
        queued: status.queue.queued,
      },
      id: "speech",
      message: status.ready
        ? "Speech model marker and provider are ready"
        : "Speech model marker or provider is not ready",
      status: status.ready ? "ok" : "error",
    });
    if (status.ready) {
      const preview = await speech.preview(
        {
          language: status.defaultLanguage,
          text: "OpenCut doctor check.",
          type: "request",
          voice: status.defaultVoiceId,
        },
        {
          jobId: "doctor",
          markNonCancellable: () => undefined,
          onProgress: () => undefined,
          signal: new AbortController().signal,
        }
      );
      await speech.discardPreview(preview.token);
      checks.push({
        details: { cleanupVerified: true },
        id: "synthesis",
        message: "Short synthesis and cleanup succeeded",
        status: "ok",
      });
    }
  } catch (error) {
    checks.push({
      details: { cleanupVerified: false, code: errorBody(error).code },
      id: "synthesis",
      message: "Speech synthesis or cleanup failed",
      status: "error",
    });
  }

  return {
    checks,
    ready: checks.every((check) => check.status !== "error"),
    version: 1,
  };
};
