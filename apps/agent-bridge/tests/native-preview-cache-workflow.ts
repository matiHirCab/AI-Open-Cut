import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { expect, vi } from "vitest";
import type { ZodType } from "zod/v4";
import {
  jobSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";

const triangle = () => {
  const bytes = Buffer.alloc(44 + 48_000 * 2);
  bytes.write("RIFF", 0);
  bytes.writeUInt32LE(bytes.length - 8, 4);
  bytes.write("WAVEfmt ", 8);
  bytes.writeUInt32LE(16, 16);
  bytes.writeUInt16LE(1, 20);
  bytes.writeUInt16LE(1, 22);
  bytes.writeUInt32LE(48_000, 24);
  bytes.writeUInt32LE(96_000, 28);
  bytes.writeUInt16LE(2, 32);
  bytes.writeUInt16LE(16, 34);
  bytes.write("data", 36);
  bytes.writeUInt32LE(bytes.length - 44, 40);
  for (let index = 0; index < 48_000; index += 1) {
    const phase = Math.floor((index * 440 * 4) / 48_000) % 4;
    const fraction = Math.floor(
      (((index * 440 * 4) % 48_000) * 16_000) / 48_000
    );
    const value = [fraction, 16_000 - fraction, -fraction, -16_000 + fraction][
      phase
    ];
    if (value === undefined) {
      throw new Error("Invalid literal phase");
    }
    bytes.writeInt16LE(value, 44 + index * 2);
  }
  return bytes;
};
const owned = (directory: string) => {
  const files: Record<string, string> = {};
  const visit = (root: string) => {
    for (const name of readdirSync(root).sort()) {
      if (name === "previews" || name === ".lock") {
        continue;
      }
      const path = join(root, name);
      if (statSync(path).isDirectory()) {
        visit(path);
      } else {
        files[path] = readFileSync(path).toString("base64");
      }
    }
  };
  visit(directory);
  return files;
};
const required = <T>(value: T | undefined): T => {
  if (value === undefined) {
    throw new Error("Missing native preview evidence");
  }
  return value;
};

export const verifyNativePreviewCacheMcp = async (packaged: boolean) => {
  const root = mkdtempSync(join(tmpdir(), "opencut-native-preview-"));
  const repo = resolve(import.meta.dirname, "../../..");
  const ffmpeg = required(process.env.OPENCUT_FFMPEG_PATH);
  const ffprobe = required(process.env.OPENCUT_FFPROBE_PATH);
  const trace = join(root, "worker-trace.jsonl");
  const wrapper = join(
    root,
    process.platform === "win32" ? "headless-capture.exe" : "headless-capture"
  );
  const compiled = spawnSync("rustc", [
    join(repo, "crates/editor-core/tests/fixtures/preview_worker_capture.rs"),
    "-o",
    wrapper,
  ]);
  expect(compiled.status, compiled.stderr.toString()).toBe(0);
  const binary = join(
    root,
    process.platform === "win32" ? "bridge.exe" : "bridge"
  );
  if (packaged) {
    const build = spawnSync("bun", [
      "build",
      resolve(import.meta.dirname, "../src/index.ts"),
      "--target=bun",
      "--compile",
      "--outfile",
      binary,
    ]);
    expect(build.status, build.stderr.toString()).toBe(0);
  }
  const projects = join(root, "projects");
  const media = join(root, "media");
  const exports = join(root, "exports");
  mkdirSync(media);
  writeFileSync(join(media, "tone.wav"), triangle());
  writeFileSync(trace, "");
  const environment = {
    ...Object.fromEntries(
      Object.entries(process.env).filter(
        (entry): entry is [string, string] => entry[1] !== undefined
      )
    ),
    OPENCUT_ALLOWED_MEDIA_DIRS: media,
    OPENCUT_EXPORTS_DIR: exports,
    OPENCUT_FFMPEG_PATH: ffmpeg,
    OPENCUT_FFPROBE_PATH: ffprobe,
    OPENCUT_HEADLESS_PATH: wrapper,
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
  };
  const connect = async () => {
    const client = new Client({ name: "native-preview-cache", version: "1" });
    await client.connect(
      new StdioClientTransport({
        args: packaged
          ? []
          : ["run", resolve(import.meta.dirname, "../src/index.ts")],
        command: packaged ? binary : "bun",
        env: environment,
        stderr: "inherit",
      })
    );
    return client;
  };
  let client = await connect();
  try {
    const call = async <T>(
      name: string,
      input: Record<string, unknown>,
      schema: ZodType<T>
    ) => {
      const result = await client.callTool({ arguments: input, name });
      if (result.isError) {
        throw new Error(JSON.stringify(result.structuredContent));
      }
      return schema.parse(result.structuredContent);
    };
    const created = await call(
      "project_create",
      { fps: 10, height: 32, name: "Native preview", width: 32 },
      writeResultSchema
    );
    const { projectId } = created;
    const initial = await call(
      "project_open",
      { projectId },
      projectStateSchema
    );
    const overlay = required(
      initial.project.tracks.find((track) => track.trackType === "overlay")
    ).id;
    const audio = required(
      initial.project.tracks.find((track) => track.trackType === "audio")
    ).id;
    await call(
      "timeline_add_solid_color",
      {
        color: "#112233",
        durationMs: 1000,
        expectedRevision: 0,
        projectId,
        startMs: 0,
        trackId: overlay,
        transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
      },
      writeResultSchema
    );
    const imported = await call(
      "asset_import",
      {
        expectedRevision: 1,
        mediaType: "audio",
        path: join(media, "tone.wav"),
        projectId,
      },
      writeResultSchema
    );
    await call(
      "timeline_add_media",
      {
        assetId: required(imported.changedIds[0]),
        durationMs: 1000,
        expectedRevision: 2,
        projectId,
        startMs: 0,
        trackId: audio,
      },
      writeResultSchema
    );
    const directory = join(projects, projectId);
    const before = owned(directory);
    const terminal = async (
      jobId: string,
      remaining = 300
    ): Promise<ReturnType<typeof jobSchema.parse>> => {
      if (remaining === 0) {
        throw new Error("Native preview job timed out");
      }
      const result = await call("job_get_status", { jobId }, jobSchema);
      if (["completed", "failed", "cancelled"].includes(result.status)) {
        expect(result.status, JSON.stringify(result.error)).toBe("completed");
        return result;
      }
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 25));
      return terminal(jobId, remaining - 1);
    };
    const render = async (isRange: boolean, revision = 3) => {
      const input = isRange
        ? {
            endMs: 1000,
            fps: 10,
            includeAudio: true,
            resolution: { height: 32, width: 32 },
            startMs: 0,
          }
        : { timeMs: 0 };
      const queued = await call(
        isRange ? "preview_render_range" : "preview_render_frame",
        { expectedRevision: revision, projectId, ...input },
        jobSchema
      );
      const completed = await terminal(queued.jobId);
      const contents = await client.readResource({
        uri: required(completed.artifactResource).uri,
      });
      const content = required(contents.contents[0]);
      if (!("blob" in content)) {
        throw new Error("Native artifact blob missing");
      }
      const bytes = Buffer.from(content.blob, "base64");
      expect(bytes.length).toBe(required(completed.artifact).sizeBytes);
      expect(required(completed.artifactResource).uri).toBe(
        `opencut://jobs/${queued.jobId}/artifact`
      );
      return {
        bytes,
        completed,
        path: join(directory, required(completed.artifact).relativePath),
      };
    };
    const frame = await render(false);
    const frameWarm = await render(false);
    const range = await render(true);
    const rangeWarm = await render(true);
    expect(frameWarm.bytes).toEqual(frame.bytes);
    expect(rangeWarm.bytes).toEqual(range.bytes);
    expect(frameWarm.path).not.toBe(frame.path);
    expect(rangeWarm.path).not.toBe(range.path);
    const stats = () =>
      readFileSync(trace, "utf8")
        .split("\n")
        .filter(Boolean)
        .flatMap((line) => {
          try {
            const value = JSON.parse(line).previewCacheTest;
            return value ? [value] : [];
          } catch {
            return [];
          }
        });
    await vi.waitFor(() => expect(stats()).toHaveLength(4));
    expect(
      stats().map((value) => [value.hits, value.misses, value.finalExecutions])
    ).toEqual([
      [0, 1, 1],
      [1, 1, 1],
      [1, 2, 2],
      [2, 2, 2],
    ]);
    const decode = (path: string, audioOnly: boolean) => {
      const args = audioOnly
        ? ["-vn", "-ac", "1", "-ar", "48000", "-f", "f32le", "pipe:1"]
        : ["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"];
      const result = spawnSync(ffmpeg, ["-v", "error", "-i", path, ...args], {
        maxBuffer: 8 * 1024 * 1024,
      });
      expect(result.status, result.stderr.toString()).toBe(0);
      return result.stdout;
    };
    for (const artifact of [frameWarm, rangeWarm]) {
      const pixels = decode(artifact.path, false);
      expect(pixels.length).toBe(32 * 32 * 3);
      // Independent constant-color SSIM: every channel is a constant plate.
      let score = 0;
      for (let channel = 0; channel < 3; channel += 1) {
        const expected = required([17, 34, 51][channel]);
        let mean = 0;
        let square = 0;
        for (let offset = channel; offset < pixels.length; offset += 3) {
          mean += required(pixels[offset]);
          square += required(pixels[offset]) ** 2;
        }
        mean /= 1024;
        const variance = square / 1024 - mean ** 2;
        score +=
          ((2 * expected * mean + 6.5025) /
            (expected ** 2 + mean ** 2 + 6.5025)) *
          (58.5225 / (variance + 58.5225));
      }
      expect(score / 3).toBeGreaterThanOrEqual(0.99);
    }
    const goldenRoot = join(
      repo,
      "crates/editor-core/tests/fixtures/render-golden"
    );
    const pointer = JSON.parse(
      readFileSync(join(goldenRoot, "CURRENT"), "utf8")
    );
    const generation = join(goldenRoot, "generations", pointer.generation);
    const manifestBytes = readFileSync(join(generation, "manifest.json"));
    expect(createHash("sha256").update(manifestBytes).digest("hex")).toBe(
      pointer.generation
    );
    const manifest = JSON.parse(manifestBytes.toString());
    const reference = required(
      manifest.references.find(
        (entry: { kind: string }) => entry.kind === "audio"
      )
    );
    const expectedAudio = readFileSync(join(generation, reference.path));
    expect(createHash("sha256").update(expectedAudio).digest("hex")).toBe(
      reference.sha256
    );
    const delivered = decode(rangeWarm.path, true);
    expect(delivered.length).toBe(expectedAudio.length);
    let squared = 0;
    for (let offset = 0; offset < delivered.length; offset += 4) {
      squared +=
        (delivered.readFloatLE(offset) - expectedAudio.readFloatLE(offset)) **
        2;
    }
    expect(Math.sqrt(squared / (delivered.length / 4))).toBeLessThanOrEqual(
      0.0001
    );
    const probe = spawnSync(ffprobe, [
      "-v",
      "error",
      "-show_entries",
      "format=duration",
      "-of",
      "json",
      rangeWarm.path,
    ]);
    expect(probe.status).toBe(0);
    expect(
      Math.abs(Number(JSON.parse(probe.stdout.toString()).format.duration) - 1)
    ).toBeLessThanOrEqual(0.1);
    expect(owned(directory)).toEqual(before);
    const stale = await client.callTool({
      arguments: { expectedRevision: 0, projectId, timeMs: 0 },
      name: "preview_render_frame",
    });
    expect(stale.isError).toBe(true);
    expect(owned(directory)).toEqual(before);
    const rollback = await client.callTool({
      arguments: {
        expectedRevision: 3,
        operations: [
          {
            color: "#f00",
            durationMs: 1000,
            operation: "add_solid_color",
            resultAlias: "pending",
            startMs: 0,
            trackId: overlay,
          },
          { color: "#fff", itemId: "@missing", operation: "update_item" },
        ],
        projectId,
      },
      name: "timeline_batch_edit",
    });
    expect(rollback.isError).toBe(true);
    expect(owned(directory)).toEqual(before);
    const afterError = await render(false);
    expect(afterError.bytes).toEqual(frame.bytes);
    await vi.waitFor(() => expect(stats()).toHaveLength(5));
    expect(stats()[4]).toMatchObject({
      finalExecutions: 2,
      hits: 3,
      misses: 2,
    });
    await call(
      "project_undo",
      { expectedRevision: 3, projectId },
      writeResultSchema
    );
    await render(false, 4);
    await call(
      "project_redo",
      { expectedRevision: 4, projectId },
      writeResultSchema
    );
    const restored = await render(false, 5);
    expect(restored.bytes).toEqual(frame.bytes);
    await call("project_open", { projectId }, projectStateSchema);
    await client.close();
    client = await connect();
    const fresh = await render(false, 5);
    expect(fresh.bytes).toEqual(frame.bytes);
    await vi.waitFor(() => expect(stats()).toHaveLength(8));
    expect(stats()[7]).toMatchObject({
      finalExecutions: 1,
      hits: 0,
      misses: 1,
    });
  } finally {
    await client.close();
    rmSync(root, { force: true, recursive: true });
  }
};
