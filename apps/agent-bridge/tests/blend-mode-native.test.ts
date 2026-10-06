import { spawnSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
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
import contract from "../../../contracts/blend-modes-v1.json";
import {
  editDraftSchema,
  jobSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";
import { verifyBlendModeWorkflow } from "./blend-mode-workflow";

const required =
  process.env.OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED === "1" ||
  process.env.OPENCUT_GOLDEN_REQUIRED === "1";
it.runIf(required)(
  "renders seven blend modes through actual MCP artifacts with full independent plates and public lifecycle",
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
    const root = mkdtempSync(join(tmpdir(), "opencut-native-blend-mcp-"));
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
        name: "native-blend-conformance",
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
        throw new Error("native MCP blend job did not complete");
      }
      await new Promise((resolveDelay) => setTimeout(resolveDelay, 100));
      return terminal(jobId, remaining - 1);
    };
    try {
      await verifyBlendModeWorkflow(client, call);
      const status = await call("editor_get_status", {}, statusSchema);
      expect(status.subsystems.rendering.ready).toBe(true);
      expect(status.subsystems.rendering.capabilities).toContain(
        "blend_modes_v1"
      );
      const { projectId } = await call(
        "project_create",
        {
          fps: 10,
          height: 64,
          name: "Independent MCP blend plates",
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
      const add = (alias: string, color: string, opacity: number) => ({
        color,
        durationMs: 1000,
        height: 64,
        opacity,
        operation: "add_rectangle",
        resultAlias: alias,
        startMs: 0,
        trackId,
        transform: { opacity, positionX: 0, positionY: 0, scale: 1 },
        width: 64,
      });
      const background = add("background", "#80c040", 1);
      Reflect.deleteProperty(background, "opacity");
      const foreground = add("foreground", "#e06020", 0.5);
      Reflect.deleteProperty(foreground, "opacity");
      const created = await call(
        "timeline_batch_edit",
        {
          expectedRevision: 0,
          operations: [
            background,
            foreground,
            {
              effects: [
                {
                  color: { a: 0, b: 0, g: 0, r: 0 },
                  id: "identity",
                  type: "color_tint",
                },
              ],
              itemId: "@foreground",
              operation: "update_item",
            },
          ],
          projectId,
        },
        writeResultSchema
      );
      const itemId = created.aliases.foreground;
      const decode = (bytes: Buffer) => {
        const result = spawnSync(
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
        expect(result.status, result.stderr.toString()).toBe(0);
        expect(result.stdout.length).toBe(64 * 64 * 3);
        return result.stdout;
      };
      const frame = async (frameRevision: number, draftId?: string) => {
        rmSync(capturedPam, { force: true });
        const queued = await call(
          draftId ? "draft_preview_frame" : "preview_render_frame",
          {
            projectId,
            timeMs: 400,
            ...(draftId ? { draftId } : { expectedRevision: frameRevision }),
          },
          jobSchema
        );
        const completed = await terminal(queued.jobId);
        const resource = await client.readResource({
          uri: completed.artifactResource?.uri ?? "",
        });
        const [content] = resource.contents;
        if (!(content && "blob" in content)) {
          throw new Error("blend artifact blob missing");
        }
        const bytes = Buffer.from(content.blob, "base64");
        expect(bytes.length).toBe(completed.artifact?.sizeBytes);
        const prepared = readFileSync(capturedPam);
        const end = prepared.indexOf("ENDHDR\n") + 7;
        expect(end).toBeGreaterThan(7);
        expect(prepared.subarray(0, end).toString()).toContain(
          "WIDTH 64\nHEIGHT 64\nDEPTH 4\n"
        );
        const raw = prepared.subarray(end);
        expect(raw.length).toBe(64 * 64 * 4);
        return { decoded: decode(bytes), raw };
      };
      const authoredRgb = (mode: string, rgb: readonly number[]) => {
        const raw = Buffer.alloc(64 * 64 * 4);
        for (let i = 0; i < 64 * 64; i += 1) {
          raw.set([...rgb, 255], i * 4);
        }
        const path = join(root, `independent-${mode}.pam`),
          output = join(root, `independent-${mode}.png`);
        writeFileSync(
          path,
          Buffer.concat([
            Buffer.from(
              "P7\nWIDTH 64\nHEIGHT 64\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n"
            ),
            raw,
          ])
        );
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
          path,
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
        return { decoded: decode(readFileSync(output)), raw };
      };
      const authored = (mode: string) =>
        authoredRgb(
          mode,
          contract.numericOracles.nativeAuthoredColored
            .rawSrgb8BeforeFinalCodec[
            mode as keyof typeof contract.numericOracles.nativeAuthoredColored.rawSrgb8BeforeFinalCodec
          ]
        );
      const encoded = (value: number) =>
        Math.round(
          255 *
            (value <= 0.003_130_8
              ? 12.92 * value
              : 1.055 * value ** (1 / 2.4) - 0.055)
        );
      const assertPlate = (
        actual: { raw: Buffer; decoded: Buffer },
        expected: { raw: Buffer; decoded: Buffer }
      ) => {
        for (let i = 0; i < actual.raw.length; i += 1) {
          expect(
            Math.abs((actual.raw[i] ?? 0) - (expected.raw[i] ?? 0)),
            `raw full pixel byte ${i}`
          ).toBeLessThanOrEqual(1);
        }
        for (let i = 0; i < actual.decoded.length; i += 1) {
          expect(
            Math.abs((actual.decoded[i] ?? 0) - (expected.decoded[i] ?? 0)),
            `converted full pixel byte ${i}`
          ).toBeLessThanOrEqual(1);
        }
      };
      let { revision } = created;
      await contract.fields.blendMode.values.reduce(async (previous, mode) => {
        await previous;
        const reference = authored(mode);
        const draft = await call(
          "draft_create",
          {
            expectedRevision: revision,
            operations: [{ blendMode: mode, itemId, operation: "update_item" }],
            projectId,
          },
          editDraftSchema
        );
        assertPlate(await frame(revision, draft.id), reference);
        const committed = await call(
          "draft_commit",
          { draftId: draft.id, expectedRevision: revision, projectId },
          writeResultSchema
        );
        ({ revision } = committed);
        assertPlate(await frame(revision), reference);
        const persisted = await call(
          "project_open",
          { projectId },
          projectStateSchema
        );
        const item = persisted.project.tracks[1]?.items.find(
          (value) => value.id === itemId
        );
        if (mode === "normal") {
          expect(item).not.toHaveProperty("blendMode");
        } else {
          expect(item).toHaveProperty("blendMode", mode);
        }
      }, Promise.resolve());
      const before = await frame(revision);
      const undone = await call(
        "project_undo",
        { expectedRevision: revision, projectId },
        writeResultSchema
      );
      const redone = await call(
        "project_redo",
        { expectedRevision: undone.revision, projectId },
        writeResultSchema
      );
      ({ revision } = redone);
      expect((await frame(revision)).decoded).toEqual(before.decoded);
      await client.close();
      client = await connect();
      expect((await frame(revision)).decoded).toEqual(before.decoded);
      const second = await call(
        "timeline_batch_edit",
        {
          expectedRevision: revision,
          operations: [
            {
              ...foreground,
              color: "#2080e0",
              resultAlias: "screen",
              transform: {
                opacity: 0.375,
                positionX: 0,
                positionY: 0,
                scale: 1,
              },
            },
            {
              blendMode: "screen",
              itemId: "@screen",
              operation: "update_item",
            },
            { blendMode: "multiply", itemId, operation: "update_item" },
          ],
          projectId,
        },
        writeResultSchema
      );
      ({ revision } = second);
      const screenId = second.aliases.screen;
      const orderOracle = contract.numericOracles.nativePublicOrder;
      const ab = authoredRgb(
        "multiply-screen",
        orderOracle.multiplyThenScreen.slice(0, 3).map(encoded)
      );
      const ba = authoredRgb(
        "screen-multiply",
        orderOracle.screenThenMultiply.slice(0, 3).map(encoded)
      );
      const firstOrder = await frame(revision);
      assertPlate(firstOrder, ab);
      const reordered = await call(
        "item_reorder",
        { expectedRevision: revision, index: 1, itemId: screenId, projectId },
        writeResultSchema
      );
      ({ revision } = reordered);
      const secondOrder = await frame(revision);
      assertPlate(secondOrder, ba);
      expect(secondOrder.decoded).not.toEqual(firstOrder.decoded);
      const undoOrder = await call(
        "project_undo",
        { expectedRevision: revision, projectId },
        writeResultSchema
      );
      ({ revision } = undoOrder);
      assertPlate(await frame(revision), ab);
      const redoOrder = await call(
        "project_redo",
        { expectedRevision: revision, projectId },
        writeResultSchema
      );
      ({ revision } = redoOrder);
      assertPlate(await frame(revision), ba);
      const persistedOrder = await call(
        "project_open",
        { projectId },
        projectStateSchema
      );
      expect(persistedOrder.project.tracks[1]?.items[1]?.id).toBe(screenId);
      await client.close();
      client = await connect();
      expect(
        await call("project_open", { projectId }, projectStateSchema)
      ).toEqual(persistedOrder);
      assertPlate(await frame(revision), ba);
    } finally {
      await client.close();
      rmSync(root, { force: true, recursive: true });
    }
  },
  60_000
);
