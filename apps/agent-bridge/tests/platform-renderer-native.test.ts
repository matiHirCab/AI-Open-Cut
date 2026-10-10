import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { afterEach, expect, it } from "vitest";
import type { ZodType } from "zod/v4";
import { verifyRuntimePackage } from "../scripts/package-runtime";
import {
  jobSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";
import { memoizedSdkValidator } from "./fixtures/memoized-sdk-validator";
import { verifyOrdinaryAudioTiming } from "./ordinary-audio-timing-workflow";
import { verifyPreviewDisposal } from "./preview-disposal-workflow";

const roots = new Set<string>();
const SSIM = /All:([0-9.]+)/u;
// The solid interior is independent of observed output. FFmpeg's YUV/RGB
// conversion can round channels; retain a three-level bound and a black control.
const goldPixel = (rgb: Buffer) => {
  const pixel = (100 * 192 + 180) * 3;
  return (
    rgb.length === 192 * 108 * 3 &&
    Math.abs((rgb[pixel] ?? -100) - 255) <= 3 &&
    Math.abs((rgb[pixel + 1] ?? -100) - 204) <= 3 &&
    (rgb[pixel + 2] ?? 100) <= 3
  );
};
const optional = [
  "audio_bus_dsp_v1",
  "audio_bus_ducking_v1",
  "audio_analysis_v1",
  "audio_master_normalization_v1",
];
afterEach(() => {
  for (const root of roots) {
    rmSync(root, { force: true, recursive: true });
  }
  roots.clear();
});
const required = (name: string) => {
  const value = process.env[name];
  if (!value) {
    throw new Error(`Required platform evidence input: ${name}`);
  }
  return value;
};
const sequence = <T>(
  values: readonly T[],
  action: (value: T) => Promise<void>
) =>
  values.reduce(
    (previous, value) => previous.then(() => action(value)),
    Promise.resolve()
  );
const projectContent = (value: unknown) =>
  Object.fromEntries(
    Object.entries(value as Record<string, unknown>).filter(
      ([key]) => !["revision", "updatedAtMs"].includes(key)
    )
  );
const files = (root: string) => {
  const result: Record<string, string> = {};
  const visit = (directory: string) => {
    for (const name of readdirSync(directory).sort()) {
      if ([".lock"].includes(name)) {
        continue;
      }
      const path = join(directory, name);
      if (statSync(path).isDirectory()) {
        visit(path);
      } else {
        result[path] = createHash("sha256")
          .update(readFileSync(path))
          .digest("hex");
      }
    }
  };
  visit(root);
  return result;
};
for (const mode of ["source", "packaged"] as const) {
  it.skipIf(process.env.OPENCUT_PLATFORM_REQUIRED !== "1")(
    `actual default ${mode} renderer availability, fallback and media`,
    async () => {
      const repository = resolve(import.meta.dirname, "../../..");
      const root = mkdtempSync(join(tmpdir(), "opencut-platform-evidence-"));
      roots.add(root);
      const media = join(root, "media"),
        projects = join(root, "projects"),
        exports = join(root, "exports"),
        work = join(root, "work");
      for (const path of [media, projects, exports, work]) {
        mkdirSync(path);
      }
      const ffmpeg = required("OPENCUT_FFMPEG_PATH"),
        ffprobe = required("OPENCUT_FFPROBE_PATH"),
        font = required("OPENCUT_TEST_FONT_PATH");
      expect(statSync(font).isFile()).toBe(true);
      const ownedFont = join(media, basename(font));
      for (const name of [
        "DejaVuSans.ttf",
        "DejaVuSans-Bold.ttf",
        "DejaVuSans-Oblique.ttf",
        "DejaVuSans-BoldOblique.ttf",
      ]) {
        copyFileSync(join(dirname(font), name), join(media, name));
      }
      const execute = (command: string, args: string[]) => {
        const result = spawnSync(command, args, {
          maxBuffer: 16 * 1024 * 1024,
        });
        expect(
          result.status,
          result.error?.message ?? result.stderr.toString()
        ).toBe(0);
        return result;
      };
      const audio = join(media, "tone.wav");
      execute(ffmpeg, [
        "-v",
        "error",
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=437:sample_rate=48000:duration=1",
        "-ac",
        "2",
        "-c:a",
        "pcm_s16le",
        audio,
      ]);
      const runtime = required("OPENCUT_PLATFORM_PACKAGE");
      await verifyRuntimePackage(runtime);
      const suffix = process.platform === "win32" ? ".exe" : "";
      await verifyPreviewDisposal(
        mode === "packaged"
          ? join(runtime, `opencut-headless${suffix}`)
          : required("OPENCUT_PLATFORM_SOURCE_HEADLESS")
      );
      const environment = {
        ...Object.fromEntries(
          Object.entries(process.env).filter(
            (entry): entry is [string, string] => entry[1] !== undefined
          )
        ),
        OPENCUT_ALLOWED_FONT_DIRS: media,
        OPENCUT_ALLOWED_MEDIA_DIRS: media,
        OPENCUT_DEFAULT_FONT_PATH: ownedFont,
        OPENCUT_EXPORTS_DIR: exports,
        OPENCUT_FFMPEG_PATH: ffmpeg,
        OPENCUT_FFPROBE_PATH: ffprobe,
        OPENCUT_HEADLESS_PATH:
          mode === "packaged"
            ? join(runtime, `opencut-headless${suffix}`)
            : required("OPENCUT_PLATFORM_SOURCE_HEADLESS"),
        OPENCUT_KOKORO_MODEL_DIR: join(root, "absent-model"),
        OPENCUT_KOKORO_PYTHON: process.env.OPENCUT_TEST_PYTHON ?? "python",
        OPENCUT_KOKORO_WORKER:
          mode === "packaged"
            ? join(runtime, "kokoro-tts/worker.py")
            : join(repository, "apps/kokoro-tts/worker.py"),
        OPENCUT_PROJECTS_DIR: projects,
        OPENCUT_TRANSCRIPTION_MODEL_DIR: join(
          root,
          "absent-transcription-model"
        ),
        OPENCUT_TRANSCRIPTION_PYTHON:
          process.env.OPENCUT_TEST_PYTHON ?? "python",
        OPENCUT_TRANSCRIPTION_WORKER:
          mode === "packaged"
            ? join(runtime, "faster-whisper/worker.py")
            : join(repository, "apps/faster-whisper/worker.py"),
        OPENCUT_TTS_WORK_DIR: work,
      };
      const validator = memoizedSdkValidator();
      const connect = async (overrides: Record<string, string> = {}) => {
        const next = new Client(
          { name: "platform-runtime", version: "1" },
          { jsonSchemaValidator: validator }
        );
        await next.connect(
          new StdioClientTransport({
            args:
              mode === "packaged"
                ? []
                : ["run", join(repository, "apps/agent-bridge/src/index.ts")],
            command:
              mode === "packaged"
                ? join(runtime, `opencut-agent-bridge${suffix}`)
                : "bun",
            cwd: root,
            env: { ...environment, ...overrides },
            stderr: "inherit",
          })
        );
        return next;
      };
      let client = await connect();
      const call = async <T>(
        name: string,
        input: Record<string, unknown>,
        schema: ZodType<T>
      ) => {
        const result = await client.callTool({ arguments: input, name });
        expect(
          result.isError,
          JSON.stringify(result.structuredContent ?? result.content)
        ).not.toBe(true);
        return schema.parse(result.structuredContent);
      };
      const terminal = async (
        jobId: string,
        attempts = 1200
      ): Promise<ReturnType<typeof jobSchema.parse>> => {
        const job = await call("job_get_status", { jobId }, jobSchema);
        if (["completed", "failed", "cancelled"].includes(job.status)) {
          return job;
        }
        if (attempts === 0) {
          throw new Error("Platform native job exceeded two minute allowance");
        }
        await new Promise((done) => setTimeout(done, 100));
        return await terminal(jobId, attempts - 1);
      };
      const records: unknown[] = [];
      try {
        const status = await call("editor_get_status", {}, statusSchema);
        expect(status.subsystems.editor.ready).toBe(true);
        expect(status.subsystems.rendering.ready).toBe(true);
        for (const capability of optional) {
          expect(status.subsystems.rendering.capabilities).toContain(
            capability
          );
        }
        records.push({ kind: "healthy", status });
        const { projectId } = await call(
          "project_create",
          { fps: 10, height: 108, name: "Portable renderer", width: 192 },
          writeResultSchema
        );
        const read = () =>
          call("project_get_state", { projectId }, projectStateSchema);
        const initial = await read();
        const audioTrack = initial.project.tracks.find(
          (track) => track.trackType === "audio"
        );
        const visualTrack = initial.project.tracks.find(
          (track) => track.trackType === "overlay"
        );
        if (!(audioTrack && visualTrack)) {
          throw new Error("Default tracks missing");
        }
        const imported = await call(
          "asset_import",
          { expectedRevision: 0, mediaType: "audio", path: audio, projectId },
          writeResultSchema
        );
        const [assetId] = imported.changedIds;
        if (!assetId) {
          throw new Error("Imported audio missing");
        }
        await call(
          "timeline_add_media",
          {
            assetId,
            durationMs: 1000,
            expectedRevision: imported.revision,
            projectId,
            startMs: 0,
            trackId: audioTrack.id,
          },
          writeResultSchema
        );
        await call(
          "timeline_add_rectangle",
          {
            color: "#ffcc00",
            durationMs: 1000,
            expectedRevision: (await read()).project.revision,
            height: 108,
            projectId,
            startMs: 0,
            trackId: visualTrack.id,
            width: 192,
          },
          writeResultSchema
        );
        await call(
          "timeline_add_text",
          {
            durationMs: 1000,
            expectedRevision: (await read()).project.revision,
            fontPath: ownedFont,
            fontSize: 16,
            projectId,
            startMs: 0,
            text: "Portable",
            trackId: visualTrack.id,
          },
          writeResultSchema
        );
        let saved = await read();
        const directory = join(projects, projectId);
        const before = files(directory);
        await sequence(
          [
            [
              "timeline_add_media",
              {
                assetId: "missing-asset",
                durationMs: 1000,
                expectedRevision: saved.project.revision,
                projectId,
                startMs: 0,
                trackId: audioTrack.id,
              },
              "ASSET_NOT_FOUND",
            ],
            [
              "timeline_add_text",
              {
                durationMs: 1000,
                expectedRevision: saved.project.revision - 1,
                projectId,
                startMs: 0,
                text: "stale",
                trackId: visualTrack.id,
              },
              "REVISION_CONFLICT",
            ],
          ] as const,
          async ([name, input, code]) => {
            const result = await client.callTool({ arguments: input, name });
            expect(result.isError).toBe(true);
            expect(result.structuredContent).toMatchObject({ error: { code } });
            expect(await read()).toEqual(saved);
            expect(files(directory)).toEqual(before);
          }
        );
        const invalid = await client.callTool({
          arguments: {
            expectedRevision: saved.project.revision,
            projectId,
            timeMs: -1,
          },
          name: "preview_render_frame",
        });
        expect(invalid.isError).toBe(true);
        expect(await read()).toEqual(saved);
        expect(files(directory)).toEqual(before);
        const render = async (name: string, input: Record<string, unknown>) => {
          const job = await terminal(
            (
              await call(
                name,
                {
                  expectedRevision: saved.project.revision,
                  projectId,
                  ...input,
                },
                jobSchema
              )
            ).jobId
          );
          expect(job.status, JSON.stringify(job.error)).toBe("completed");
          expect(job.revision).toBe(saved.project.revision);
          if (!job.artifact) {
            throw new Error("Missing native artifact");
          }
          const path = join(
            job.kind === "export" ? exports : directory,
            job.artifact.relativePath
          );
          expect(statSync(path).size).toBeGreaterThan(0);
          records.push({ input, job, name });
          return path;
        };
        const frame = await render("preview_render_frame", { timeMs: 500 });
        const originalFrame = readFileSync(frame);
        const rgb = execute(ffmpeg, [
          "-v",
          "error",
          "-i",
          frame,
          "-f",
          "rawvideo",
          "-pix_fmt",
          "rgb24",
          "-",
        ]).stdout;
        expect(rgb.length).toBe(192 * 108 * 3);
        expect(goldPixel(rgb)).toBe(true);
        expect(goldPixel(Buffer.alloc(rgb.length))).toBe(false);
        const range = await render("preview_review_range", {
          endMs: 1000,
          resolution: "project",
          startMs: 0,
        });
        const exported = await render("project_export_video", {
          format: "mp4",
          overwrite: false,
          relativePath: "portable.mp4",
          resolution: "project",
        });
        const probes = [range, exported].map(
          (path) =>
            JSON.parse(
              execute(ffprobe, [
                "-v",
                "error",
                "-show_streams",
                "-show_format",
                "-of",
                "json",
                path,
              ]).stdout.toString()
            ) as {
              streams: {
                codec_type: string;
                width?: number;
                height?: number;
                nb_frames?: string;
                sample_rate?: string;
                channels?: number;
              }[];
              format: { duration: string };
            }
        );
        for (const probe of probes) {
          expect(
            probe.streams.find((stream) => stream.codec_type === "video")
          ).toMatchObject({ height: 108, nb_frames: "10", width: 192 });
          expect(
            probe.streams.find((stream) => stream.codec_type === "audio")
          ).toMatchObject({ channels: 2, sample_rate: "48000" });
          expect(
            Math.abs(Number(probe.format.duration) - 1)
          ).toBeLessThanOrEqual(0.1);
        }
        const ssimOutput = execute(ffmpeg, [
          "-i",
          range,
          "-i",
          exported,
          "-lavfi",
          "[0:v][1:v]ssim",
          "-f",
          "null",
          "-",
        ]).stderr.toString();
        const ssim = Number(SSIM.exec(ssimOutput)?.[1]);
        expect(ssim).toBeGreaterThanOrEqual(0.99);
        const decode = (path: string) =>
          execute(ffmpeg, [
            "-v",
            "error",
            "-i",
            path,
            "-vn",
            "-ac",
            "2",
            "-ar",
            "48000",
            "-f",
            "f32le",
            "-",
          ]).stdout;
        const left = decode(range),
          right = decode(exported);
        expect(left.length).toBe(right.length);
        expect(left.length).toBeGreaterThanOrEqual(48_000 * 2 * 4);
        let squared = 0,
          energy = 0;
        for (let offset = 0; offset < left.length; offset += 4) {
          const sample = left.readFloatLE(offset),
            delta = sample - right.readFloatLE(offset);
          expect(Number.isFinite(sample)).toBe(true);
          squared += delta * delta;
          energy += sample * sample;
        }
        const rms = Math.sqrt(squared / (left.length / 4));
        expect(rms).toBeLessThanOrEqual(0.0001);
        expect(Math.sqrt(energy / (left.length / 4))).toBeGreaterThan(0.005);
        records.push({ kind: "media", pcmRms: rms, probes, ssim });
        const evidence = required("OPENCUT_PLATFORM_EVIDENCE_DIR");
        mkdirSync(evidence, { recursive: true });
        copyFileSync(frame, join(evidence, `${mode}-frame.png`));
        copyFileSync(exported, join(evidence, `${mode}-export.mp4`));
        const timingEvidence = await verifyOrdinaryAudioTiming(
          client,
          ffmpeg,
          audio,
          exports,
          evidence,
          mode
        );
        records.push(timingEvidence);
        await call(
          "project_undo",
          { expectedRevision: saved.project.revision, projectId },
          writeResultSchema
        );
        expect(projectContent((await read()).project)).not.toEqual(
          projectContent(saved.project)
        );
        await call(
          "project_redo",
          { expectedRevision: (await read()).project.revision, projectId },
          writeResultSchema
        );
        expect(projectContent((await read()).project)).toEqual(
          projectContent(saved.project)
        );
        saved = await read();
        await client.close();
        validator.clear();
        client = await connect();
        expect(
          await call(
            "project_open",
            { projectId: timingEvidence.projectId },
            projectStateSchema
          )
        ).toEqual(timingEvidence.state);
        expect(
          await call("project_open", { projectId }, projectStateSchema)
        ).toEqual(saved);
        const reopened = await render("preview_render_frame", { timeMs: 500 });
        expect(readFileSync(reopened)).toEqual(originalFrame);
        await sequence(
          [
            "ffmpeg-missing",
            "ffprobe-missing",
            "base-missing",
            "delay-missing",
            "optional-missing",
          ] as const,
          async (failure) => {
            await client.close();
            validator.clear();
            const overrides: Record<string, string> = {};
            if (failure === "ffmpeg-missing") {
              overrides.OPENCUT_FFMPEG_PATH = join(root, "missing-ffmpeg");
            } else if (failure === "ffprobe-missing") {
              overrides.OPENCUT_FFPROBE_PATH = join(root, "missing-ffprobe");
            } else {
              overrides.OPENCUT_FFMPEG_PATH = required(
                "OPENCUT_PLATFORM_FILTER_FIXTURE"
              );
              overrides.OPENCUT_PLATFORM_FILTER_MODE = failure;
              overrides.OPENCUT_PLATFORM_REAL_FFMPEG = ffmpeg;
            }
            client = await connect(overrides);
            const unavailable = await call(
              "editor_get_status",
              {},
              statusSchema
            );
            expect(unavailable.ready).toBe(true);
            expect(unavailable.subsystems.editor.ready).toBe(true);
            expect(unavailable.subsystems.editor.capabilities).toEqual(
              status.subsystems.editor.capabilities
            );
            if (
              ["base-missing", "delay-missing", "optional-missing"].includes(
                failure
              )
            ) {
              expect(unavailable.paths.ffmpeg.ready).toBe(true);
            }
            const created = await call(
              "project_create",
              { fps: 10, height: 108, name: `Fallback ${failure}`, width: 192 },
              writeResultSchema
            );
            const fallbackRead = () =>
              call(
                "project_get_state",
                { projectId: created.projectId },
                projectStateSchema
              );
            const emptyFallback = await fallbackRead();
            const fallbackTrack = emptyFallback.project.tracks.find(
              (track) => track.trackType === "overlay"
            );
            if (!fallbackTrack) {
              throw new Error("Fallback overlay track missing");
            }
            const fallbackEdit = await call(
              "timeline_add_text",
              {
                durationMs: 1000,
                expectedRevision: 0,
                projectId: created.projectId,
                startMs: 0,
                text: "Available offline",
                trackId: fallbackTrack.id,
              },
              writeResultSchema
            );
            const editedFallback = await fallbackRead();
            await call(
              "project_undo",
              {
                expectedRevision: fallbackEdit.revision,
                projectId: created.projectId,
              },
              writeResultSchema
            );
            expect(projectContent((await fallbackRead()).project)).toEqual(
              projectContent(emptyFallback.project)
            );
            await call(
              "project_redo",
              {
                expectedRevision: (await fallbackRead()).project.revision,
                projectId: created.projectId,
              },
              writeResultSchema
            );
            expect(projectContent((await fallbackRead()).project)).toEqual(
              projectContent(editedFallback.project)
            );
            const persistedFallback = await fallbackRead();
            await client.close();
            validator.clear();
            client = await connect(overrides);
            expect(
              await call(
                "project_open",
                { projectId: created.projectId },
                projectStateSchema
              )
            ).toEqual(persistedFallback);
            expect(unavailable.subsystems.rendering.ready).toBe(
              failure === "optional-missing"
            );
            for (const capability of optional) {
              expect(
                unavailable.subsystems.rendering.capabilities
              ).not.toContain(capability);
            }
            if (failure !== "optional-missing") {
              expect(unavailable.subsystems.rendering.error?.code).toBe(
                "DEPENDENCY_UNAVAILABLE"
              );
            }
            expect(
              await call("project_open", { projectId }, projectStateSchema)
            ).toEqual(saved);
            const edited = await call(
              "timeline_add_text",
              {
                durationMs: 1000,
                expectedRevision: saved.project.revision,
                projectId,
                startMs: 0,
                text: "Editing remains available",
                trackId: visualTrack.id,
              },
              writeResultSchema
            );
            expect(edited.revision).toBe(saved.project.revision + 1);
            await call(
              "project_undo",
              { expectedRevision: edited.revision, projectId },
              writeResultSchema
            );
            expect(projectContent((await read()).project)).toEqual(
              projectContent(saved.project)
            );
            saved = await read();
            if (failure === "optional-missing") {
              await call(
                "audio_bus_set_dsp",
                {
                  busId: "master",
                  dsp: { compressor: null, eq: [], gainDb: -3, pan: 0 },
                  expectedRevision: saved.project.revision,
                  projectId,
                },
                writeResultSchema
              );
              saved = await read();
            }
            const retained = files(directory);
            const job = await terminal(
              (
                await call(
                  "preview_review_range",
                  {
                    endMs: 1000,
                    expectedRevision: saved.project.revision,
                    projectId,
                    resolution: "project",
                    startMs: 0,
                  },
                  jobSchema
                )
              ).jobId
            );
            expect(job.status).toBe("failed");
            expect(job.error?.code).toBe("DEPENDENCY_UNAVAILABLE");
            expect(job.artifact).toBeUndefined();
            expect(await read()).toEqual(saved);
            expect(files(directory)).toEqual(retained);
            records.push({ failure, job, status: unavailable });
            if (failure === "optional-missing") {
              await call(
                "project_undo",
                { expectedRevision: saved.project.revision, projectId },
                writeResultSchema
              );
              saved = await read();
            }
          }
        );
        writeFileSync(
          join(evidence, `${mode}-evidence.json`),
          `${JSON.stringify({ mode, platform: process.platform, records, versions: { ffmpeg: execute(ffmpeg, ["-version"]).stdout.toString(), ffprobe: execute(ffprobe, ["-version"]).stdout.toString() } }, null, 2)}\n`
        );
        await verifyRuntimePackage(runtime);
      } finally {
        await client.close();
      }
    },
    600_000
  );
}
