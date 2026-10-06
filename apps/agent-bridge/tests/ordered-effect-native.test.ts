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
import catalog from "../../../contracts/extended-visual-animation-v1.json";
import {
  editDraftSchema,
  jobSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";
import { independentOrderedEffectPlate } from "./ordered-effect-oracle";
import { verifyOrderedEffectWorkflow } from "./ordered-effect-workflow";

const required =
  process.env.OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED === "1" ||
  process.env.OPENCUT_GOLDEN_REQUIRED === "1";
it.runIf(required)(
  "renders independent ordered-effect plates through actual MCP reorder drafts history and reconnect",
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
    const root = mkdtempSync(
      join(tmpdir(), "opencut-native-ordered-effects-mcp-")
    );
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
        name: "native-ordered-effects-conformance",
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
        throw new Error("native MCP ordered-effect job did not complete");
      }
      await new Promise((resolveDelay) => setTimeout(resolveDelay, 100));
      return terminal(jobId, remaining - 1);
    };
    const decode = (bytes: Uint8Array) => {
      const result = spawnSync(
        ffmpeg,
        [
          "-v",
          "error",
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
    const converted = (pixels: Uint8Array) => {
      const pam = join(root, "independently-authored.pam"),
        png = join(root, "independently-converted.png");
      writeFileSync(
        pam,
        Buffer.concat([
          Buffer.from(
            "P7\nWIDTH 64\nHEIGHT 64\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n"
          ),
          pixels,
        ])
      );
      const result = spawnSync(ffmpeg, [
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
        pam,
        "-filter_complex_threads",
        "1",
        "-filter_complex",
        "[0:v]format=yuv420p[base0];[1:v]fps=10,settb=AVTB,setpts=PTS-STARTPTS+0/TB,format=rgba[plate];[base0][plate]overlay=format=auto:x=0:y=0:eof_action=pass[composed];[composed]scale=64:64:force_original_aspect_ratio=decrease,pad=64:64:(ow-iw)/2:(oh-ih)/2,format=yuv420p[video]",
        "-map",
        "[video]",
        "-frames:v",
        "1",
        "-y",
        png,
      ]);
      expect(result.status, result.stderr.toString()).toBe(0);
      return decode(readFileSync(png));
    };
    const close = (actual: Uint8Array, expected: Uint8Array, label: string) => {
      expect(actual.length, label).toBe(expected.length);
      const maximum = actual.reduce(
        (result, value, i) =>
          Math.max(result, Math.abs(value - (expected[i] ?? Number.NaN))),
        0
      );
      expect(maximum, label).toBeLessThanOrEqual(1);
    };
    try {
      await verifyOrderedEffectWorkflow(client, call);
      const f = catalog.orderedEffectCases;
      const status = await call("editor_get_status", {}, statusSchema);
      expect(status.projectSchemaVersion).toBe(36);
      expect(status.subsystems.rendering.ready).toBe(true);
      const { projectId } = await call(
        "project_create",
        {
          fps: 10,
          height: 64,
          name: "Independent MCP ordered-effect plate",
          width: 64,
        },
        writeResultSchema
      );
      const read = () =>
        call("project_open", { projectId }, projectStateSchema);
      const initial = await read();
      const added = await call(
        "timeline_batch_edit",
        {
          expectedRevision: 0,
          operations: [
            {
              durationMs: 800,
              operation: "add_shape",
              resultAlias: "leaf",
              startMs: 0,
              trackId: initial.project.tracks[1]?.id,
              ...f.source,
            },
            {
              effects: f.orders.washThenShade,
              itemId: "@leaf",
              operation: "update_item",
            },
            {
              effects: f.orders.shadeThenWash,
              itemId: "@leaf",
              operation: "update_item",
            },
          ],
          projectId,
        },
        writeResultSchema
      );
      const itemId = added.aliases.leaf;
      if (!itemId) {
        throw new Error("ordered-effect alias missing");
      }
      const stack = async () => {
        const item = (await read()).project.tracks[1]?.items.find(
          (value) => value.id === itemId
        );
        if (!(item && "effects" in item)) {
          throw new Error("ordered-effect shape missing");
        }
        return item.effects;
      };
      const frame = async (
        declared: Parameters<typeof independentOrderedEffectPlate>[0],
        draftId?: string,
        timeMs = 200
      ) => {
        rmSync(capturedPam, { force: true });
        const state = await read();
        const queued = await call(
          draftId ? "draft_preview_frame" : "preview_render_frame",
          {
            projectId,
            timeMs,
            ...(draftId
              ? { draftId }
              : { expectedRevision: state.project.revision }),
          },
          jobSchema
        );
        const job = await terminal(queued.jobId);
        const resource = await client.readResource({
          uri: job.artifactResource?.uri ?? "",
        });
        const [content] = resource.contents;
        if (!(content && "blob" in content)) {
          throw new Error("ordered-effect native artifact missing");
        }
        const png = Buffer.from(content.blob, "base64");
        expect(png.length).toBe(job.artifact?.sizeBytes);
        const pam = readFileSync(capturedPam),
          marker = Buffer.from("ENDHDR\n");
        const header = pam.indexOf(marker) + marker.length;
        expect(header).toBeGreaterThan(marker.length);
        const expected = independentOrderedEffectPlate(declared);
        close(pam.subarray(header), expected, "actual prepared complete plate");
        const pixels = decode(png);
        close(
          pixels,
          converted(expected),
          "actual MCP converted independent plate"
        );
        return pixels;
      };
      expect(await stack()).toEqual(f.orders.shadeThenWash);
      const a = await frame(f.orders.shadeThenWash);
      const standaloneState = await read();
      await call(
        "timeline_update_item",
        {
          effects: f.orders.washThenShade,
          expectedRevision: standaloneState.project.revision,
          itemId,
          projectId,
        },
        writeResultSchema
      );
      expect(await stack()).toEqual(f.orders.washThenShade);
      const standaloneB = await frame(f.orders.washThenShade);
      expect(standaloneB.equals(a)).toBe(false);
      const reversed = await read();
      await call(
        "project_undo",
        { expectedRevision: reversed.project.revision, projectId },
        writeResultSchema
      );
      expect(await stack()).toEqual(f.orders.shadeThenWash);
      expect((await frame(f.orders.shadeThenWash)).equals(a)).toBe(true);
      const undoneStandalone = await read();
      await call(
        "project_redo",
        { expectedRevision: undoneStandalone.project.revision, projectId },
        writeResultSchema
      );
      expect(await stack()).toEqual(f.orders.washThenShade);
      expect((await frame(f.orders.washThenShade)).equals(standaloneB)).toBe(
        true
      );
      const redoneStandalone = await read();
      await call(
        "project_undo",
        { expectedRevision: redoneStandalone.project.revision, projectId },
        writeResultSchema
      );
      expect(await stack()).toEqual(f.orders.shadeThenWash);
      const beforeDraft = await read();
      const durableBefore = ["project.json", "history.json"].map((name) =>
        readFileSync(join(projects, projectId, name))
      );
      const draft = await call(
        "draft_create",
        {
          expectedRevision: beforeDraft.project.revision,
          operations: [
            {
              effects: f.orders.washThenShade,
              itemId,
              operation: "update_item",
            },
          ],
          projectId,
        },
        editDraftSchema
      );
      const b = await frame(f.orders.washThenShade, draft.id);
      expect(b.equals(a)).toBe(false);
      expect(b.equals(standaloneB)).toBe(true);
      expect((await read()).project).toEqual(beforeDraft.project);
      for (const [i, name] of ["project.json", "history.json"].entries()) {
        expect(readFileSync(join(projects, projectId, name))).toEqual(
          durableBefore[i]
        );
      }
      await call(
        "draft_commit",
        {
          draftId: draft.id,
          expectedRevision: beforeDraft.project.revision,
          projectId,
        },
        writeResultSchema
      );
      expect(await stack()).toEqual(f.orders.washThenShade);
      expect(await frame(f.orders.washThenShade)).toEqual(b);
      await call(
        "project_undo",
        { expectedRevision: (await read()).project.revision, projectId },
        writeResultSchema
      );
      expect(await stack()).toEqual(f.orders.shadeThenWash);
      expect(await frame(f.orders.shadeThenWash)).toEqual(a);
      await call(
        "project_redo",
        { expectedRevision: (await read()).project.revision, projectId },
        writeResultSchema
      );
      expect(await stack()).toEqual(f.orders.washThenShade);
      expect(await frame(f.orders.washThenShade)).toEqual(b);
      const persisted = await read();
      await client.close();
      client = await connect();
      expect(await read()).toEqual(persisted);
      expect(await frame(f.orders.washThenShade)).toEqual(b);
      await [f.orders.glowThenWash, f.orders.washThenGlow].reduce(
        async (previous, declared) => {
          await previous;
          await call(
            "timeline_update_item",
            {
              effects: declared,
              expectedRevision: (await read()).project.revision,
              itemId,
              projectId,
            },
            writeResultSchema
          );
          expect(await stack()).toEqual(declared);
          await frame(declared);
        },
        Promise.resolve()
      );
    } finally {
      await client.close();
      rmSync(root, { force: true, recursive: true });
    }
  },
  60_000
);
