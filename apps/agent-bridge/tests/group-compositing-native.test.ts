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
import catalog from "../../../contracts/group-compositing-v1.json";
import soundEvents from "../../../contracts/semantic-sound-events-v1.json";
import {
  editDraftSchema,
  jobSchema,
  projectStateSchema,
  statusSchema,
  visualEffectSchema,
  writeResultSchema,
} from "../src/schemas";
import {
  alignedPcm,
  audiovisualFacts,
  decodedFrame,
  independentSsim,
} from "./group-compositing-media-oracle";
import { independentGroupPlate } from "./group-compositing-oracle";

const required =
  process.env.OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED === "1" ||
  process.env.OPENCUT_GOLDEN_REQUIRED === "1";
it.runIf(required)(
  "renders independent group-compositing plates through actual MCP reorder drafts history and reconnect",
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
      join(tmpdir(), "opencut-native-group-compositings-mcp-")
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
        name: "native-group-compositings-conformance",
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
        throw new Error("native MCP group-compositing job did not complete");
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
      const w = catalog.nativeWitness;
      const flash = visualEffectSchema.parse(w.flash),
        particles = visualEffectSchema.parse(w.particles);
      const blur = visualEffectSchema.parse({
        id: "positive",
        radiusPx: 1,
        type: "gaussian_blur",
      });
      const aStack = [flash, particles, blur],
        bStack = [particles, flash, blur];
      const status = await call("editor_get_status", {}, statusSchema);
      expect(status.projectSchemaVersion).toBe(
        soundEvents.projectSchemaVersion
      );
      expect(status.subsystems.rendering.ready).toBe(true);
      expect(status.capabilities).toContain("group_compositing_models_v1");
      expect(status.subsystems.rendering.capabilities).toContain(
        "group_compositing_v1"
      );
      const { projectId } = await call(
        "project_create",
        {
          fps: 10,
          height: 64,
          name: "Independent MCP aggregate plate",
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
              operation: "add_group",
              resultAlias: "owner",
              startMs: 0,
              trackId: initial.project.tracks[1]?.id,
            },
            {
              clip: { type: "composition_bounds" },
              effects: aStack,
              itemId: "@owner",
              operation: "update_item",
              transform2d: {
                anchor: { x: 0, y: 0 },
                opacity: 0.65,
                position: { unit: "pixels", x: 0, y: 0 },
                rotationDeg: 0,
                scaleX: 1,
                scaleY: 1,
                skewXDeg: 0,
                skewYDeg: 0,
              },
            },
            {
              color: "#00ff00",
              durationMs: 800,
              height: 24,
              operation: "add_rectangle",
              resultAlias: "child",
              startMs: 0,
              trackId: initial.project.tracks[1]?.id,
              transform: { opacity: 0.4, positionX: 4, positionY: 8, scale: 1 },
              width: 32,
            },
            {
              itemId: "@child",
              operation: "item_set_parent",
              parent: { id: "@owner", scope: "root" },
            },
          ],
          projectId,
        },
        writeResultSchema
      );
      const itemId = added.aliases.owner;
      if (!itemId) {
        throw new Error("aggregate alias missing");
      }
      const tonePath = join(media, "tone.wav");
      const generatedTone = spawnSync(ffmpeg, [
        "-v",
        "error",
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:sample_rate=48000:duration=0.8",
        "-c:a",
        "pcm_s16le",
        "-y",
        tonePath,
      ]);
      expect(generatedTone.status, generatedTone.stderr.toString()).toBe(0);
      const imported = await call(
        "asset_import",
        {
          expectedRevision: (await read()).project.revision,
          mediaType: "audio",
          path: tonePath,
          projectId,
        },
        writeResultSchema
      );
      const audioTrack = (await read()).project.tracks.find(
        (track) => track.trackType === "audio"
      );
      if (!audioTrack) {
        throw new Error("Actual audio track missing");
      }
      await call(
        "timeline_add_media",
        {
          assetId: imported.changedIds[0],
          durationMs: 800,
          expectedRevision: (await read()).project.revision,
          projectId,
          startMs: 0,
          trackId: audioTrack.id,
        },
        writeResultSchema
      );
      const baseline = await read();
      const unchanged = ["project.json", "history.json"].map((name) =>
        readFileSync(join(projects, projectId, name))
      );
      const invalid = { ...w.particles, count: 257 };
      const reject = async (
        name: string,
        fields: Record<string, unknown>,
        code = "INVALID_ARGUMENT"
      ) => {
        const result = await client.callTool({
          arguments: {
            expectedRevision: baseline.project.revision,
            projectId,
            ...fields,
          },
          name,
        });
        expect(result.isError, name).toBe(true);
        if (result.structuredContent) {
          expect(result.structuredContent).toMatchObject({
            error: { code, retryable: code === "REVISION_CONFLICT" },
          });
        } else {
          expect(result.content).toEqual(
            expect.arrayContaining([expect.objectContaining({ type: "text" })])
          );
        }
        expect((await read()).project).toEqual(baseline.project);
        for (const [i, fileName] of [
          "project.json",
          "history.json",
        ].entries()) {
          expect(readFileSync(join(projects, projectId, fileName))).toEqual(
            unchanged[i]
          );
        }
      };
      await catalog.clipCases
        .filter((entry) => !entry.accepted && entry.value !== null)
        .reduce(async (previous, { value: clip }) => {
          await previous;
          await reject("timeline_update_item", { clip, itemId });
          await ["timeline_batch_edit", "draft_create"].reduce(
            async (prior, name) => {
              await prior;
              await reject(name, {
                operations: [
                  { effects: bStack, itemId, operation: "update_item" },
                  { clip, itemId, operation: "update_item" },
                ],
              });
            },
            Promise.resolve()
          );
        }, Promise.resolve());
      await reject("timeline_update_item", { effects: [invalid], itemId });
      await reject("timeline_update_item", {
        clip: { extra: true, type: "composition_bounds" },
        itemId,
      });
      await reject("timeline_update_item", {
        clip: { type: "composition_bounds" },
        itemId: added.aliases.child,
      });
      await reject("timeline_update_item", { effects: [flash, flash], itemId });
      await reject(
        "timeline_update_item",
        { effects: [], itemId: "missing" },
        "ITEM_NOT_FOUND"
      );
      await reject(
        "timeline_update_item",
        {
          effects: [],
          expectedRevision: baseline.project.revision - 1,
          itemId,
        },
        "REVISION_CONFLICT"
      );
      await ["timeline_batch_edit", "draft_create"].reduce(
        async (previous, name) => {
          await previous;
          await reject(name, {
            operations: [
              { effects: bStack, itemId, operation: "update_item" },
              { effects: [invalid], itemId, operation: "update_item" },
            ],
          });
        },
        Promise.resolve()
      );
      let rangePcm: Buffer | undefined, exportPcm: Buffer | undefined;
      let generation = 0;
      const allIntents = async (declared: typeof aStack) => {
        const { revision } = (await read()).project;
        const range = await terminal(
          (
            await call(
              "preview_render_range",
              {
                endMs: 800,
                expectedRevision: revision,
                fps: 10,
                includeAudio: true,
                projectId,
                resolution: { height: 64, width: 64 },
                startMs: 200,
              },
              jobSchema
            )
          ).jobId
        );
        const resource = await client.readResource({
          uri: range.artifactResource?.uri ?? "",
        });
        const [content] = resource.contents;
        if (!(content && "blob" in content)) {
          throw new Error("Actual range artifact missing");
        }
        const rangePath = join(root, `range-${generation}.mp4`),
          exportName = `aggregate-${generation}.mp4`;
        generation += 1;
        const bytes = Buffer.from(content.blob, "base64");
        expect(bytes.length).toBe(range.artifact?.sizeBytes);
        writeFileSync(rangePath, bytes);
        await terminal(
          (
            await call(
              "project_export_video",
              {
                expectedRevision: revision,
                format: "mp4",
                overwrite: false,
                projectId,
                relativePath: exportName,
                resolution: "project",
              },
              jobSchema
            )
          ).jobId
        );
        const exportPath = join(exportsDirectory, exportName);
        const expected200 = converted(independentGroupPlate(declared, 200)),
          expected600 = converted(independentGroupPlate(declared, 600));
        expect(
          independentSsim(decodedFrame(ffmpeg, rangePath, 0), expected200)
        ).toBeGreaterThanOrEqual(0.99);
        expect(
          independentSsim(decodedFrame(ffmpeg, rangePath, 0.4), expected600)
        ).toBeGreaterThanOrEqual(0.99);
        expect(
          independentSsim(decodedFrame(ffmpeg, exportPath, 0.2), expected200)
        ).toBeGreaterThanOrEqual(0.99);
        expect(
          independentSsim(decodedFrame(ffmpeg, exportPath, 0.6), expected600)
        ).toBeGreaterThanOrEqual(0.99);
        const currentRange = audiovisualFacts(ffmpeg, ffprobe, rangePath, 6),
          currentExport = audiovisualFacts(ffmpeg, ffprobe, exportPath, 8);
        if (rangePcm) {
          alignedPcm(currentRange, rangePcm);
        } else {
          rangePcm = currentRange;
        }
        if (exportPcm) {
          alignedPcm(currentExport, exportPcm);
        } else {
          exportPcm = currentExport;
        }
      };
      const frame = async (
        declared: typeof aStack,
        draftId?: string,
        timeMs = 200
      ) => {
        rmSync(capturedPam, { force: true });
        const queued = await call(
          draftId ? "draft_preview_frame" : "preview_render_frame",
          {
            projectId,
            timeMs,
            ...(draftId
              ? { draftId }
              : { expectedRevision: (await read()).project.revision }),
          },
          jobSchema
        );
        const job = await terminal(queued.jobId);
        const resource = await client.readResource({
          uri: job.artifactResource?.uri ?? "",
        });
        const [content] = resource.contents;
        if (!(content && "blob" in content)) {
          throw new Error("native aggregate artifact missing");
        }
        const png = Buffer.from(content.blob, "base64");
        expect(png.length).toBe(job.artifact?.sizeBytes);
        const pam = readFileSync(capturedPam),
          marker = Buffer.from("ENDHDR\n");
        const header = pam.indexOf(marker) + marker.length;
        expect(header).toBeGreaterThan(marker.length);
        const expected = independentGroupPlate(declared, timeMs);
        close(
          pam.subarray(header),
          expected,
          "complete actual prepared aggregate plate"
        );
        const pixels = decode(png);
        close(
          pixels,
          converted(expected),
          "complete actual MCP independent converted plate"
        );
        return pixels;
      };
      const a = await frame(aStack);
      await allIntents(aStack);
      await [0, 700].reduce(async (previous, time) => {
        await previous;
        await frame(aStack, undefined, time);
      }, Promise.resolve());
      const before = await read();
      const files = ["project.json", "history.json"];
      const durable = files.map((name) =>
        readFileSync(join(projects, projectId, name))
      );
      const draft = await call(
        "draft_create",
        {
          expectedRevision: before.project.revision,
          operations: [{ effects: bStack, itemId, operation: "update_item" }],
          projectId,
        },
        editDraftSchema
      );
      const b = await frame(bStack, draft.id);
      expect(b.equals(a)).toBe(false);
      expect((await read()).project).toEqual(before.project);
      for (const [i, name] of files.entries()) {
        expect(readFileSync(join(projects, projectId, name))).toEqual(
          durable[i]
        );
      }
      await call(
        "draft_commit",
        {
          draftId: draft.id,
          expectedRevision: before.project.revision,
          projectId,
        },
        writeResultSchema
      );
      expect(await frame(bStack)).toEqual(b);
      await allIntents(bStack);
      await call(
        "project_undo",
        { expectedRevision: (await read()).project.revision, projectId },
        writeResultSchema
      );
      expect(await frame(aStack)).toEqual(a);
      await call(
        "project_redo",
        { expectedRevision: (await read()).project.revision, projectId },
        writeResultSchema
      );
      expect(await frame(bStack)).toEqual(b);
      const persisted = await read();
      await client.close();
      client = await connect();
      expect(await read()).toEqual(persisted);
      expect(await frame(bStack)).toEqual(b);
      const cStack = [flash, blur, particles];
      await call(
        "timeline_update_item",
        {
          effects: cStack,
          expectedRevision: (await read()).project.revision,
          itemId,
          projectId,
        },
        writeResultSchema
      );
      const c = await frame(cStack);
      expect(c.equals(a)).toBe(false);
      await call(
        "timeline_update_item",
        {
          clip: null,
          expectedRevision: (await read()).project.revision,
          itemId,
          projectId,
        },
        writeResultSchema
      );
      const owner = (await read()).project.tracks[1]?.items.find(
        (item) => item.id === itemId
      );
      expect(owner?.clip).toBeUndefined();
      expect(owner?.effects).toEqual(cStack);
      await frame(cStack);
    } finally {
      await client.close();
      rmSync(root, { force: true, recursive: true });
    }
  },
  60_000
);
