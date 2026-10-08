import { spawnSync } from "node:child_process";
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
import { join, resolve } from "node:path";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { expect, it } from "vitest";
import type { ZodType } from "zod/v4";
import audioAnalysis from "../../../contracts/audio-analysis-v1.json";
import audioBusDucking from "../../../contracts/audio-bus-ducking-v1.json";
import busCatalog from "../../../contracts/audio-buses-v1.json";
import catalog from "../../../contracts/masked-hero-reveal-v1.json";
import soundCatalog from "../../../contracts/semantic-sound-events-v1.json";
import {
  editDraftSchema,
  jobSchema,
  projectStateSchema,
  statusSchema,
  timelineItemSchema,
  writeResultSchema,
} from "../src/schemas";
import { independentSsim } from "./group-compositing-media-oracle";
import {
  convertedHero,
  decodedHeroVideo,
  heroBytes,
  heroDecodedFrame,
  heroMovie,
  heroPlate,
  heroReferenceRoot,
} from "./masked-hero-reveal-oracle";

const required =
  process.env.OPENCUT_GOLDEN_REQUIRED === "1" ||
  process.env.OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED === "1" ||
  process.env.OPENCUT_TRACK_MATTE_RENDER_REQUIRED === "1";
function inventory(root: string) {
  const files: Record<string, string> = {};
  const visit = (path: string) => {
    for (const name of readdirSync(path)) {
      const child = join(path, name);
      if (statSync(child).isDirectory()) {
        files[child] = "directory";
        visit(child);
      } else if (name !== ".lock") {
        files[child] = readFileSync(child).toString("base64");
      }
    }
  };
  visit(root);
  return files;
}
function assertRecipe(
  state: ReturnType<typeof projectStateSchema.parse>,
  ids: Record<string, string>,
  reversed: boolean
) {
  const p = state.project;
  expect(p.settings).toEqual(catalog.settings);
  expect(p.name).toBe("Masked hero reveal v1");
  expect(catalog.projectSchemaVersion).toBe(38);
  expect(p.schemaVersion).toBe(audioBusDucking.projectSchemaVersion);
  expect(p.audioBuses).toEqual(busCatalog.defaultBuses);
  expect(p.tracks.map((t) => t.items.length)).toEqual([0, 4, 1, 0]);
  for (const [index, role] of (
    ["owner", "provider", "probe", "hero"] as const
  ).entries()) {
    const recipe = catalog.roles[role];
    const expected: Record<string, unknown> = {
      ...structuredClone(recipe),
      durationMs: 800,
      hidden: false,
      id: ids[role],
      stackOrder: index,
      startMs: 0,
      transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
      type: recipe.kind,
    };
    expected.kind = undefined;
    expected.parentRole = undefined;
    expected.parent = undefined;
    if (role === "hero" || role === "probe") {
      expected.parent = { id: ids.owner, scope: "root" };
    }
    if (role === "hero") {
      expected.matte = { channel: "alpha", sourceId: ids.provider };
      if (reversed) {
        expected.effects = [
          catalog.roles.hero.effects[0],
          catalog.roles.hero.effects[2],
          catalog.roles.hero.effects[1],
        ];
      }
    }
    if (role !== "owner") {
      expected.keyframes = [];
    }
    for (const key of ["effects", "masks", "animationChannels"]) {
      if (
        Array.isArray(expected[key]) &&
        (expected[key] as unknown[]).length === 0
      ) {
        expected[key] = undefined;
      }
    }
    if (expected.matteOnly === false) {
      expected.matteOnly = undefined;
    }
    expect(p.tracks[1]?.items[index], `complete canonical ${role}`).toEqual(
      timelineItemSchema.parse(
        Object.fromEntries(
          Object.entries(expected).filter(([, value]) => value !== undefined)
        )
      )
    );
  }
  expect(p.assets).toHaveLength(1);
  const [asset] = p.assets;
  if (!asset) {
    throw new Error("canonical stereo asset missing");
  }
  expect(asset).toMatchObject({
    contentHash: { algorithm: "sha256", digest: catalog.audio.sourceSha256 },
    durationMs: 800,
    fileName: "source.wav",
    hasAudio: true,
    mediaType: "audio",
    probe: {
      audioChannels: 2,
      audioCodec: "pcm_s16le",
      audioSampleRateHz: 48_000,
      durationMs: 800,
      formatName: "wav",
      hasAudio: true,
      hasVideo: false,
    },
    sizeBytes: 153_644,
  });
  expect(asset.projectRelativePath).toBe(
    `assets/sha256/b6/${catalog.audio.sourceSha256}`
  );
  expect(p.tracks[2]?.items[0]).toEqual(
    timelineItemSchema.parse({
      assetId: asset.id,
      audio: { fadeInMs: 0, fadeOutMs: 0, muted: false, volume: 1 },
      durationMs: 800,
      hidden: false,
      id: ids.audio,
      keyframes: [],
      sourceInMs: 0,
      stackOrder: 0,
      startMs: 0,
      transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
      type: "media",
      zIndex: 0,
    })
  );
  expect(p.tracks[2]?.audioRole).toBe("unassigned");
  expect(p.tracks[2]?.ducking).toBeNull();
  for (const t of p.tracks) {
    expect(t.locked || t.hidden || t.muted).toBe(false);
  }
}
it.runIf(required).each([false, true])(
  "actual MCP authors exact hero (alias batch %s) with native media, resources, drafts, history and reconnect",
  async (batch) => {
    const ffmpeg = process.env.OPENCUT_FFMPEG_PATH ?? "ffmpeg",
      ffprobe = process.env.OPENCUT_FFPROBE_PATH ?? "ffprobe";
    expect(spawnSync(ffmpeg, ["-version"]).status).toBe(0);
    expect(spawnSync(ffprobe, ["-version"]).status).toBe(0);
    const root = mkdtempSync(join(tmpdir(), "opencut-masked-hero-mcp-")),
      projects = join(root, "projects"),
      media = join(root, "media"),
      exports = join(root, "exports");
    for (const path of [projects, media, exports]) {
      mkdirSync(path, { recursive: true });
    }
    const wav = join(media, "source.wav");
    copyFileSync(join(heroReferenceRoot, "source.wav"), wav);
    const captured = join(root, "actual-linear-scene.pam"),
      source = join(root, "capture.rs"),
      proxy = join(
        root,
        `capture${process.platform === "win32" ? ".exe" : ""}`
      );
    writeFileSync(
      source,
      `use std::{path::Path,process::{Command,Stdio}};
fn main(){let args:Vec<_>=std::env::args_os().skip(1).collect();for argument in &args {let path=Path::new(argument);if path.file_name()==Some(std::ffi::OsStr::new("linear-scene.pam")){std::fs::copy(path,${JSON.stringify(captured)}).expect("capture actual scene");}}let status=Command::new(${JSON.stringify(ffmpeg)}).args(args).stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit()).status().expect("real FFmpeg");std::process::exit(status.code().unwrap_or(1));}`
    );
    const compiled = spawnSync(process.env.RUSTC ?? "rustc", [
      "--edition",
      "2024",
      "--crate-name",
      "capture",
      source,
      "-o",
      proxy,
    ]);
    expect(compiled.status, compiled.stderr.toString()).toBe(0);
    const environment = {
      ...Object.fromEntries(
        Object.entries(process.env).filter(
          (entry): entry is [string, string] => entry[1] !== undefined
        )
      ),
      OPENCUT_ALLOWED_MEDIA_DIRS: media,
      OPENCUT_EXPORTS_DIR: exports,
      OPENCUT_FFMPEG_PATH: proxy,
      OPENCUT_FFPROBE_PATH: ffprobe,
      OPENCUT_HEADLESS_PATH:
        process.env.OPENCUT_TEST_HEADLESS_PATH ??
        resolve(import.meta.dirname, "../../../target/debug/opencut-headless"),
      OPENCUT_PROJECTS_DIR: projects,
    };
    const connect = async () => {
      const client = new Client({
        name: "masked-hero-reveal-conformance",
        version: "1",
      });
      await client.connect(
        new StdioClientTransport({
          args: ["run", resolve(import.meta.dirname, "../src/index.ts")],
          command: "bun",
          env: environment,
          stderr: "inherit",
        })
      );
      return client;
    };
    let client = await connect();
    const call = async <T>(
      name: string,
      input: Record<string, unknown>,
      schema: ZodType<T>
    ) => {
      const result = await client.request({
        method: "tools/call",
        params: { arguments: input, name },
      });
      expect(
        result.isError,
        `${name}: ${JSON.stringify({ content: result.content, structured: result.structuredContent })}`
      ).not.toBe(true);
      return schema.parse(result.structuredContent);
    };
    const terminal = (jobId: string) => {
      const poll = async (
        attempt: number
      ): Promise<ReturnType<typeof jobSchema.parse>> => {
        if (attempt === 200) {
          throw new Error("actual native hero job did not complete");
        }
        const job = await call("job_get_status", { jobId }, jobSchema);
        if (job.status === "completed") {
          return job;
        }
        expect(
          ["failed", "cancelled"].includes(job.status),
          JSON.stringify(job)
        ).toBe(false);
        await new Promise((done) => setTimeout(done, 100));
        return poll(attempt + 1);
      };
      return poll(0);
    };
    const artifact = async (jobId: string, path: string) => {
      const job = await terminal(jobId);
      const response = await client.readResource({
        uri: job.artifactResource?.uri ?? "",
      });
      const [content] = response.contents;
      if (!(content && "blob" in content)) {
        throw new Error("actual artifact resource missing");
      }
      const bytes = Buffer.from(content.blob, "base64");
      expect(bytes.length).toBe(job.artifact?.sizeBytes);
      writeFileSync(path, bytes);
      return bytes;
    };
    try {
      const status = await call("editor_get_status", {}, statusSchema);
      expect(status.projectSchemaVersion).toBe(
        audioBusDucking.projectSchemaVersion
      );
      expect(status.subsystems.rendering.ready).toBe(true);
      const { tools } = await client.request({ method: "tools/list" });
      const addedTools = [
        "speech_markers_generate",
        "audio_bus_set_route",
        "audio_track_route",
        soundCatalog.operation,
        "timeline_add_audio_event",
        "audio_bus_set_dsp",
        audioBusDucking.operation,
        audioAnalysis.tool,
      ];
      expect(
        tools.filter((tool) => !addedTools.includes(tool.name))
      ).toHaveLength(78);
      expect(
        tools.filter((tool) => tool.name === "speech_markers_generate")
      ).toHaveLength(1);
      for (const name of busCatalog.operations) {
        expect(tools.filter((tool) => tool.name === name)).toHaveLength(1);
      }

      const created = await call(
          "project_create",
          {
            name: catalog.operationTranscript.createProject.name,
            ...catalog.settings,
          },
          writeResultSchema
        ),
        { projectId } = created;
      const read = () =>
        call("project_open", { projectId }, projectStateSchema);
      const initial = await read();
      expect(initial.project.tracks[1]?.name).toBe("Overlay");
      expect(initial.project.tracks[2]?.name).toBe("Audio");
      const imported = await call(
        "asset_import",
        { expectedRevision: 0, mediaType: "audio", path: wav, projectId },
        writeResultSchema
      );
      const operations = JSON.parse(
        JSON.stringify(catalog.operationTranscript.aliasBatch)
          .replaceAll("{{overlayTrackId}}", initial.project.tracks[1]?.id ?? "")
          .replaceAll("{{audioTrackId}}", initial.project.tracks[2]?.id ?? "")
          .replaceAll("{{audioAssetId}}", imported.changedIds[0] ?? "")
      ) as Record<string, unknown>[];
      let ids: Record<string, string> = {};
      if (batch) {
        ids = (
          await call(
            "timeline_batch_edit",
            { expectedRevision: 1, operations, projectId },
            writeResultSchema
          )
        ).aliases;
      } else {
        const mapping: Record<string, string> = {
          add_group: "add_group",
          add_media: "timeline_add_media",
          add_shape: "timeline_add_shape",
          item_set_parent: "item_set_parent",
          item_set_z_index: "item_set_z_index",
          set_animation_channels: "timeline_set_animation_channels",
          update_item: "timeline_update_item",
        };
        await operations.reduce(async (previous, raw) => {
          await previous;
          const { resultAlias, operation, ...rest } = raw;
          let text = JSON.stringify(rest);
          for (const [role, id] of Object.entries(ids)) {
            text = text.replaceAll(`"@${role}"`, JSON.stringify(id));
          }
          const input = JSON.parse(text) as Record<string, unknown>;
          const tool = mapping[String(operation)];
          if (!tool) {
            throw new Error(`unmapped operation ${String(operation)}`);
          }
          const result = await call(
            tool,
            {
              expectedRevision: (await read()).project.revision,
              projectId,
              ...input,
            },
            writeResultSchema
          );
          if (typeof resultAlias === "string") {
            const [id] = result.changedIds;
            if (!id) {
              throw new Error("actual created ID missing");
            }
            ids[resultAlias] = id;
          }
        }, Promise.resolve());
      }
      expect(Object.keys(ids)).toHaveLength(5);
      const { hero } = ids;
      if (!hero) {
        throw new Error("actual hero alias missing");
      }
      const dir = join(projects, projectId);
      const baseline = await read();
      assertRecipe(baseline, ids, false);
      expect(baseline.durationMs).toBe(800);
      const frame = async (
        generation: string,
        time: number,
        draftId?: string
      ) => {
        rmSync(captured, { force: true });
        const job = await call(
          draftId ? "draft_preview_frame" : "preview_render_frame",
          {
            projectId,
            timeMs: time,
            ...(draftId
              ? { draftId }
              : { expectedRevision: (await read()).project.revision }),
          },
          jobSchema
        );
        const path = join(
          root,
          `actual-${batch}-${generation}-${time}-${draftId ?? "current"}.png`
        );
        await artifact(job.jobId, path);
        const pam = readFileSync(captured);
        const marker = Buffer.from("ENDHDR\n"),
          offset = pam.indexOf(marker) + marker.length;
        expect(offset).toBeGreaterThan(marker.length);
        heroBytes(
          pam.subarray(offset),
          heroPlate(generation, time),
          "MCP complete independent PAM"
        );
        const pixels = heroDecodedFrame(ffmpeg, path, 0);
        heroBytes(
          pixels,
          convertedHero(ffmpeg, root, generation, time),
          "MCP independent PNG"
        );
        return pixels;
      };
      const movies = async (generation: string, label: string) => {
        const paths = new Map<number, string>();
        const signatures: Record<string, { video: Buffer; audio: Buffer }> = {};
        await [0, 200].reduce(async (previous, start) => {
          await previous;
          const job = await call(
            "preview_render_range",
            {
              endMs: 800,
              expectedRevision: (await read()).project.revision,
              fps: 10,
              includeAudio: true,
              projectId,
              resolution: { height: 64, width: 64 },
              startMs: start,
            },
            jobSchema
          );
          const path = join(
            root,
            `range-${batch}-${generation}-${label}-${start}.mp4`
          );
          await artifact(job.jobId, path);
          const audio = heroMovie(
            ffmpeg,
            ffprobe,
            root,
            path,
            generation,
            start
          );
          signatures[String(start)] = {
            audio,
            video: decodedHeroVideo(ffmpeg, path),
          };
          paths.set(start, path);
        }, Promise.resolve());
        const relativePath = `hero-${batch}-${generation}-${label}.mp4`;
        const job = await call(
          "project_export_video",
          {
            expectedRevision: (await read()).project.revision,
            format: "mp4",
            overwrite: false,
            projectId,
            relativePath,
            resolution: "project",
          },
          jobSchema
        );
        await terminal(job.jobId);
        const output = join(exports, relativePath);
        const audio = heroMovie(ffmpeg, ffprobe, root, output, generation, 0);
        signatures.export = { audio, video: decodedHeroVideo(ffmpeg, output) };
        const full = paths.get(0),
          partial = paths.get(200);
        if (!(full && partial)) {
          throw new Error("complete range contexts missing");
        }
        for (let t = 0; t < 800; t += 100) {
          expect(
            independentSsim(
              heroDecodedFrame(ffmpeg, full, t / 1000),
              heroDecodedFrame(ffmpeg, output, t / 1000)
            )
          ).toBeGreaterThanOrEqual(0.99);
          if (t >= 200) {
            const pixels = heroDecodedFrame(ffmpeg, partial, (t - 200) / 1000);
            expect(
              independentSsim(pixels, heroDecodedFrame(ffmpeg, full, t / 1000))
            ).toBeGreaterThanOrEqual(0.99);
            expect(
              independentSsim(
                pixels,
                heroDecodedFrame(ffmpeg, output, t / 1000)
              )
            ).toBeGreaterThanOrEqual(0.99);
          }
        }
        return signatures;
      };
      const sameMovies = (
        a: Awaited<ReturnType<typeof movies>>,
        b: Awaited<ReturnType<typeof movies>>
      ) => {
        expect(Object.keys(a)).toEqual(Object.keys(b));
        for (const key of Object.keys(a)) {
          const left = a[key],
            right = b[key];
          if (!(left && right)) {
            throw new Error("missing decoded movie context");
          }
          expect(
            left.video.equals(right.video),
            `exact decoded video ${key}`
          ).toBe(true);
          expect(
            left.audio.equals(right.audio),
            `exact stereo audio ${key}`
          ).toBe(true);
        }
      };
      await catalog.plates.samples.reduce(async (previous, time) => {
        await previous;
        await frame("baseline", time);
      }, Promise.resolve());
      expect(await frame("baseline", 400)).toEqual(
        await frame("baseline", 400)
      );
      const cold = await movies("baseline", "cold");
      sameMovies(cold, await movies("baseline", "warm"));
      const effects = [
        catalog.roles.hero.effects[0],
        catalog.roles.hero.effects[2],
        catalog.roles.hero.effects[1],
      ];
      const reorder = { effects, itemId: hero, operation: "update_item" };
      const durable = inventory(dir);
      const draft = await call(
        "draft_create",
        {
          expectedRevision: baseline.project.revision,
          operations: [
            reorder,
            {
              busId: "sfx",
              defaultGainDb: -24,
              event: "impact",
              operation: soundCatalog.operation,
              variantAssetIds: [imported.changedIds[0]],
              variantSeed: 1,
            },
          ],
          projectId,
        },
        editDraftSchema
      );
      expect(await read()).toEqual(baseline);
      for (const [path, bytes] of Object.entries(durable)) {
        if (statSync(path).isDirectory()) {
          expect(bytes).toBe("directory");
        } else {
          expect(readFileSync(path).toString("base64")).toBe(bytes);
        }
      }
      await catalog.plates.samples.reduce(async (previous, time) => {
        await previous;
        await frame("reverse", time, draft.id);
      }, Promise.resolve());
      const badChannels = structuredClone(catalog.roles.hero.animationChannels);
      const terminalKey = badChannels[0]?.keyframes.at(-1);
      if (!terminalKey) {
        throw new Error("canonical terminal key missing");
      }
      terminalKey.timeMs = 800;
      const invalid = {
        animationChannels: badChannels,
        itemId: hero,
        operation: "set_animation_channels",
      };
      const stable = inventory(dir);
      const reject = async (
        name: string,
        input: Record<string, unknown>,
        code: string,
        retryable = false
      ) => {
        const result = await client.request({
          method: "tools/call",
          params: {
            arguments: {
              expectedRevision: baseline.project.revision,
              projectId,
              ...input,
            },
            name,
          },
        });
        expect(result.isError).toBe(true);
        expect(result.structuredContent).toMatchObject({
          error: { code, retryable },
        });
        expect(await read()).toEqual(baseline);
        expect(inventory(dir)).toEqual(stable);
      };
      await reject(
        "timeline_set_animation_channels",
        { animationChannels: badChannels, itemId: hero },
        "INVALID_ARGUMENT"
      );
      await reject(
        "timeline_update_item",
        { itemId: hero, matte: { channel: "alpha", sourceId: "missing" } },
        "ITEM_NOT_FOUND"
      );
      await reject(
        "timeline_update_item",
        {
          effects,
          expectedRevision: baseline.project.revision - 1,
          itemId: hero,
        },
        "REVISION_CONFLICT",
        true
      );
      await reject(
        "timeline_batch_edit",
        { operations: [reorder, invalid] },
        "INVALID_ARGUMENT"
      );
      await reject(
        "draft_update",
        { draftId: draft.id, operations: [invalid] },
        "INVALID_ARGUMENT"
      );
      await call(
        "draft_commit",
        {
          draftId: draft.id,
          expectedRevision: (await read()).project.revision,
          projectId,
        },
        writeResultSchema
      );
      const committed = await movies("reverse", "committed");
      const reversed = await frame("reverse", 400);
      await call(
        "project_undo",
        { expectedRevision: (await read()).project.revision, projectId },
        writeResultSchema
      );
      await frame("baseline", 400);
      await call(
        "project_redo",
        { expectedRevision: (await read()).project.revision, projectId },
        writeResultSchema
      );
      expect(await frame("reverse", 400)).toEqual(reversed);
      const persisted = await read();
      assertRecipe(persisted, ids, true);
      await client.close();
      client = await connect();
      expect(await read()).toEqual(persisted);
      expect(await frame("reverse", 400)).toEqual(reversed);
      sameMovies(committed, await movies("reverse", "reopened"));
      const audioTrack = persisted.project.tracks.find(
        (track) => track.trackType === "audio"
      );
      if (!audioTrack) {
        throw new Error("Canonical native audio track missing");
      }
      await call(
        "timeline_batch_edit",
        {
          expectedRevision: persisted.project.revision,
          operations: [
            {
              busId: "sfx",
              defaultGainDb: -120,
              event: "impact",
              operation: soundCatalog.operation,
              variantAssetIds: [imported.changedIds[0]],
              variantSeed: soundCatalog.maximumVariantSeed,
            },
            {
              busId: "music",
              operation: "audio_bus_set_route",
              outputBusId: "sfx",
            },
            {
              busId: "music",
              operation: "audio_track_route",
              scope: "root",
              trackId: audioTrack.id,
            },
          ],
          projectId,
        },
        writeResultSchema
      );
      const busRouted = await read();
      expect(busRouted.project.soundDefinitions).toEqual([
        {
          busId: "sfx",
          defaultGainDb: -120,
          event: "impact",
          variantAssetIds: [imported.changedIds[0]],
          variantSeed: soundCatalog.maximumVariantSeed,
        },
      ]);
      expect(busRouted.project.audioBuses[1]?.outputBusId).toBe("sfx");
      expect(
        busRouted.project.tracks.find((track) => track.id === audioTrack.id)
          ?.audioBusId
      ).toBe("music");
      expect(
        busRouted.project.tracks.map(
          ({ audioBusId: _route, ...track }) => track
        )
      ).toEqual(persisted.project.tracks);
      expect(await frame("reverse", 400)).toEqual(reversed);
      sameMovies(committed, await movies("reverse", "bus-routed"));
      await client.close();
      client = await connect();
      expect(await read()).toEqual(busRouted);
      expect(await frame("reverse", 400)).toEqual(reversed);
      const busDraft = await call(
        "draft_create",
        {
          expectedRevision: busRouted.project.revision,
          operations: [
            {
              busId: "voiceover",
              operation: "audio_track_route",
              scope: "root",
              trackId: audioTrack.id,
            },
          ],
          projectId,
        },
        editDraftSchema
      );
      expect(await frame("reverse", 400, busDraft.id)).toEqual(reversed);
      expect(await read()).toEqual(busRouted);
      await call(
        "draft_commit",
        {
          draftId: busDraft.id,
          expectedRevision: busRouted.project.revision,
          projectId,
        },
        writeResultSchema
      );
      sameMovies(committed, await movies("reverse", "bus-draft-committed"));
      const beforeLegacy = await read();
      const legacy = structuredClone(beforeLegacy.project) as unknown as Record<
        string,
        unknown
      >;
      legacy.schemaVersion = 38;
      Reflect.deleteProperty(legacy, "audioBuses");
      Reflect.deleteProperty(legacy, "soundDefinitions");
      for (const track of legacy.tracks as Record<string, unknown>[]) {
        Reflect.deleteProperty(track, "audioBusId");
      }
      writeFileSync(join(dir, "project.json"), JSON.stringify(legacy, null, 2));
      const adopted = await read();
      expect(adopted.project.revision).toBe(beforeLegacy.project.revision);
      expect(adopted.project.schemaVersion).toBe(
        audioBusDucking.projectSchemaVersion
      );
      expect(adopted.project.soundDefinitions).toEqual([]);
      expect(adopted.project.audioBuses).toEqual(busCatalog.defaultBuses);
      const oldFields = structuredClone(adopted.project) as unknown as Record<
        string,
        unknown
      >;
      oldFields.schemaVersion = 38;
      Reflect.deleteProperty(oldFields, "audioBuses");
      Reflect.deleteProperty(oldFields, "soundDefinitions");
      expect(oldFields).toEqual(legacy);
      expect(await frame("reverse", 400)).toEqual(reversed);
      sameMovies(committed, await movies("reverse", "legacy-adopted"));
    } finally {
      await client.close();
      rmSync(root, { force: true, recursive: true });
    }
  },
  120_000
);
