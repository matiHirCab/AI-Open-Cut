import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { expect, it } from "vitest";
import type { ZodType } from "zod/v4";
import {
  editDraftSchema,
  jobSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

const required =
  process.env.OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED === "1" ||
  process.env.OPENCUT_GOLDEN_REQUIRED === "1";
it.runIf(required)(
  "renders alpha and luma mattes through actual MCP artifacts with independent color and revision controls",
  async () => {
    const ffmpeg = process.env.OPENCUT_FFMPEG_PATH ?? "ffmpeg";
    const ffprobe = process.env.OPENCUT_FFPROBE_PATH ?? "ffprobe";
    expect(
      spawnSync(ffmpeg, ["-version"]).status,
      "mandatory actual FFmpeg"
    ).toBe(0);
    expect(
      spawnSync(ffprobe, ["-version"]).status,
      "mandatory actual FFprobe"
    ).toBe(0);
    const root = mkdtempSync(join(tmpdir(), "opencut-native-matte-mcp-"));
    const projects = join(root, "projects"),
      media = join(root, "media"),
      exportsDirectory = join(root, "exports");
    for (const path of [projects, media, exportsDirectory]) {
      mkdirSync(path, { recursive: true });
    }
    const capturedPam = join(root, "actual-linear-scene.pam");
    const proxySource = join(root, "ffmpeg_capture.rs");
    const proxyExecutable = join(
      root,
      `ffmpeg-capture${process.platform === "win32" ? ".exe" : ""}`
    );
    writeFileSync(
      proxySource,
      `
use std::{path::Path, process::{Command, Stdio}};
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    for argument in &args {
        let path = Path::new(argument);
        if path.file_name() == Some(std::ffi::OsStr::new("linear-scene.pam")) {
            std::fs::copy(path, ${JSON.stringify(capturedPam)}).expect("copy actual prepared scene");
        }
    }
    let status = Command::new(${JSON.stringify(ffmpeg)}).args(args).stdin(Stdio::inherit())
        .stdout(Stdio::inherit()).stderr(Stdio::inherit()).status().expect("forward real FFmpeg");
    if let Some(code) = status.code() { std::process::exit(code); }
    #[cfg(unix)] { use std::os::unix::process::ExitStatusExt; std::process::exit(128 + status.signal().unwrap_or(1)); }
    #[cfg(not(unix))] std::process::exit(1);
}
`
    );
    const compiledProxy = spawnSync(process.env.RUSTC ?? "rustc", [
      "--edition",
      "2024",
      "--crate-name",
      "ffmpeg_capture",
      proxySource,
      "-o",
      proxyExecutable,
    ]);
    expect(compiledProxy.status, compiledProxy.stderr.toString()).toBe(0);
    const environment = {
      ...Object.fromEntries(
        Object.entries(process.env).filter(
          (entry): entry is [string, string] => entry[1] !== undefined
        )
      ),
      OPENCUT_ALLOWED_MEDIA_DIRS: media,
      OPENCUT_EXPORTS_DIR: exportsDirectory,
      OPENCUT_FFMPEG_PATH: proxyExecutable,
      OPENCUT_FFPROBE_PATH: ffprobe,
      OPENCUT_HEADLESS_PATH:
        process.env.OPENCUT_TEST_HEADLESS_PATH ??
        resolve(import.meta.dirname, "../../../target/debug/opencut-headless"),
      OPENCUT_PROJECTS_DIR: projects,
    };
    const connect = async () => {
      const next = new Client({
        name: "native-matte-conformance",
        version: "1",
      });
      await next.connect(
        new StdioClientTransport({
          args: ["run", resolve(import.meta.dirname, "../src/index.ts")],
          command: "bun",
          env: environment,
          stderr: "inherit",
        })
      );
      return next;
    };
    let client = await connect();
    const call = async <Output>(
      name: string,
      input: Record<string, unknown>,
      schema: ZodType<Output>
    ) => {
      const result = await client.callTool({ arguments: input, name });
      expect(
        result.isError,
        `${name}: ${JSON.stringify(result.structuredContent)}`
      ).not.toBe(true);
      return schema.parse(result.structuredContent);
    };
    const terminal = async (
      jobId: string,
      remaining = 100
    ): Promise<ReturnType<typeof jobSchema.parse>> => {
      const job = await call("job_get_status", { jobId }, jobSchema);
      if (job.status === "completed") {
        return job;
      }
      expect(
        ["failed", "cancelled"].includes(job.status),
        JSON.stringify(job)
      ).toBe(false);
      if (remaining === 0) {
        throw new Error("native MCP matte job did not complete");
      }
      await new Promise((resolveDelay) => setTimeout(resolveDelay, 100));
      return terminal(jobId, remaining - 1);
    };
    try {
      const status = await call("editor_get_status", {}, statusSchema);
      expect(status.subsystems.rendering.ready).toBe(true);
      expect(status.subsystems.rendering.capabilities).toContain(
        "track_mattes_v1"
      );
      const { projectId } = await call(
        "project_create",
        {
          fps: 10,
          height: 64,
          name: "Independent MCP matte pixels",
          width: 64,
        },
        writeResultSchema
      );
      const state = await call(
        "project_open",
        { projectId },
        projectStateSchema
      );
      const trackId = state.project.tracks[1]?.id;
      const created = await call(
        "timeline_add_rectangle",
        {
          color: "#0000ff",
          durationMs: 1000,
          expectedRevision: 0,
          height: 64,
          projectId,
          startMs: 0,
          trackId,
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
          width: 64,
        },
        writeResultSchema
      );
      const [itemId] = created.changedIds;
      const rawFrames = new WeakMap<Buffer, Buffer>();
      const convertedReferences = new WeakMap<Buffer, Buffer>();
      const referenceCache = new Map<string, Buffer>();
      const frame = async (revision: number, timeMs: number) => {
        rmSync(capturedPam, { force: true });
        const queued = await call(
          "preview_render_frame",
          { expectedRevision: revision, projectId, timeMs },
          jobSchema
        );
        const completed = await terminal(queued.jobId);
        expect(completed.artifactResource?.mimeType).toBe("image/png");
        const resource = await client.readResource({
          uri: completed.artifactResource?.uri ?? "",
        });
        const [content] = resource.contents;
        if (!(content && "blob" in content)) {
          throw new Error("native frame artifact blob missing");
        }
        const bytes = Buffer.from(content.blob, "base64");
        expect(bytes.length).toBe(completed.artifact?.sizeBytes);
        const decoded = spawnSync(
          ffmpeg,
          [
            "-v",
            "error",
            "-threads",
            "1",
            "-i",
            "pipe:0",
            "-frames:v",
            "1",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "pipe:1",
          ],
          { input: bytes, maxBuffer: 1024 * 1024 }
        );
        expect(decoded.status, decoded.stderr.toString()).toBe(0);
        expect(decoded.stdout.length).toBe(64 * 64 * 3);
        if (existsSync(capturedPam)) {
          const prepared = readFileSync(capturedPam);
          const headerEnd = prepared.indexOf("ENDHDR\n") + 7;
          expect(headerEnd).toBeGreaterThan(7);
          expect(prepared.subarray(0, headerEnd).toString()).toContain(
            "WIDTH 64\nHEIGHT 64\nDEPTH 4\n"
          );
          const raw = prepared.subarray(headerEnd);
          expect(raw.length).toBe(64 * 64 * 4);
          rawFrames.set(decoded.stdout, raw);
          process.stdout.write(
            `Actual prepared PAM revision=${revision} timeMs=${timeMs} pixel16,16=${Array.from(raw.subarray((16 * 64 + 16) * 4, (16 * 64 + 16) * 4 + 4))}\n`
          );
        } else {
          expect(revision).toBe(1);
          process.stdout.write("Baseline no-matte frame has no prepared PAM\n");
        }
        verifyFrame(decoded.stdout, revision, timeMs);
        return decoded.stdout;
      };
      const pixel = (bytes: Buffer, x: number, y: number) =>
        Array.from(bytes.subarray((y * 64 + x) * 3, (y * 64 + x) * 3 + 3));
      const encoded = (linear: number) =>
        Math.round(
          255 *
            (linear <= 0.003_130_8
              ? 12.92 * linear
              : 1.055 * linear ** (1 / 2.4) - 0.055)
        );
      const blue = (bytes: Buffer, x: number, y: number, expected: number) => {
        const raw = rawFrames.get(bytes);
        if (raw) {
          const rawPixel = Array.from(
            raw.subarray((y * 64 + x) * 4, (y * 64 + x) * 4 + 4)
          );
          expect(rawPixel[0]).toBeLessThan(2);
          expect(rawPixel[1]).toBeLessThan(2);
          expect(rawPixel[3]).toBe(255);
          expect(
            Math.abs((rawPixel[2] ?? 0) - expected),
            `raw analytic pixel (${x},${y})`
          ).toBeLessThanOrEqual(1);
        } else {
          expect(expected, "only the unmasked baseline may lack a PAM").toBe(
            255
          );
        }
        const reference = convertedReferences.get(bytes);
        if (!reference) {
          throw new Error("full independently authored reference missing");
        }
        const referencePixel = pixel(reference, x, y);
        const [red = 0, green = 0, component = 0] = pixel(bytes, x, y);
        expect(red).toBeLessThan(2);
        expect(green).toBeLessThan(2);
        expect(
          Math.abs(component - (referencePixel[2] ?? 0)),
          `final pixel (${x},${y}) RGB=${red},${green},${component}, converted independent blue=${referencePixel[2]}, analytic raw blue=${expected}`
        ).toBeLessThanOrEqual(1);
      };
      const black = (bytes: Buffer, x: number, y: number) => {
        expect(pixel(bytes, x, y).every((value) => value < 2)).toBe(true);
        const raw = rawFrames.get(bytes);
        if (!raw) {
          throw new Error("raw black proof missing");
        }
        expect(
          Array.from(
            raw.subarray((y * 64 + x) * 4, (y * 64 + x) * 4 + 3)
          ).every((value) => value < 2)
        ).toBe(true);
        expect(raw[(y * 64 + x) * 4 + 3]).toBe(255);
      };
      const authoredPlate = (left: number, width: number, channel: number) => {
        const key = `${left}-${width}-${channel}`;
        const cached = referenceCache.get(key);
        if (cached) {
          return cached;
        }
        const rgba = Buffer.alloc(64 * 64 * 4);
        for (let y = 0; y < 64; y += 1) {
          for (let x = 0; x < 64; x += 1) {
            rgba[(y * 64 + x) * 4 + 2] =
              x >= left && x < left + width ? channel : 0;
            rgba[(y * 64 + x) * 4 + 3] = 255;
          }
        }
        const plate = join(root, `independent-${key}.pam`);
        writeFileSync(
          plate,
          Buffer.concat([
            Buffer.from(
              "P7\nWIDTH 64\nHEIGHT 64\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n"
            ),
            rgba,
          ])
        );
        const output = join(root, `independent-${key}.png`);
        const converted = spawnSync(ffmpeg, [
          "-v",
          "error",
          "-nostdin",
          "-f",
          "lavfi",
          "-i",
          "color=c=black:s=64x64:r=10:d=1",
          "-loop",
          "1",
          "-i",
          plate,
          "-filter_complex_threads",
          "1",
          "-filter_complex",
          "[0:v]format=yuv420p[base0];[1:v]fps=10,settb=AVTB,setpts=PTS-STARTPTS+0/TB,format=rgba[plate];[base0][plate]overlay=format=auto:x=0:y=0:eof_action=pass[composed];[composed]scale=64:64:force_original_aspect_ratio=decrease,pad=64:64:(ow-iw)/2:(oh-ih)/2,format=yuv420p[video]",
          "-map",
          "[video]",
          "-frames:v",
          "1",
          "-y",
          output,
        ]);
        expect(converted.status, converted.stderr.toString()).toBe(0);
        const decoded = spawnSync(ffmpeg, [
          "-v",
          "error",
          "-threads",
          "1",
          "-i",
          output,
          "-frames:v",
          "1",
          "-f",
          "rawvideo",
          "-pix_fmt",
          "rgb24",
          "pipe:1",
        ]);
        expect(decoded.status, decoded.stderr.toString()).toBe(0);
        expect(decoded.stdout.length).toBe(64 * 64 * 3);
        referenceCache.set(key, decoded.stdout);
        return decoded.stdout;
      };
      const verifyFrame = (bytes: Buffer, revision: number, timeMs: number) => {
        expect([1, 2, 3, 4, 6, 7, 8, 9]).toContain(revision);
        let channel = encoded(0.7152 * 0.5);
        if (revision === 1) {
          channel = 255;
        } else if (revision === 2) {
          channel = encoded(0.5);
        } else if (revision === 3) {
          channel = encoded(0.2126 * 0.5);
        }
        expect([0, 500]).toContain(timeMs);
        const left = revision === 1 ? 0 : (Math.min(timeMs, 500) * 8) / 500;
        const width = revision === 1 ? 64 : 32;
        convertedReferences.set(bytes, authoredPlate(left, width, channel));
        blue(bytes, 16, 16, channel);
        if (left === 0) {
          blue(bytes, 4, 16, channel);
        } else {
          black(bytes, 4, 16);
        }
        if (width === 64) {
          blue(bytes, 48, 16, channel);
        } else {
          black(bytes, 48, 16);
        }
        process.stdout.write(
          `Dual raw/converted proof revision=${revision} timeMs=${timeMs}: analytic=${channel}, finalReference=${pixel(convertedReferences.get(bytes) as Buffer, 16, 16)[2]}\n`
        );
      };
      const baseline = await frame(1, 500);
      blue(baseline, 4, 16, 255);
      blue(baseline, 48, 16, 255);
      const provider = await call(
        "timeline_batch_edit",
        {
          expectedRevision: 1,
          operations: [
            {
              color: "#ff0000",
              durationMs: 1000,
              height: 64,
              operation: "add_rectangle",
              resultAlias: "provider",
              startMs: 0,
              trackId,
              transform: { opacity: 0.5, positionX: 0, positionY: 0, scale: 1 },
              width: 32,
            },
            { itemId: "@provider", matteOnly: true, operation: "update_item" },
            {
              animationChannels: [
                {
                  keyframes: [
                    {
                      curve: "linear",
                      timeMs: 0,
                      value: { type: "scalar", value: 0 },
                    },
                    {
                      curve: "hold",
                      timeMs: 500,
                      value: { type: "scalar", value: 8 },
                    },
                  ],
                  property: "transform.position_x",
                },
              ],
              itemId: "@provider",
              operation: "set_animation_channels",
            },
            {
              itemId,
              matte: { channel: "alpha", sourceId: "@provider" },
              operation: "update_item",
            },
          ],
          projectId,
        },
        writeResultSchema
      );
      const providerId = provider.aliases.provider;
      if (!providerId) {
        throw new Error("native matte provider alias missing");
      }
      const alphaFirst = await frame(2, 0);
      blue(alphaFirst, 4, 16, encoded(0.5));
      black(alphaFirst, 48, 16);
      const alpha = await frame(2, 500);
      black(alpha, 4, 16);
      blue(alpha, 16, 16, encoded(0.5));
      black(alpha, 48, 16);
      expect(alpha.equals(baseline)).toBe(false);
      expect(await frame(2, 500)).toEqual(alpha);
      await client.close();
      client = await connect();
      expect(
        (await call("project_open", { projectId }, projectStateSchema)).project
          .revision
      ).toBe(2);
      expect(await frame(2, 500)).toEqual(alpha);
      await call(
        "timeline_update_item",
        {
          expectedRevision: 2,
          itemId,
          matte: { channel: "luma", sourceId: providerId },
          projectId,
        },
        writeResultSchema
      );
      const luma = await frame(3, 500);
      black(luma, 4, 16);
      blue(luma, 16, 16, encoded(0.2126 * 0.5));
      black(luma, 48, 16);
      expect(luma.equals(alpha)).toBe(false);
      expect(await frame(3, 500)).toEqual(luma);
      await client.close();
      client = await connect();
      expect(await frame(3, 500)).toEqual(luma);
      await call(
        "timeline_update_item",
        {
          color: "#00ff00",
          expectedRevision: 3,
          itemId: providerId,
          projectId,
        },
        writeResultSchema
      );
      const greenLuma = await frame(4, 500);
      black(greenLuma, 4, 16);
      blue(greenLuma, 16, 16, encoded(0.7152 * 0.5));
      black(greenLuma, 48, 16);
      expect(greenLuma.equals(luma)).toBe(false);
      await client.close();
      client = await connect();
      expect(await frame(4, 500)).toEqual(greenLuma);
      const retained = await call(
        "draft_create",
        {
          expectedRevision: 4,
          operations: [
            {
              itemId,
              matte: { channel: "alpha", sourceId: providerId },
              operation: "update_item",
            },
          ],
          projectId,
        },
        editDraftSchema
      );
      await call(
        "timeline_delete_item",
        { expectedRevision: 4, itemId, projectId },
        writeResultSchema
      );
      const projectDirectory = join(projects, projectId);
      const draftPath = join(projectDirectory, "drafts", `${retained.id}.json`);
      const inventory = (directory: string): Record<string, string> =>
        Object.fromEntries(
          readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
            const path = join(directory, entry.name);
            return entry.isDirectory()
              ? Object.entries(inventory(path)).map(([name, bytes]) => [
                  `${entry.name}/${name}`,
                  bytes,
                ])
              : [[entry.name, readFileSync(path).toString("base64")]];
          })
        );
      const failedPreview = async (
        jobId: string,
        remaining = 100
      ): Promise<void> => {
        const job = await call("job_get_status", { jobId }, jobSchema);
        if (job.status === "failed") {
          expect(job.error).toMatchObject({
            code: "REVISION_CONFLICT",
            retryable: true,
          });
          expect(job.artifact).toBeUndefined();
          expect(job.artifactResource).toBeUndefined();
          return;
        }
        expect(["completed", "cancelled"].includes(job.status)).toBe(false);
        if (remaining === 0) {
          throw new Error("stale draft preview did not fail");
        }
        await new Promise((resolveDelay) => setTimeout(resolveDelay, 100));
        await failedPreview(jobId, remaining - 1);
      };
      const verifyFailedPreview = async (baseRevision: number) => {
        const current = await call(
          "project_open",
          { projectId },
          projectStateSchema
        );
        const before = inventory(projectDirectory);
        const draftRecord = await call(
          "draft_get",
          { draftId: retained.id, projectId },
          editDraftSchema
        );
        expect(draftRecord.baseRevision).toBe(baseRevision);
        const queued = await call(
          "draft_preview_frame",
          { draftId: retained.id, projectId, timeMs: 500 },
          jobSchema
        );
        await failedPreview(queued.jobId);
        expect(
          await call("project_open", { projectId }, projectStateSchema)
        ).toEqual(current);
        expect(inventory(projectDirectory)).toEqual(before);
      };
      await verifyFailedPreview(4);
      await client.close();
      const record = JSON.parse(readFileSync(draftPath, "utf8"));
      record.baseRevision = 999;
      writeFileSync(draftPath, JSON.stringify(record));
      client = await connect();
      await verifyFailedPreview(999);
      // Preserve the stale-base controls above, then exercise native history on
      // the restored active green-luma scene through the public transport.
      await call(
        "project_undo",
        { expectedRevision: 5, projectId },
        writeResultSchema
      );
      const restored = await call(
        "project_open",
        { projectId },
        projectStateSchema
      );
      expect(restored.project.revision).toBe(6);
      expect(
        restored.project.tracks[1]?.items.find((entry) => entry.id === itemId)
      ).toHaveProperty("matte", { channel: "luma", sourceId: providerId });
      expect(await frame(6, 500)).toEqual(greenLuma);
      await call(
        "item_reorder",
        { expectedRevision: 6, index: 0, itemId: providerId, projectId },
        writeResultSchema
      );
      const reordered = await call(
        "project_open",
        { projectId },
        projectStateSchema
      );
      expect(reordered.project.revision).toBe(7);
      expect(reordered.project.tracks[1]?.items[0]?.id).toBe(providerId);
      expect(reordered.project.tracks[1]?.items).not.toEqual(
        restored.project.tracks[1]?.items
      );
      const reorderedPixels = await frame(7, 500);
      black(reorderedPixels, 4, 16);
      blue(reorderedPixels, 16, 16, encoded(0.7152 * 0.5));
      black(reorderedPixels, 48, 16);
      await call(
        "project_undo",
        { expectedRevision: 7, projectId },
        writeResultSchema
      );
      const undone = await call(
        "project_open",
        { projectId },
        projectStateSchema
      );
      expect(undone.project.revision).toBe(8);
      expect(undone.project.tracks).toEqual(restored.project.tracks);
      expect(await frame(8, 500)).toEqual(greenLuma);
      await call(
        "project_redo",
        { expectedRevision: 8, projectId },
        writeResultSchema
      );
      const redone = await call(
        "project_open",
        { projectId },
        projectStateSchema
      );
      expect(redone.project.revision).toBe(9);
      expect(redone.project.tracks).toEqual(reordered.project.tracks);
      expect(await frame(9, 500)).toEqual(reorderedPixels);
      await client.close();
      client = await connect();
      expect(
        await call("project_open", { projectId }, projectStateSchema)
      ).toEqual(redone);
      expect(await frame(9, 500)).toEqual(reorderedPixels);

      const videoPath = join(media, "audio-provider.mkv");
      const generated = spawnSync(ffmpeg, [
        "-v",
        "error",
        "-y",
        "-threads",
        "1",
        "-f",
        "lavfi",
        "-i",
        "color=red:s=64x64:r=10:d=1",
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:sample_rate=48000:duration=1",
        "-c:v",
        "ffv1",
        "-c:a",
        "pcm_s16le",
        "-shortest",
        videoPath,
      ]);
      expect(generated.status, generated.stderr.toString()).toBe(0);
      const audioProject = await call(
        "project_create",
        {
          fps: 10,
          height: 64,
          name: "Public audio-bearing matteOnly",
          width: 64,
        },
        writeResultSchema
      );
      const audioId = audioProject.projectId;
      const audioState = await call(
        "project_open",
        { projectId: audioId },
        projectStateSchema
      );
      const videoTrack = audioState.project.tracks.find(
        (entry) => entry.trackType === "video"
      );
      if (!videoTrack) {
        throw new Error("audio-provider video track missing");
      }
      const imported = await call(
        "asset_import",
        {
          expectedRevision: 0,
          mediaType: "video",
          path: videoPath,
          projectId: audioId,
        },
        writeResultSchema
      );
      const [assetId] = imported.changedIds;
      const inserted = await call(
        "timeline_add_media",
        {
          assetId,
          durationMs: 1000,
          expectedRevision: 1,
          projectId: audioId,
          startMs: 0,
          trackId: videoTrack.id,
        },
        writeResultSchema
      );
      const [mediaId] = inserted.changedIds;
      expect(
        (await call("project_open", { projectId: audioId }, projectStateSchema))
          .project.assets[0]?.probe.hasAudio
      ).toBe(true);
      const audioRecipient = await call(
        "timeline_add_rectangle",
        {
          color: "#0000ff",
          durationMs: 1000,
          expectedRevision: 2,
          height: 64,
          projectId: audioId,
          startMs: 0,
          trackId: videoTrack.id,
          width: 64,
        },
        writeResultSchema
      );
      await call(
        "timeline_update_item",
        {
          expectedRevision: 3,
          itemId: audioRecipient.changedIds[0],
          matte: { channel: "alpha", sourceId: mediaId },
          projectId: audioId,
        },
        writeResultSchema
      );
      const audiovisual = async (revision: number) => {
        const queued = await call(
          "preview_render_range",
          {
            endMs: 1000,
            expectedRevision: revision,
            fps: 10,
            includeAudio: true,
            projectId: audioId,
            resolution: { height: 64, width: 64 },
            startMs: 0,
          },
          jobSchema
        );
        const completed = await terminal(queued.jobId);
        const resource = await client.readResource({
          uri: completed.artifactResource?.uri ?? "",
        });
        const [content] = resource.contents;
        if (!(content && "blob" in content)) {
          throw new Error("audio artifact blob missing");
        }
        const bytes = Buffer.from(content.blob, "base64");
        expect(bytes.length).toBe(completed.artifact?.sizeBytes);
        const decoded = spawnSync(
          ffmpeg,
          [
            "-v",
            "error",
            "-threads",
            "1",
            "-i",
            "pipe:0",
            "-map",
            "0:a:0",
            "-ac",
            "1",
            "-ar",
            "48000",
            "-f",
            "f32le",
            "pipe:1",
          ],
          { input: bytes, maxBuffer: 1024 * 1024 }
        );
        expect(decoded.status, decoded.stderr.toString()).toBe(0);
        const probed = spawnSync(
          ffprobe,
          [
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
            "pipe:0",
          ],
          { input: bytes }
        );
        expect(probed.status, probed.stderr.toString()).toBe(0);
        const probe = JSON.parse(probed.stdout.toString());
        const audio = probe.streams.find(
          (entry: { codec_type: string }) => entry.codec_type === "audio"
        );
        const video = probe.streams.find(
          (entry: { codec_type: string }) => entry.codec_type === "video"
        );
        expect(audio).toBeDefined();
        expect(video).toBeDefined();
        expect(Number(video.nb_frames)).toBe(10);
        expect(Math.abs(Number(video.duration) - 1)).toBeLessThanOrEqual(0.1);
        expect(
          Math.abs(Number(audio.duration) - Number(video.duration))
        ).toBeLessThanOrEqual(0.1);
        expect(
          Math.abs(Number(audio.start_time) - Number(video.start_time))
        ).toBeLessThanOrEqual(0.1);
        return { audio, pcm: decoded.stdout, video };
      };
      const audibleBefore = await audiovisual(4);
      expect(audibleBefore.pcm.length).toBeGreaterThanOrEqual(48_000 * 4);
      let energy = 0;
      for (let offset = 0; offset < audibleBefore.pcm.length; offset += 4) {
        energy += audibleBefore.pcm.readFloatLE(offset) ** 2;
      }
      expect(
        Math.sqrt(energy / (audibleBefore.pcm.length / 4))
      ).toBeGreaterThan(0.01);
      await call(
        "timeline_update_item",
        {
          expectedRevision: 4,
          itemId: mediaId,
          matteOnly: true,
          projectId: audioId,
        },
        writeResultSchema
      );
      const matteOnlyState = await call(
        "project_open",
        { projectId: audioId },
        projectStateSchema
      );
      expect(
        matteOnlyState.project.tracks.find(
          (entry) => entry.id === videoTrack.id
        )?.items[0]
      ).toHaveProperty("matteOnly", true);
      const audibleAfter = await audiovisual(5);
      expect(audibleAfter.pcm).toEqual(audibleBefore.pcm);
      process.stdout.write(
        `Referenced matteOnly Media audio: PCM bytes=${audibleAfter.pcm.length}, RMS=${Math.sqrt(energy / (audibleBefore.pcm.length / 4))}, frames=${audibleAfter.video.nb_frames}, duration=${audibleAfter.video.duration}, audioStart=${audibleAfter.audio.start_time}, audioDuration=${audibleAfter.audio.duration}\n`
      );
      expect(audibleAfter.audio).toEqual(audibleBefore.audio);
      expect(audibleAfter.video).toMatchObject({
        duration: audibleBefore.video.duration,
        nb_frames: audibleBefore.video.nb_frames,
        start_time: audibleBefore.video.start_time,
      });
    } finally {
      await client.close();
      rmSync(root, { force: true, recursive: true });
    }
  },
  60_000
);
