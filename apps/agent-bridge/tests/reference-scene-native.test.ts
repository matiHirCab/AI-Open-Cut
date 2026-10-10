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
import { dirname, join, resolve } from "node:path";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { afterEach, expect, it, vi } from "vitest";
import type { ZodType } from "zod/v4";
import recipe from "../../../contracts/complete-reference-scene-v1.json";
import { requireCompleteReferenceCacheTrace } from "../scripts/motion-native-cache-trace";
import {
  jobSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";
import { memoizedSdkValidator } from "./fixtures/memoized-sdk-validator";

const ownedRoots = new Set<string>();
afterEach(() => {
  for (const root of ownedRoots) {
    rmSync(root, { force: true, recursive: true });
  }
  ownedRoots.clear();
});

const required = <T>(value: T | undefined): T => {
  if (value === undefined) {
    throw new Error("Missing required reference evidence");
  }
  return value;
};
const substitute = (input: unknown, ids: Record<string, string>): unknown => {
  if (typeof input === "string") {
    return ids[input] ?? input;
  }
  if (Array.isArray(input)) {
    return input.map((value) => substitute(value, ids));
  }
  if (input && typeof input === "object") {
    return Object.fromEntries(
      Object.entries(input).map(([key, value]) => [key, substitute(value, ids)])
    );
  }
  return input;
};
const SSIM_SCORE = /All:([0-9.]+)/;
// Literal decoration witness, independent of the recipe and renderer output.
// The authored ellipse has an opaque gold interior at (127,89).
const goldWitness = (rgb: Buffer) => {
  const offset = (89 * 192 + 127) * 3;
  return (
    rgb.length === 192 * 108 * 3 &&
    required(rgb[offset]) >= 250 &&
    Math.abs(required(rgb[offset + 1]) - 204) <= 3 &&
    required(rgb[offset + 2]) <= 3
  );
};
const sequence = <T>(
  values: readonly T[],
  action: (value: T) => Promise<void>
) =>
  values.reduce(
    (previous, value) => previous.then(() => action(value)),
    Promise.resolve()
  );
const content = (project: unknown) =>
  Object.fromEntries(
    Object.entries(structuredClone(project) as Record<string, unknown>).filter(
      ([key]) => !["revision", "updatedAtMs"].includes(key)
    )
  );
const inventory = (directory: string) => {
  const result: Record<string, string> = {};
  const visit = (path: string) => {
    for (const name of readdirSync(path).sort()) {
      if (["previews", "analyses", ".lock"].includes(name)) {
        continue;
      }
      const child = join(path, name);
      if (statSync(child).isDirectory()) {
        visit(child);
      } else {
        result[child] = createHash("sha256")
          .update(readFileSync(child))
          .digest("hex");
      }
    }
  };
  visit(directory);
  return result;
};
for (const mode of ["source", "compiled"] as const) {
  const packaged = mode === "compiled";
  it.skipIf(process.env.OPENCUT_REFERENCE_REQUIRED !== "1")(
    `native ten-group reference through ${packaged ? "compiled" : "source"} MCP`,
    async () => {
      const repo = resolve(import.meta.dirname, "../../..");
      const root = mkdtempSync(join(tmpdir(), "opencut-reference-"));
      ownedRoots.add(root);
      const media = join(root, "media"),
        projects = join(root, "projects"),
        exports = join(root, "exports"),
        work = join(root, "work");
      for (const path of [media, projects, exports, work]) {
        mkdirSync(path);
      }
      const ffmpeg = required(process.env.OPENCUT_FFMPEG_PATH),
        ffprobe = required(process.env.OPENCUT_FFPROBE_PATH),
        font = required(process.env.OPENCUT_TEST_FONT_PATH);
      expect(statSync(font).isFile()).toBe(true);
      const ownedFont = join(media, "DejaVuSans.ttf");
      for (const name of [
        "DejaVuSans.ttf",
        "DejaVuSans-Bold.ttf",
        "DejaVuSans-Oblique.ttf",
        "DejaVuSans-BoldOblique.ttf",
      ]) {
        copyFileSync(join(dirname(font), name), join(media, name));
      }
      const python = process.env.OPENCUT_TEST_PYTHON ?? "python";
      const worker = resolve(
        import.meta.dirname,
        "fixtures/reference_scene_worker.py"
      );
      const sources = spawnSync(python, [worker, "--sources", media]);
      expect(
        sources.status,
        sources.error?.message ?? sources.stderr?.toString()
      ).toBe(0);
      const binary = join(
        root,
        process.platform === "win32" ? "bridge.exe" : "bridge"
      );
      if (packaged) {
        const compiled = spawnSync("bun", [
          "build",
          join(repo, "apps/agent-bridge/src/index.ts"),
          "--target=bun",
          "--compile",
          "--outfile",
          binary,
        ]);
        expect(compiled.status, compiled.stderr.toString()).toBe(0);
      }
      const trace = join(root, "cache-trace.jsonl");
      writeFileSync(trace, "");
      const wrapper = join(
        root,
        process.platform === "win32" ? "capture.exe" : "capture"
      );
      const capture = spawnSync("rustc", [
        join(
          repo,
          "crates/editor-core/tests/fixtures/preview_worker_capture.rs"
        ),
        "-o",
        wrapper,
      ]);
      expect(capture.status, capture.stderr.toString()).toBe(0);
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
        OPENCUT_HEADLESS_PATH: wrapper,
        OPENCUT_KOKORO_MODEL_DIR: root,
        OPENCUT_KOKORO_PYTHON: python,
        OPENCUT_KOKORO_WORKER: worker,
        OPENCUT_PREVIEW_TEST_HEADLESS:
          process.env.OPENCUT_TEST_HEADLESS_PATH ??
          join(
            repo,
            "target/debug",
            process.platform === "win32"
              ? "opencut-headless.exe"
              : "opencut-headless"
          ),
        OPENCUT_PREVIEW_TEST_TRACE: trace,
        OPENCUT_PROJECTS_DIR: projects,
        OPENCUT_TTS_WORK_DIR: work,
      };
      const validator = memoizedSdkValidator();
      const connect = async () => {
        const next = new Client(
          { name: "complete-reference", version: "1" },
          { jsonSchemaValidator: validator }
        );
        await next.connect(
          new StdioClientTransport({
            args: packaged
              ? []
              : ["run", join(repo, "apps/agent-bridge/src/index.ts")],
            command: packaged ? binary : "bun",
            env: environment,
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
        if (result.isError) {
          throw new Error(
            `${name}: ${JSON.stringify(result.structuredContent ?? result.content)}`
          );
        }
        return schema.parse(result.structuredContent);
      };
      const terminal = async (
        jobId: string,
        attempts = 6000
      ): Promise<ReturnType<typeof jobSchema.parse>> => {
        const job = await call("job_get_status", { jobId }, jobSchema);
        if (["completed", "failed", "cancelled"].includes(job.status)) {
          expect(job.status, JSON.stringify(job.error)).toBe("completed");
          return job;
        }
        if (attempts === 0) {
          throw new Error(
            "Reference native job exceeded bounded 10 minute allowance"
          );
        }
        await new Promise((done) => setTimeout(done, 100));
        return await terminal(jobId, attempts - 1);
      };
      const records: unknown[] = [];
      try {
        const { projectId } = await call(
          "project_create",
          { name: recipe.name, ...recipe.settings },
          writeResultSchema
        );
        const read = () =>
          call("project_get_state", { projectId }, projectStateSchema);
        const initial = await read();
        expect(initial.project.assets).toEqual([]);
        expect(initial.project.components).toEqual([]);
        const ids: Record<string, string> = {
          $audio: required(
            initial.project.tracks.find((t) => t.trackType === "audio")
          ).id,
          $visual: required(
            initial.project.tracks.find((t) => t.trackType === "overlay")
          ).id,
        };
        const voice = await terminal(
          (
            await call(
              "speech_generate_and_insert",
              {
                expectedRevision: 0,
                language: "en-US",
                projectId,
                startMs: 0,
                text: "EVERY SINGLE ONE rules Starting with Venusaur",
                trackId: ids.$audio,
                voice: "fixture",
              },
              jobSchema
            )
          ).jobId
        );
        ids.$voice = required(voice.result?.assetId);
        await sequence(["music", "event0", "event1"], async (name) => {
          const imported = await call(
            "asset_import",
            {
              expectedRevision: (await read()).project.revision,
              mediaType: "audio",
              path: join(media, `${name}.wav`),
              projectId,
            },
            writeResultSchema
          );
          ids[`$${name}`] = required(imported.changedIds[0]);
        });
        const authored = await call(
          "timeline_batch_edit",
          {
            expectedRevision: (await read()).project.revision,
            operations: substitute(recipe.operations, ids),
            projectId,
          },
          writeResultSchema
        );
        const saved = await read();
        const { revision } = saved.project;
        const directory = join(projects, projectId);
        expect(saved.project.settings).toEqual(recipe.settings);
        expect(saved.project.components).toHaveLength(1);
        expect(saved.project.components[0]?.tracks[0]?.items).toHaveLength(6);
        expect(
          saved.project.markers.map(({ name, timeMs }) => ({ name, timeMs }))
        ).toEqual(recipe.cues.map(({ name, timeMs }) => ({ name, timeMs })));
        const items = saved.project.tracks.flatMap((t) => t.items),
          item = (alias: string) =>
            required(items.find((v) => v.id === authored.aliases[alias]));
        for (const [index, title] of ["Plan", "Build", "Check"].entries()) {
          expect(item(`card${index}`)).toMatchObject({
            componentId: authored.aliases.card,
            parent: { id: authored.aliases.parent },
            slotValues: {
              number: { value: String(index + 1) },
              title: { value: title },
            },
          });
        }
        expect(item("grid")).toMatchObject({
          grid: { pattern: { type: "diagonal" } },
          type: "grid",
          zIndex: -10,
        });
        expect(item("decorative-copies")).toMatchObject({
          repeater: { copies: 3 },
          type: "repeater",
        });
        expect(item("word0")).toMatchObject({
          animationPresetProvenance: {
            "transform.scale_x": { presetId: "impact_slam" },
          },
          startMs: 500,
          type: "text",
        });
        expect(item("hero-hero")).toMatchObject({
          matte: { sourceId: authored.aliases["hero-provider"] },
          startMs: 4300,
        });
        const before = inventory(directory);
        const invalidSlot = await client.callTool({
          arguments: {
            expectedRevision: revision,
            operations: [
              {
                itemId: authored.aliases.card0,
                offsetMs: 0,
                operation: "component_instance_duplicate",
                slotValues: { title: { type: "number", value: 1 } },
              },
            ],
            projectId,
          },
          name: "timeline_batch_edit",
        });
        expect(invalidSlot.isError).toBe(true);
        expect(invalidSlot.structuredContent).toMatchObject({
          error: { code: "INVALID_ARGUMENT", retryable: false },
        });
        expect(await read()).toEqual(saved);
        expect(inventory(directory)).toEqual(before);
        await sequence([revision - 1, revision], async (expectedRevision) => {
          const failed = await client.callTool({
            arguments: {
              expectedRevision,
              operations: [
                {
                  itemId: authored.aliases.grid,
                  operation: "item_set_z_index",
                  zIndex: 2,
                },
                { itemId: "missing", operation: "delete_item" },
              ],
              projectId,
            },
            name: "timeline_batch_edit",
          });
          expect(failed.isError).toBe(true);
          expect(failed.structuredContent).toMatchObject({
            error: {
              code:
                expectedRevision === revision
                  ? "ITEM_NOT_FOUND"
                  : "REVISION_CONFLICT",
              retryable: expectedRevision !== revision,
            },
          });
          expect(await read()).toEqual(saved);
          expect(inventory(directory)).toEqual(before);
        });
        const render = async (
          name: string,
          input: Record<string, unknown>,
          expectedRevision = revision
        ) => {
          const job = await terminal(
            (
              await call(
                name,
                { expectedRevision, projectId, ...input },
                jobSchema
              )
            ).jobId
          );
          expect(job.revision).toBe(expectedRevision);
          expect(job.artifactResource?.uri).toBe(
            `opencut://jobs/${job.jobId}/artifact`
          );
          const artifact = required(job.artifact),
            path = join(
              job.kind === "export" ? exports : directory,
              artifact.relativePath
            );
          expect(statSync(path).size).toBe(artifact.sizeBytes);
          records.push({ artifact, input, name, revision: job.revision });
          return { job, path };
        };
        const frame = await render("preview_render_frame", { timeMs: 650 });
        expect(frame.job.artifact?.mimeType).toBe("image/png");
        const range = await render("preview_review_range", {
          endMs: 6000,
          resolution: "project",
          startMs: 0,
        });
        const probe = (path: string) => {
          const result = spawnSync(ffprobe, [
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
            path,
          ]);
          expect(result.status, result.stderr.toString()).toBe(0);
          return JSON.parse(result.stdout.toString()) as {
            streams: {
              codec_type: string;
              width?: number;
              height?: number;
              r_frame_rate?: string;
            }[];
            format: { duration: string };
          };
        };
        await sequence(
          [
            ["540p", 960, 540],
            ["720p", 1280, 720],
          ] as const,
          async ([resolution, width, height]) => {
            const cold = await render("preview_review_range", {
              endMs: 1000,
              resolution,
              startMs: 0,
            });
            const warm = await render("preview_review_range", {
              endMs: 1000,
              resolution,
              startMs: 0,
            });
            expect(readFileSync(warm.path)).toEqual(readFileSync(cold.path));
            expect(warm.path).not.toBe(cold.path);
            const { streams } = probe(cold.path);
            expect(streams.find((s) => s.codec_type === "video")).toMatchObject(
              {
                height,
                r_frame_rate: "10/1",
                width,
              }
            );
            expect(streams.some((s) => s.codec_type === "audio")).toBe(true);
          }
        );
        const readStats = () =>
          readFileSync(trace, "utf8")
            .split("\n")
            .filter(Boolean)
            .flatMap((line) => {
              try {
                const entry = JSON.parse(line) as {
                  previewCacheTest?: {
                    hits: number;
                    misses: number;
                    finalExecutions: number;
                  };
                };
                return entry.previewCacheTest ? [entry.previewCacheTest] : [];
              } catch {
                return [];
              }
            });
        await vi.waitFor(() => expect(readStats()).toHaveLength(6));
        const stats = readStats();
        requireCompleteReferenceCacheTrace(process.platform, stats);
        records.push({ cacheStats: stats });
        const exported = await render("project_export_video", {
          format: "mp4",
          overwrite: false,
          relativePath: "reference.mp4",
          resolution: "project",
        });
        expect(
          Math.abs(Number(probe(exported.path).format.duration) - 6)
        ).toBeLessThanOrEqual(0.1);
        const decode = (path: string, audio: boolean) => {
          const result = spawnSync(
            ffmpeg,
            [
              "-v",
              "error",
              "-i",
              path,
              ...(audio
                ? ["-vn", "-ar", "48000", "-ac", "2", "-f", "f32le", "pipe:1"]
                : [
                    "-frames:v",
                    "1",
                    "-pix_fmt",
                    "rgb24",
                    "-f",
                    "rawvideo",
                    "pipe:1",
                  ]),
            ],
            { maxBuffer: 8 * 1024 * 1024 }
          );
          expect(result.status, result.stderr.toString()).toBe(0);
          return result.stdout;
        };
        const pcm = decode(range.path, true),
          other = decode(exported.path, true);
        expect(pcm.length).toBeGreaterThan(48_000 * 4);
        expect(pcm.length).toBe(other.length);
        let square = 0,
          energy = 0;
        for (let offset = 0; offset < pcm.length; offset += 4) {
          const a = pcm.readFloatLE(offset),
            b = other.readFloatLE(offset);
          square += (a - b) ** 2;
          energy += a * a;
        }
        const pcmRms = Math.sqrt(square / (pcm.length / 4));
        expect(Math.sqrt(energy / (pcm.length / 4))).toBeGreaterThan(0.01);
        expect(pcmRms).toBeLessThanOrEqual(0.0001);
        const voiceAsset = required(
          saved.project.assets.find((asset) => asset.id === ids.$voice)
        );
        const voicePcm = decode(
          join(directory, voiceAsset.projectRelativePath),
          true
        );
        expect(voicePcm.length).toBe(6000 * 48 * 2 * 4);
        const voiceWindows = (samples: Buffer) => {
          const energyAt = (ms: number) => {
            let total = 0;
            for (let i = ms * 48; i < (ms + 10) * 48; i += 1) {
              total += samples.readFloatLE(i * 8) ** 2;
            }
            return Math.sqrt(total / 480);
          };
          return (
            [0, 450, 950, 1450, 2000, 3000, 4000, 5100].every(
              (ms) => energyAt(ms) === 0
            ) &&
            [550, 1050, 1550, 2450, 3250, 4350].every(
              (ms) => energyAt(ms) > 0.06
            )
          );
        };
        expect(voiceWindows(voicePcm)).toBe(true);
        const missingVoice = Buffer.alloc(voicePcm.length);
        expect(voiceWindows(missingVoice)).toBe(false);
        records.push({
          alignmentOffsetSamples: 0,
          independentAudioWitness:
            "six literal oscillator voice windows and silent gaps",
          pcmRms,
        });
        const pixels = decode(frame.path, false);
        expect(goldWitness(pixels)).toBe(true);
        const missingDecoration = Buffer.from(pixels);
        missingDecoration.fill(
          0,
          (89 * 192 + 127) * 3,
          (89 * 192 + 127) * 3 + 3
        );
        expect(goldWitness(missingDecoration)).toBe(false);
        records.push({
          independentWitness: "opaque gold ellipse at 127,89",
          rgb: [
            ...pixels.subarray((89 * 192 + 127) * 3, (89 * 192 + 127) * 3 + 3),
          ],
        });
        const ssim = spawnSync(
          ffmpeg,
          [
            "-v",
            "info",
            "-i",
            range.path,
            "-i",
            exported.path,
            "-lavfi",
            "[0:v][1:v]ssim",
            "-an",
            "-f",
            "null",
            "-",
          ],
          { maxBuffer: 2 * 1024 * 1024 }
        );
        expect(ssim.status, ssim.stderr.toString()).toBe(0);
        const ssimScore = Number(
          required(ssim.stderr.toString().match(SSIM_SCORE)?.[1])
        );
        expect(ssimScore).toBeGreaterThanOrEqual(0.99);
        records.push({
          exportProbe: probe(exported.path),
          rangeProbe: probe(range.path),
          ssimScore,
        });
        const analysis = await terminal(
          (
            await call(
              "audio_analyze_mix",
              {
                endMs: 6000,
                expectedRevision: revision,
                projectId,
                startMs: 0,
                waveformBins: 64,
              },
              jobSchema
            )
          ).jobId
        );
        const audioSummary = required(analysis.audioAnalysis);
        expect(audioSummary.integratedLufs).not.toBeNull();
        expect(audioSummary.truePeakDbtp).not.toBeNull();
        records.push({ analysis: audioSummary });
        expect(inventory(directory)).toEqual(before);
        const changed = await call(
          "timeline_batch_edit",
          {
            expectedRevision: revision,
            operations: [
              {
                itemId: authored.aliases.card0,
                offsetMs: 0,
                operation: "component_instance_duplicate",
                slotValues: { title: { type: "text", value: "Inspect" } },
              },
              {
                itemId: authored.aliases.parent,
                operation: "update_item",
                staggerMs: 120,
              },
            ],
            projectId,
          },
          writeResultSchema
        );
        const edited = await read();
        expect(edited.project.revision).toBe(changed.revision);
        const editedFrame = await render(
          "preview_render_frame",
          { timeMs: 650 },
          changed.revision
        );
        const editedBytes = readFileSync(editedFrame.path);
        await call(
          "project_undo",
          { expectedRevision: changed.revision, projectId },
          writeResultSchema
        );
        expect(content((await read()).project)).toEqual(content(saved.project));
        const undoneFrame = await render(
          "preview_render_frame",
          { timeMs: 650 },
          changed.revision + 1
        );
        expect(readFileSync(undoneFrame.path)).toEqual(
          readFileSync(frame.path)
        );
        await call(
          "project_redo",
          { expectedRevision: changed.revision + 1, projectId },
          writeResultSchema
        );
        expect(content((await read()).project)).toEqual(
          content(edited.project)
        );
        const redoneFrame = await render(
          "preview_render_frame",
          { timeMs: 650 },
          changed.revision + 2
        );
        expect(readFileSync(redoneFrame.path)).toEqual(editedBytes);
        const restored = await read();
        await client.close();
        validator.clear();
        client = await connect();
        expect(
          await call("project_open", { projectId }, projectStateSchema)
        ).toEqual(restored);
        const reopenedFrame = await render(
          "preview_render_frame",
          { timeMs: 650 },
          restored.project.revision
        );
        expect(readFileSync(reopenedFrame.path)).toEqual(editedBytes);
        const report = process.env.OPENCUT_REFERENCE_REPORT_DIR;
        if (report) {
          const destination = join(report, mode);
          mkdirSync(destination, { recursive: true });
          copyFileSync(exported.path, join(destination, "reference.mp4"));
          writeFileSync(
            join(destination, "evidence.json"),
            JSON.stringify(
              { records, revision, settings: recipe.settings },
              null,
              2
            )
          );
        }
      } finally {
        await client.close();
        validator.clear();
        rmSync(root, { force: true, recursive: true });
      }
    },
    1_500_000
  );
}
