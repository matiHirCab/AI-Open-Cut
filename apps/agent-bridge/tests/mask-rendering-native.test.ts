import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { expect, it } from "vitest";
import type { ZodType } from "zod/v4";
import {
  jobSchema,
  maskSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

const required =
  process.env.OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED === "1" ||
  process.env.OPENCUT_GOLDEN_REQUIRED === "1";
it.runIf(required)(
  "renders animated masks through actual MCP artifacts with independent color and revision controls",
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
    const root = mkdtempSync(join(tmpdir(), "opencut-native-mask-mcp-"));
    const projects = join(root, "projects"),
      media = join(root, "media"),
      exportsDirectory = join(root, "exports");
    for (const path of [projects, media, exportsDirectory]) {
      mkdirSync(path, { recursive: true });
    }
    const environment = {
      ...Object.fromEntries(
        Object.entries(process.env).filter(
          (entry): entry is [string, string] => entry[1] !== undefined
        )
      ),
      OPENCUT_ALLOWED_MEDIA_DIRS: media,
      OPENCUT_EXPORTS_DIR: exportsDirectory,
      OPENCUT_FFMPEG_PATH: ffmpeg,
      OPENCUT_FFPROBE_PATH: ffprobe,
      OPENCUT_HEADLESS_PATH:
        process.env.OPENCUT_TEST_HEADLESS_PATH ??
        resolve(import.meta.dirname, "../../../target/debug/opencut-headless"),
      OPENCUT_PROJECTS_DIR: projects,
    };
    const connect = async () => {
      const next = new Client({
        name: "native-mask-conformance",
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
        throw new Error("native MCP mask job did not complete");
      }
      await new Promise((resolveDelay) => setTimeout(resolveDelay, 100));
      return terminal(jobId, remaining - 1);
    };
    try {
      const status = await call("editor_get_status", {}, statusSchema);
      expect(status.subsystems.rendering.ready).toBe(true);
      expect(status.subsystems.rendering.capabilities).toContain(
        "mask_rendering_v1"
      );
      const { projectId } = await call(
        "project_create",
        { fps: 10, height: 64, name: "Independent MCP mask pixels", width: 64 },
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
          color: "#ff0000",
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
      const frame = async (revision: number, timeMs: number) => {
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
        return decoded.stdout;
      };
      const pixel = (bytes: Buffer, x: number, y: number) =>
        Array.from(bytes.subarray((y * 64 + x) * 3, (y * 64 + x) * 3 + 3));
      const red = (bytes: Buffer, x: number, y: number) => {
        const [r, g, b] = pixel(bytes, x, y);
        expect(r).toBeGreaterThan(100);
        expect(g).toBeLessThan(30);
        expect(b).toBeLessThan(30);
      };
      const black = (bytes: Buffer, x: number, y: number) => {
        expect(pixel(bytes, x, y).every((value) => value < 5)).toBe(true);
      };
      const baseline = await frame(1, 500);
      red(baseline, 4, 16);
      red(baseline, 48, 16);
      const mask = maskSchema.parse({
        channel: "alpha",
        expansionPx: 0,
        featherPx: 0,
        id: "cutout",
        inverted: false,
        operation: "add",
        source: {
          paint: { color: { a: 1, b: 1, g: 1, r: 1 }, type: "solid" },
          path: {
            commands: [
              { to: { x: 0, y: 0 }, type: "moveTo" },
              { to: { x: 32, y: 0 }, type: "lineTo" },
              { to: { x: 32, y: 64 }, type: "lineTo" },
              { to: { x: 0, y: 64 }, type: "lineTo" },
              { type: "close" },
            ],
            fillRule: "nonzero",
          },
          type: "path",
        },
        transform: {
          anchor: { x: 0, y: 0 },
          opacity: 1,
          position: { unit: "pixels", x: 0, y: 0 },
          rotationDeg: 0,
          scaleX: 1,
          scaleY: 1,
          skewXDeg: 0,
          skewYDeg: 0,
        },
      });
      const animationChannels = [
        {
          keyframes: [
            { curve: "linear", timeMs: 0, value: { type: "scalar", value: 0 } },
            { curve: "hold", timeMs: 500, value: { type: "scalar", value: 8 } },
          ],
          property: "mask.transform.position_x",
          target: { id: "cutout", kind: "mask", scope: "root" },
        },
      ];
      await call(
        "timeline_batch_edit",
        {
          expectedRevision: 1,
          operations: [
            { itemId, masks: [mask], operation: "update_item" },
            { animationChannels, itemId, operation: "set_animation_channels" },
          ],
          projectId,
        },
        writeResultSchema
      );
      const first = await frame(2, 0);
      red(first, 4, 16);
      black(first, 48, 16);
      const shifted = await frame(2, 500);
      black(shifted, 4, 16);
      red(shifted, 16, 16);
      black(shifted, 48, 16);
      expect(shifted.equals(baseline)).toBe(false);
      expect(await frame(2, 500)).toEqual(shifted);
      await client.close();
      client = await connect();
      expect(
        (await call("project_open", { projectId }, projectStateSchema)).project
          .revision
      ).toBe(2);
      expect(await frame(2, 500)).toEqual(shifted);
      await call(
        "timeline_update_item",
        { color: "#0000ff", expectedRevision: 2, itemId, projectId },
        writeResultSchema
      );
      const blue = await frame(3, 500);
      const [r, g, b] = pixel(blue, 16, 16);
      expect(b).toBeGreaterThan(100);
      expect(r).toBeLessThan(30);
      expect(g).toBeLessThan(30);
      black(blue, 4, 16);
      black(blue, 48, 16);
      expect(blue.equals(shifted)).toBe(false);
      await client.close();
      client = await connect();
      expect(await frame(3, 500)).toEqual(blue);
    } finally {
      await client.close();
      rmSync(root, { force: true, recursive: true });
    }
  },
  60_000
);
