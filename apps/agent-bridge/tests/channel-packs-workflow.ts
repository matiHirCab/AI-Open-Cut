/** Genuine native/FFmpeg acceptance. Run explicitly with a built source or assembled headless binary. */
import { deepStrictEqual, equal, ok } from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import {
  mkdtemp,
  readdir,
  readFile,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import contract from "../../../contracts/channel-style-packs-v1.json";
import narrationBlocks from "../../../contracts/narration-blocks-v1.json";
import narrationFixture from "../../../contracts/narration-profiles-v1.json";
import {
  channelApplyResultSchema,
  channelCaptureResultSchema,
  channelPackSchema,
} from "../src/channel-packs/contract";
import { loadBridgeConfig } from "../src/config";
import { HeadlessClient } from "../src/headless";
import {
  artifactSchema,
  componentFieldsSchema,
  narrationProfileSchema,
  narrationProfileSettingsSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";

const executable = process.env.OPENCUT_HEADLESS_PATH;
ok(
  executable,
  "Set OPENCUT_HEADLESS_PATH; native channel acceptance cannot skip"
);
const root = await mkdtemp(join(tmpdir(), "opencut-channel-native-"));
const store = join(root, "projects");
const environment: NodeJS.ProcessEnv = {
  ...process.env,
  OPENCUT_ALLOWED_MEDIA_DIRS: root,
  OPENCUT_CONFIG_ROOT: root,
  OPENCUT_EXPORTS_DIR: join(root, "exports"),
  OPENCUT_HEADLESS_PATH: executable,
  OPENCUT_PROJECTS_DIR: store,
};
let headless = new HeadlessClient(loadBridgeConfig(environment));
const cli = fileURLToPath(
  new URL("../src/channel-packs-cli.ts", import.meta.url)
);
const sha = (bytes: Uint8Array) =>
  createHash("sha256").update(bytes).digest("hex");

let cleanupConfirmed = true;
const processOutput = (binary: string, args: string[], input?: string) =>
  new Promise<{ status: number | null; stdout: Buffer; stderr: Buffer }>(
    (resolve, reject) => {
      const child = spawn(binary, args, { env: environment, stdio: "pipe" });
      const out: Buffer[] = [];
      const err: Buffer[] = [];
      let bytes = 0;
      let stopping = false;
      const requestStop = () => {
        if (stopping) {
          return;
        }
        stopping = true;
        // CLI interruption closes its existing native owner and awaits its
        // active command. Never kill only the CLI and remove live native files.
        child.kill("SIGTERM");
        const deadline = setTimeout(() => {
          cleanupConfirmed = false;
          child.unref();
          child.stdin.destroy();
          child.stdout.destroy();
          child.stderr.destroy();
          reject(
            new Error(
              `Native subprocess shutdown unconfirmed; preserve ${root}`
            )
          );
        }, 10_000);
        child.once("close", () => clearTimeout(deadline));
      };
      const timer = setTimeout(requestStop, 120_000);
      child.stdout.on("data", (chunk: Buffer) => {
        bytes += chunk.length;
        if (bytes > 16 * 1024 * 1024) {
          requestStop();
        } else {
          out.push(chunk);
        }
      });
      child.stderr.on("data", (chunk: Buffer) => {
        if (Buffer.concat(err).length < 16_384) {
          err.push(chunk);
        }
      });
      child.on("error", (error) => {
        clearTimeout(timer);
        reject(error);
      });
      child.on("close", (status) => {
        clearTimeout(timer);
        resolve({
          status,
          stderr: Buffer.concat(err),
          stdout: Buffer.concat(out),
        });
      });
      child.stdin.on("error", () => {
        /* Process status remains authoritative. */
      });
      child.stdin.end(input);
    }
  );
const runCli = async (
  command: string,
  input: unknown,
  name?: string,
  raw = false
) => {
  const result = await processOutput(
    process.execPath,
    [cli, command, ...(name === undefined ? [] : [name])],
    raw ? String(input) : JSON.stringify(input)
  );
  equal(result.status, 0, result.stderr.toString());
  return JSON.parse(result.stdout.toString()) as unknown;
};
const inventory = async (
  directory: string,
  prefix = ""
): Promise<Record<string, string>> => {
  const entries = await readdir(directory, { withFileTypes: true });
  const records = await Promise.all(
    entries.map(async (entry) => {
      const relative = join(prefix, entry.name);
      const path = join(directory, entry.name);
      if (entry.isDirectory()) {
        return inventory(path, relative);
      }
      ok(entry.isFile());
      return { [relative]: sha(await readFile(path)) };
    })
  );
  return Object.assign({}, ...records) as Record<string, string>;
};
const state = (projectId: string) =>
  headless.call({ operation: "get_state", projectId }, projectStateSchema);
const transform = { opacity: 1, positionX: 0, positionY: 0, scale: 1 };
const at = (x: number, y: number) => ({
  anchor: { x: 0, y: 0 },
  opacity: 1,
  position: { unit: "pixels", x, y },
  rotationDeg: 0,
  scaleX: 1,
  scaleY: 1,
  skewXDeg: 0,
  skewYDeg: 0,
});
const rgba = (hex: string) => ({
  a: 1,
  b: Number.parseInt(hex.slice(5, 7), 16) / 255,
  g: Number.parseInt(hex.slice(3, 5), 16) / 255,
  r: Number.parseInt(hex.slice(1, 3), 16) / 255,
});
const create = (name: string) =>
  headless.call(
    {
      name,
      operation: "create_project",
      settings: { fps: 24, height: 180, width: 320 },
    },
    writeResultSchema
  );
const preview = async (projectId: string) => {
  const current = await state(projectId);
  const artifact = await headless.call(
    {
      expectedRevision: current.project.revision,
      operation: "render_preview",
      projectId,
      timeMs: 500,
    },
    artifactSchema
  );
  const path = join(store, projectId, artifact.relativePath);
  const decoded = await processOutput(
    environment.OPENCUT_FFMPEG_PATH ?? "ffmpeg",
    ["-v", "error", "-i", path, "-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"]
  );
  equal(decoded.status, 0, decoded.stderr.toString());
  equal(decoded.stdout.length, 320 * 180 * 3);
  return decoded.stdout;
};
const pixel = (frame: Buffer, x: number, y: number) => [
  ...frame.subarray((y * 320 + x) * 3, (y * 320 + x) * 3 + 3),
];
const literalReferences = new Map<string, number[]>();
const withinConversionBound = (actual: number[], expected: number[]) => {
  for (const [index, channel] of expected.entries()) {
    const observed = actual[index];
    ok(observed !== undefined);
    ok(
      Math.abs(observed - channel) <= 3,
      `PNG conversion exceeded 3: ${actual} vs ${expected}`
    );
  }
};
const assertPatch = (
  frame: Buffer,
  x: number,
  y: number,
  authored: number[]
) => {
  const reference = literalReferences.get(authored.join(","));
  ok(
    reference,
    "independent literal color reference must precede native rendering"
  );
  const actual = pixel(frame, x, y);
  withinConversionBound(actual, reference);
  withinConversionBound(actual, authored);
};
const assertFrame = (frame: Buffer, accent: number[]) => {
  assertPatch(frame, 12, 12, accent);
  assertPatch(frame, 250, 150, [23, 50, 77]);
  assertPatch(frame, 268, 28, [30, 180, 210]);
  let white = 0;
  for (let y = 60; y < 110; y += 1) {
    for (let x = 8; x < 250; x += 1) {
      const p = pixel(frame, x, y);
      if (p.every((channel) => channel > 220)) {
        white += 1;
      }
    }
  }
  ok(
    white > 30,
    "original caption glyphs must render in their independent region"
  );
};

try {
  // Generate these from the unchanged authored RGB inputs before creating any
  // project. Actual renderer output never supplies an expected color.
  await Promise.all(
    [
      [232, 180, 74],
      [23, 50, 77],
      [30, 180, 210],
      [160, 64, 192],
      [66, 165, 156],
    ].map(async (authoredRgb) => {
      const hex = authoredRgb
        .map((channel) => channel.toString(16).padStart(2, "0"))
        .join("");
      const reference = await processOutput(
        environment.OPENCUT_FFMPEG_PATH ?? "ffmpeg",
        [
          "-v",
          "error",
          "-f",
          "lavfi",
          "-i",
          `color=c=0x${hex}:s=320x180:r=24:d=1`,
          "-frames:v",
          "1",
          "-threads",
          "1",
          "-f",
          "rawvideo",
          "-pix_fmt",
          "rgb24",
          "pipe:1",
        ]
      );
      equal(reference.status, 0, reference.stderr.toString());
      equal(reference.stdout.length, 320 * 180 * 3);
      const converted = pixel(reference.stdout, 12, 12);
      withinConversionBound(converted, authoredRgb);
      literalReferences.set(authoredRgb.join(","), converted);
    })
  );
  const image = Buffer.from(
    contract.independentRenderOracle.image.pngBase64,
    "base64"
  );
  equal(sha(image), contract.independentRenderOracle.image.sha256);
  equal(image.length, contract.independentRenderOracle.image.sizeBytes);
  const imagePath = join(root, "original.png");
  await writeFile(imagePath, image);
  const source = await create("MontiPKM original source");
  const imported = await headless.call(
    {
      expectedRevision: 0,
      mediaType: "image",
      operation: "import_asset",
      path: imagePath,
      projectId: source.projectId,
    },
    writeResultSchema
  );
  const profile = await headless.call(
    {
      operation: "narration_profile_create",
      settings: narrationProfileSettingsSchema.parse(narrationFixture.settings),
    },
    narrationProfileSchema
  );
  const fields = componentFieldsSchema.parse({
    durationMs: 1000,
    height: 180,
    name: "Original configurable card",
    slots: [
      {
        binding: { property: "shape.fill.color", targetLayerId: "accent" },
        constraints: {},
        defaultValue: {
          type: "color",
          value: contract.independentRenderOracle.accent,
        },
        id: "accent",
        kind: "color",
        name: "Accent",
        required: false,
      },
      {
        binding: { property: "text.document", targetLayerId: "caption_text" },
        constraints: {},
        defaultValue: {
          type: "text",
          value: contract.independentRenderOracle.caption,
        },
        id: "caption",
        kind: "text",
        name: "Caption",
        required: false,
      },
    ],
    tracks: [
      {
        id: "card_track",
        items: [
          {
            color: contract.independentRenderOracle.background,
            durationMs: 1000,
            id: "background",
            keyframes: [],
            stackOrder: 0,
            startMs: 0,
            type: "solid_color",
          },
          {
            durationMs: 1000,
            fill: {
              color: rgba(contract.independentRenderOracle.accent),
              type: "solid",
            },
            geometry: { height: 40, type: "rectangle", width: 40 },
            id: "accent",
            keyframes: [],
            stackOrder: 1,
            startMs: 0,
            stroke: null,
            transform2d: at(0, 0),
            type: "shape",
          },
          {
            assetId: imported.changedIds[0],
            audio: { fadeInMs: 0, fadeOutMs: 0, muted: false, volume: 1 },
            durationMs: 1000,
            id: "artwork",
            keyframes: [],
            sourceInMs: 0,
            stackOrder: 2,
            startMs: 0,
            transform2d: at(260, 20),
            type: "media",
          },
          {
            color: "#FFFFFF",
            document: {
              runs: [{ text: contract.independentRenderOracle.caption }],
            },
            durationMs: 1000,
            fontSize: 20,
            id: "caption_text",
            keyframes: [],
            stackOrder: 3,
            startMs: 0,
            text: contract.independentRenderOracle.caption,
            transform2d: at(8, 60),
            type: "text",
          },
        ],
        name: "Original layers",
        trackType: "overlay",
      },
    ],
    width: 320,
  });
  const authored = await headless.call(
    {
      edit: { operation: "component_create", ...fields },
      expectedRevision: 1,
      operation: "edit",
      projectId: source.projectId,
    },
    writeResultSchema
  );
  const [recipeKey] = authored.changedIds;
  ok(recipeKey);
  const capture = {
    captionRole: { recipeKey, textSlotId: "caption" },
    componentIds: [recipeKey],
    expectedRevision: 2,
    name: "MontiPKM original",
    narrationProfile: { expectedVersion: 1, profileId: profile.id },
    projectId: source.projectId,
  };
  const sourceBefore = await inventory(join(store, source.projectId));
  const captured = channelCaptureResultSchema.parse(
    await runCli("capture", { capture })
  );
  deepStrictEqual(await inventory(join(store, source.projectId)), sourceBefore);
  equal(
    captured.pack.package.componentSchemaVersion,
    contract.componentSchemaVersion
  );
  for (const font of captured.pack.package.fonts) {
    ok(
      contract.independentRenderOracle.fonts.some(
        (expected) =>
          expected.sha256 === font.sha256 &&
          expected.sizeBytes === font.sizeBytes
      )
    );
  }
  const exported = await runCli("export", { packId: captured.pack.id });
  deepStrictEqual(exported, captured.pack.package);
  const copy = channelPackSchema.parse(
    await runCli(
      "import",
      JSON.stringify(exported),
      "Explicit independent copy",
      true
    )
  );
  ok(copy.id !== captured.pack.id);
  const beforeInvalid = await readFile(join(store, "channel-style-packs.json"));
  await Promise.all(
    [
      '{"formatVersion":1,"formatVersion":2}',
      JSON.stringify({ ...captured.pack.package, script: "forbidden" }),
    ].map(async (invalid) => {
      const rejected = await processOutput(
        process.execPath,
        [cli, "import", "Rejected"],
        invalid
      );
      equal(rejected.status, 1);
      equal(rejected.stdout.length, 0);
      deepStrictEqual(
        await readFile(join(store, "channel-style-packs.json")),
        beforeInvalid
      );
    })
  );
  const targets = await Promise.all(
    ["Independent A", "Independent B"].map(async (name) => {
      const target = await create(name);
      const targetImage = await headless.call(
        {
          expectedRevision: 0,
          mediaType: "image",
          operation: "import_asset",
          path: imagePath,
          projectId: target.projectId,
        },
        writeResultSchema
      );
      const current = await state(target.projectId);
      const trackId = current.project.tracks.find(
        (track) => track.trackType === "overlay"
      )?.id;
      ok(trackId);
      const application = {
        mode: "instantiate",
        placements: [
          {
            durationMs: 1000,
            hidden: false,
            parent: null,
            recipeKey,
            slotValues: {},
            staggerMs: 0,
            startMs: 0,
            timeScale: 1,
            trackId,
            transform,
            transform2d: null,
            trimStartMs: 0,
            zIndex: 0,
          },
        ],
      };
      // Exercise supported legacy target migration in the same native apply,
      // rather than opening it first (which would migrate separately).
      if (name === "Independent B") {
        const legacyPath = join(store, target.projectId, "project.json");
        const legacy = JSON.parse(await readFile(legacyPath, "utf8"));
        legacy.schemaVersion = 45;
        Reflect.deleteProperty(legacy, "narrationManifests");
        await writeFile(legacyPath, JSON.stringify(legacy, null, 2));
      }
      const request = {
        application,
        expectedRevision: 1,
        expectedVersion: 1,
        images: [
          {
            assetId: targetImage.changedIds[0],
            dependencyKey: captured.pack.package.images[0]?.key,
          },
        ],
        narration: { mode: "saved" },
        packId: captured.pack.id,
        projectId: target.projectId,
      };
      const missing = await processOutput(
        process.execPath,
        [cli, "apply"],
        JSON.stringify({ ...request, images: [] })
      );
      equal(missing.status, 1);
      equal(
        JSON.parse(missing.stderr.toString()).error.code,
        "DEPENDENCY_UNAVAILABLE"
      );
      const applied = channelApplyResultSchema.parse(
        await runCli("apply", request)
      );
      equal(applied.write.revision, 2);
      if (name === "Independent B") {
        const migrated = await state(target.projectId);
        equal(
          migrated.project.schemaVersion,
          narrationBlocks.projectSchemaVersion
        );
        const history = JSON.parse(
          await readFile(join(store, target.projectId, "history.json"), "utf8")
        );
        equal(
          history.undo.at(-1)?.schemaVersion,
          narrationBlocks.projectSchemaVersion
        );
      }
      equal(applied.narrationProfile?.id, profile.id);
      return { applied, id: target.projectId, request };
    })
  );
  const [a, b] = targets;
  ok(a && b);
  ok(
    a.applied.components[0]?.componentId !==
      b.applied.components[0]?.componentId
  );
  const firstA = await preview(a.id);
  const firstB = await preview(b.id);
  deepStrictEqual(firstA, firstB);
  assertFrame(firstA, [232, 180, 74]);
  const aState = await state(a.id);
  const instance = aState.project.tracks
    .flatMap((track) => track.items)
    .find((item) => item.type === "component_instance");
  ok(instance?.type === "component_instance");
  await headless.call(
    {
      edit: {
        componentId: instance.componentId,
        durationMs: 1000,
        itemId: instance.id,
        operation: "component_instance_update",
        slotValues: {
          accent: { type: "color", value: "#A040C0" },
          caption: { type: "text", value: "Personal A" },
        },
        startMs: 0,
        timeScale: 1,
        trimStartMs: 0,
      },
      expectedRevision: 2,
      operation: "edit",
      projectId: a.id,
    },
    writeResultSchema
  );
  const personalized = await preview(a.id);
  assertFrame(personalized, [160, 64, 192]);
  deepStrictEqual(await preview(b.id), firstB);
  const updatedSlots = fields.slots?.map((slot) => ({
    ...slot,
    defaultValue:
      slot.id === "accent"
        ? {
            type: "color" as const,
            value: contract.independentRenderOracle.reappliedAccent,
          }
        : {
            type: "text" as const,
            value: contract.independentRenderOracle.reappliedCaption,
          },
  }));
  ok(updatedSlots);
  await headless.call(
    {
      edit: {
        componentId: recipeKey,
        operation: "component_define_slots",
        slots: updatedSlots,
      },
      expectedRevision: 2,
      operation: "edit",
      projectId: source.projectId,
    },
    writeResultSchema
  );
  const updated = channelCaptureResultSchema.parse(
    await runCli("update", {
      capture: { ...capture, expectedRevision: 3 },
      expectedVersion: 1,
      packId: captured.pack.id,
    })
  );
  equal(updated.pack.version, 2);
  deepStrictEqual(await preview(b.id), firstB);
  await Promise.all(
    targets.map(async (target) => {
      const current = await state(target.id);
      const componentId = target.applied.components[0]?.componentId;
      ok(componentId);
      const request = {
        ...target.request,
        application: {
          components: [{ componentId, recipeKey }],
          mode: "reapply",
        },
        expectedRevision: current.project.revision,
        expectedVersion: 2,
      };
      const result = channelApplyResultSchema.parse(
        await runCli("apply", request)
      );
      ok(result.components[0]?.changed);
      const stale = await processOutput(
        process.execPath,
        [cli, "apply"],
        JSON.stringify(request)
      );
      equal(stale.status, 1);
      equal(
        JSON.parse(stale.stderr.toString()).error.code,
        "REVISION_CONFLICT"
      );
    })
  );
  deepStrictEqual(await preview(a.id), personalized);
  const changedB = await preview(b.id);
  assertFrame(changedB, [66, 165, 156]);
  await rename(
    join(store, "channel-style-packs.json"),
    join(store, "hidden-channel-library.json")
  );
  await headless.close();
  headless = new HeadlessClient(loadBridgeConfig(environment));
  deepStrictEqual(await preview(a.id), personalized);
  deepStrictEqual(await preview(b.id), changedB);
  const bCurrent = await state(b.id);
  await headless.call(
    {
      expectedRevision: bCurrent.project.revision,
      operation: "undo",
      projectId: b.id,
    },
    writeResultSchema
  );
  deepStrictEqual(await preview(b.id), firstB);
  const bUndo = await state(b.id);
  await headless.call(
    {
      expectedRevision: bUndo.project.revision,
      operation: "redo",
      projectId: b.id,
    },
    writeResultSchema
  );
  deepStrictEqual(await preview(b.id), changedB);
  console.log(
    JSON.stringify({
      componentSchemaVersion: captured.pack.package.componentSchemaVersion,
      independentLiteralReferences: Object.fromEntries(literalReferences),
      independentProjects: targets.length,
      initialFrameSha256: sha(firstA),
      legacyTargetMigratedInApply: true,
      libraryIndependentReopenUndoRedo: true,
      personalizedFrameSha256: sha(personalized),
      projectSchemaVersion: bCurrent.project.schemaVersion,
      reappliedFrameSha256: sha(changedB),
      result: "PASS",
      root,
      runtime: executable,
      sourceUnchangedAfterCapture: true,
    })
  );
} finally {
  await headless.close();
  if (cleanupConfirmed && process.env.OPENCUT_CHANNEL_KEEP_ROOT !== "1") {
    await rm(root, { force: true, recursive: true });
  }
}
