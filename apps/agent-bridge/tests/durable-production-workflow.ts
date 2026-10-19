/** Explicit real-runtime acceptance also reused by the existing hermetic smoke runners. */
import { deepStrictEqual, equal, ok } from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { setTimeout } from "node:timers/promises";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import type { z } from "zod/v4";
import { JOB_RUNTIME_MARKER } from "../src/job-runtime";
import {
  catalogSchema,
  DURABLE_CAPABILITY,
  DURABLE_FILES,
  durableOutputSchemas,
  type PublicRecord,
} from "../src/production-requests/contract";
import {
  jobSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";
import { memoizedSdkValidator } from "./fixtures/memoized-sdk-validator";

type Parameters = ConstructorParameters<typeof StdioClientTransport>[0];
const WHITESPACE = /\s+/;
const sha = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const quote = (value: string) => `'${value.replaceAll("'", "'\\''")}'`;
const call = async <T>(
  client: Client,
  name: string,
  args: Record<string, unknown>,
  schema: z.ZodType<T>
): Promise<T> => {
  const result = await client.callTool({ arguments: args, name });
  ok(!result.isError, JSON.stringify(result.structuredContent));
  return schema.parse(result.structuredContent);
};
const connect = async (parameters: Parameters) => {
  const validator = memoizedSdkValidator();
  const client = new Client(
    { name: "durable-production-acceptance", version: "1" },
    { jsonSchemaValidator: validator }
  );
  const transport = new StdioClientTransport(parameters);
  await client.connect(transport);
  return {
    client,
    close: async () => {
      await client.close();
      validator.clear();
    },
    transport,
  };
};
const wait = async (
  condition: () => boolean | Promise<boolean>,
  remaining = 2000
): Promise<void> => {
  if (await condition()) {
    return;
  }
  ok(remaining > 0, "Owned workflow did not reach its expected boundary");
  await setTimeout(10);
  return wait(condition, remaining - 1);
};
const record = (client: Client, requestId: string) =>
  call(
    client,
    "production_request_get",
    { requestId, version: 1 },
    durableOutputSchemas.get
  );
const terminal = async (
  client: Client,
  requestId: string
): Promise<PublicRecord> => {
  await wait(
    async () => (await record(client, requestId)).request?.state !== "running"
  );
  const result = (await record(client, requestId)).request;
  ok(result);
  return result;
};
const wave = () => {
  const frames = 4800;
  const bytes = Buffer.alloc(44 + frames * 2);
  bytes.write("RIFF");
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
  bytes.writeUInt32LE(frames * 2, 40);
  for (let index = 0; index < frames; index += 1) {
    bytes.writeInt16LE(
      Math.round(4000 * Math.sin((2 * Math.PI * 660 * index) / 48_000)),
      44 + index * 2
    );
  }
  return bytes;
};
const inventory = (directory: string, prefix = ""): Record<string, string> =>
  Object.assign(
    {},
    ...readdirSync(directory, { withFileTypes: true }).map((entry) => {
      const path = join(directory, entry.name),
        relative = join(prefix, entry.name);
      if (entry.isDirectory()) {
        return inventory(path, relative);
      }
      if (!(entry.name.endsWith(".json") || entry.name.endsWith(".wav"))) {
        return {};
      }
      ok(
        entry.isFile(),
        "Authoritative fixture data must remain ordinary files"
      );
      return { [relative]: sha(readFileSync(path)) };
    })
  );
const execute = (command: string, args: string[]) => {
  const result = spawnSync(command, args, { maxBuffer: 16_777_216 });
  equal(result.status, 0, result.stderr?.toString());
  return result.stdout;
};
const nativeOracle = (
  ffmpeg: string,
  ffprobe: string,
  path: string,
  format: "png" | "mp4",
  audio: boolean
) => {
  const pixels = execute(ffmpeg, [
    "-v",
    "error",
    "-i",
    path,
    "-frames:v",
    "1",
    "-vf",
    "format=rgb24",
    "-f",
    "rawvideo",
    "pipe:1",
  ]);
  equal(pixels.length, 160 * 90 * 3);
  // Independent literal-color reference includes backend quantization. The
  // existing native pipeline includes additional YUV conversions even for PNG.
  const reference = execute(ffmpeg, [
    "-v",
    "error",
    "-f",
    "lavfi",
    "-i",
    "color=c=0x123456:s=160x90:r=10",
    "-frames:v",
    "1",
    "-pix_fmt",
    "rgb24",
    "-f",
    "rawvideo",
    "pipe:1",
  ]);
  equal(reference.length, pixels.length);
  for (let index = 0; index < pixels.length; index += 1) {
    ok(
      Math.abs((pixels[index] ?? -100) - (reference[index] ?? 100)) <=
        (format === "png" ? 3 : 5),
      "Original full-frame literal-color reference must match"
    );
    ok(
      Math.abs((pixels[index] ?? -100) - ([18, 52, 86][index % 3] ?? 100)) <=
        (format === "png" ? 3 : 5),
      "Original full-frame RGB source must match within conversion quantization"
    );
  }
  if (format === "png") {
    return;
  }
  const parsed = JSON.parse(
    execute(ffprobe, [
      "-v",
      "error",
      "-show_streams",
      "-of",
      "json",
      path,
    ]).toString()
  ) as { streams: { codec_type: string; width?: number; height?: number }[] };
  equal(
    parsed.streams.some((stream) => stream.codec_type === "audio"),
    audio
  );
  ok(
    parsed.streams.some(
      (stream) =>
        stream.codec_type === "video" &&
        stream.width === 160 &&
        stream.height === 90
    )
  );
  if (audio) {
    const pcm = execute(ffmpeg, [
      "-v",
      "error",
      "-i",
      path,
      "-vn",
      "-ac",
      "1",
      "-ar",
      "48000",
      "-f",
      "f32le",
      "pipe:1",
    ]);
    let energy = 0,
      crossings = 0;
    const first = 512,
      last = Math.min(4500, pcm.length / 4);
    ok(last > first + 3000);
    for (let index = first; index < last; index += 1) {
      const sample = pcm.readFloatLE(index * 4);
      energy += sample * sample;
      if (
        index > first &&
        pcm.readFloatLE((index - 1) * 4) <= 0 &&
        sample > 0
      ) {
        crossings += 1;
      }
    }
    const rms = Math.sqrt(energy / (last - first));
    ok(rms > 0.04 && rms < 0.13, `Original tone RMS ${rms}`);
    ok(
      Math.abs((crossings * 48_000) / (last - first) - 660) < 25,
      "Original 660Hz tone must survive native output"
    );
  }
};

const runningCrash = async (options: {
  parameters: Parameters;
  work: string;
  common: { projectId: string; expectedRevision: number };
  gate: string;
  entered: string;
  invocations: string;
}) => {
  ok(
    process.platform !== "win32",
    "Explicit subprocess-crash acceptance requires POSIX; Windows runtime verification remains separate"
  );
  const { parameters, work, common, gate, entered, invocations } = options;
  const isolated: Parameters = {
    ...parameters,
    env: { ...parameters.env, OPENCUT_TTS_WORK_DIR: work },
  };
  mkdirSync(work);
  let owner = await connect(isolated);
  const stoppedGroups = new Set<number>();
  const failures: unknown[] = [];
  try {
    equal(
      (
        await call(
          owner.client,
          "production_request_list",
          { version: 1 },
          durableOutputSchemas.list
        )
      ).requests.length,
      0
    );
    const requestId = randomUUID();
    const identity = { requestId, version: 1 };
    await call(
      owner.client,
      "production_request_admit",
      {
        ...identity,
        intent: {
          kind: "export_video",
          request: {
            ...common,
            format: "mp4",
            overwrite: false,
            relativePath: "original-interrupted-export.mp4",
            resolution: "project",
          },
        },
      },
      durableOutputSchemas.admit
    );
    rmSync(entered, { force: true });
    writeFileSync(gate, "hold");
    const started = await call(
      owner.client,
      "production_request_start",
      identity,
      durableOutputSchemas.start
    );
    ok(started.job);
    await wait(() => existsSync(entered));
    const catalogPath = join(
      work,
      DURABLE_FILES.directory,
      DURABLE_FILES.catalog
    );
    const image = readFileSync(catalogPath);
    equal(
      catalogSchema.parse(JSON.parse(image.toString())).records[0]?.state,
      "running"
    );
    const { pid } = owner.transport;
    ok(pid);
    const output =
      process.platform === "darwin"
        ? execute("pgrep", ["-P", String(pid)])
        : execute("ps", ["-o", "pid=", "--ppid", String(pid)]);
    const children = output.toString().trim().split(WHITESPACE).map(Number);
    ok(
      children.length > 0 &&
        children.every((value) => Number.isSafeInteger(value) && value > 1),
      "Observe the controlled live native producer before owner death"
    );
    // Headless producers use their own process group. Freeze the known groups,
    // kill the owner, then terminate those same groups before reconciliation.
    for (const child of children) {
      stoppedGroups.add(child);
      process.kill(-child, "SIGSTOP");
    }
    const closed = new Promise<void>((resolveClosed) => {
      owner.client.onclose = resolveClosed;
    });
    process.kill(pid, "SIGKILL");
    for (const child of children) {
      process.kill(-child, "SIGKILL");
      stoppedGroups.delete(child);
    }
    await closed;
    await owner.close();
    rmSync(gate);
    const encoded = readFileSync(invocations, "utf8");
    owner = await connect(isolated);
    const current = (await record(owner.client, requestId)).request;
    equal(current?.state, "uncertain");
    equal(current.uncertainty, "previous_runtime");
    equal(current.outcome, null);
    const repeated = await call(
      owner.client,
      "production_request_start",
      identity,
      durableOutputSchemas.start
    );
    equal(repeated.request.state, "uncertain");
    equal(repeated.job, null);
    const missing = await owner.client.callTool({
      arguments: { jobId: started.job.jobId },
      name: "job_get_status",
    });
    equal(missing.isError, true);
    equal(readFileSync(invocations, "utf8"), encoded);
    equal(sha(readFileSync(catalogPath)), sha(image));
  } catch (error) {
    failures.push(error);
  } finally {
    for (const child of stoppedGroups) {
      try {
        process.kill(-child, "SIGKILL");
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code !== "ESRCH") {
          failures.push(error);
        }
      }
    }
    try {
      rmSync(gate, { force: true });
      await owner.close();
    } catch (error) {
      failures.push(error);
    }
  }
  if (failures.length > 0) {
    throw new AggregateError(
      failures,
      "Owned crash workflow or cleanup failed"
    );
  }
};

export const verifyDurableProductionWorkflow = async (
  base: Parameters,
  native = false
) => {
  const root = mkdtempSync(join(tmpdir(), "opencut-durable-mcp-"));
  const projects = join(root, "projects"),
    exports = join(root, "exports"),
    work = join(root, "work");
  for (const path of [projects, exports, work]) {
    mkdirSync(path);
  }
  const ffmpeg = native
    ? (process.env.OPENCUT_FFMPEG_PATH ?? "ffmpeg")
    : base.env?.OPENCUT_FFMPEG_PATH;
  const ffprobe = native
    ? (process.env.OPENCUT_FFPROBE_PATH ?? "ffprobe")
    : base.env?.OPENCUT_FFPROBE_PATH;
  ok(
    ffmpeg && ffprobe,
    "Explicit backend paths are required; this workflow never readiness-skips"
  );
  const proxy = join(root, "owned-ffmpeg"),
    gate = join(root, "gate"),
    entered = join(root, "entered"),
    invocations = join(root, "invocations");
  writeFileSync(
    proxy,
    `#!/bin/sh\nexec bun ${quote(resolve(import.meta.dirname, "fixtures/durable-render-gate.ts"))} "$@"\n`
  );
  chmodSync(proxy, 0o755);
  const parameters: Parameters = {
    ...base,
    env: {
      ...base.env,
      OPENCUT_ALLOWED_MEDIA_DIRS: root,
      OPENCUT_CONFIG_ROOT: root,
      OPENCUT_DURABLE_PRODUCTION_REQUESTS: "1",
      OPENCUT_DURABLE_TEST_BACKEND: ffmpeg,
      OPENCUT_DURABLE_TEST_ENTERED: entered,
      OPENCUT_DURABLE_TEST_GATE: gate,
      OPENCUT_DURABLE_TEST_INVOCATIONS: invocations,
      OPENCUT_EXPORTS_DIR: exports,
      OPENCUT_FFMPEG_PATH: proxy,
      OPENCUT_FFPROBE_PATH: ffprobe,
      OPENCUT_LOG_LEVEL: "error",
      OPENCUT_PROJECTS_DIR: projects,
      OPENCUT_TTS_WORK_DIR: work,
    },
  };
  let owner = await connect(parameters);
  const directory = join(work, DURABLE_FILES.directory),
    catalogPath = join(directory, DURABLE_FILES.catalog);
  const outcomes: { kind: string; bytes: number; sha256: string }[] = [];
  try {
    const status = await call(
      owner.client,
      "editor_get_status",
      {},
      statusSchema
    );
    ok(status.capabilities.includes(DURABLE_CAPABILITY));
    equal(existsSync(directory), false);
    equal(
      (
        await call(
          owner.client,
          "production_request_list",
          { version: 1 },
          durableOutputSchemas.list
        )
      ).requests.length,
      0
    );
    equal(existsSync(directory), false);
    const created = await call(
      owner.client,
      "project_create",
      {
        fps: 10,
        height: 90,
        name: "Original bounded durable production fixture",
        width: 160,
      },
      writeResultSchema
    );
    const initial = await call(
      owner.client,
      "project_get_state",
      { projectId: created.projectId },
      projectStateSchema
    );
    const overlay = initial.project.tracks.find(
        (track) => track.trackType === "overlay"
      ),
      audio = initial.project.tracks.find(
        (track) => track.trackType === "audio"
      );
    ok(overlay && audio);
    const solid = await call(
      owner.client,
      "timeline_add_solid_color",
      {
        color: "#123456",
        durationMs: 100,
        expectedRevision: created.revision,
        projectId: created.projectId,
        startMs: 0,
        trackId: overlay.id,
        transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
      },
      writeResultSchema
    );
    const source = join(root, "original-tone.wav");
    writeFileSync(source, wave());
    const sourceSha256 = sha(readFileSync(source));
    const imported = await call(
      owner.client,
      "asset_import",
      {
        expectedRevision: solid.revision,
        mediaType: "audio",
        path: source,
        projectId: created.projectId,
      },
      writeResultSchema
    );
    const withAsset = await call(
      owner.client,
      "project_get_state",
      { projectId: created.projectId },
      projectStateSchema
    );
    const [asset] = withAsset.project.assets;
    ok(asset);
    const edited = await call(
      owner.client,
      "timeline_batch_edit",
      {
        expectedRevision: imported.revision,
        operations: [
          {
            assetId: asset.id,
            durationMs: 100,
            operation: "add_media",
            sourceInMs: 0,
            startMs: 0,
            trackId: audio.id,
          },
        ],
        projectId: created.projectId,
      },
      writeResultSchema
    );
    const before = inventory(join(projects, created.projectId));
    const common = {
      expectedRevision: edited.revision,
      projectId: created.projectId,
    };
    const intents = [
      { kind: "preview_frame", request: { ...common, timeMs: 0 } },
      {
        kind: "preview_range",
        request: {
          ...common,
          endMs: 100,
          fps: 10,
          resolution: { height: 90, width: 160 },
          startMs: 0,
        },
      },
      {
        kind: "review_range",
        request: {
          ...common,
          endMs: 100,
          fps: 10,
          resolution: "project",
          startMs: 0,
        },
      },
      {
        kind: "export_video",
        request: {
          ...common,
          format: "mp4",
          overwrite: false,
          relativePath: "original-export.mp4",
          resolution: "project",
        },
      },
    ];
    const ids = intents.map(() => randomUUID());
    const produce = async (index: number): Promise<void> => {
      const intent = intents[index],
        requestId = ids[index];
      ok(intent && requestId);
      const identity = { requestId, version: 1 };
      const admitted = await call(
        owner.client,
        "production_request_admit",
        { ...identity, intent },
        durableOutputSchemas.admit
      );
      equal(admitted.request.state, "queued");
      equal(admitted.created, true);
      equal(
        (
          await call(
            owner.client,
            "production_request_admit",
            { ...identity, intent },
            durableOutputSchemas.admit
          )
        ).created,
        false
      );
      const started = await call(
        owner.client,
        "production_request_start",
        identity,
        durableOutputSchemas.start
      );
      ok(started.job);
      const result = await terminal(owner.client, requestId);
      equal(result.state, "completed");
      ok(result.outcome?.kind === "rendered");
      const job = await call(
        owner.client,
        "job_get_status",
        { jobId: started.job.jobId },
        jobSchema
      );
      ok(job.artifact);
      const path = join(
        intent.kind === "export_video"
          ? exports
          : join(projects, created.projectId),
        job.artifact.relativePath
      );
      equal(statSync(path).size, result.outcome.sizeBytes);
      if (native) {
        nativeOracle(
          ffmpeg,
          ffprobe,
          path,
          result.outcome.format,
          intent.kind === "review_range" || intent.kind === "export_video"
        );
      }
      outcomes.push({
        bytes: result.outcome.sizeBytes,
        kind: intent.kind,
        sha256: sha(readFileSync(path)),
      });
    };
    await produce(0);
    await produce(1);
    await produce(2);
    await produce(3);
    const queuedId = randomUUID();
    await call(
      owner.client,
      "production_request_admit",
      { intent: intents[0], requestId: queuedId, version: 1 },
      durableOutputSchemas.admit
    );
    // Actual published export while terminal metadata is blocked by an owned test stage.
    writeFileSync(gate, "hold");
    const faultId = randomUUID();
    const faultIntent = {
      kind: "export_video",
      request: {
        ...common,
        format: "mp4",
        overwrite: false,
        relativePath: "original-terminal-fault.mp4",
        resolution: "project",
      },
    };
    await call(
      owner.client,
      "production_request_admit",
      { intent: faultIntent, requestId: faultId, version: 1 },
      durableOutputSchemas.admit
    );
    const faultJob = await call(
      owner.client,
      "production_request_start",
      { requestId: faultId, version: 1 },
      durableOutputSchemas.start
    );
    ok(faultJob.job);
    await wait(() => existsSync(entered));
    const stage = join(directory, DURABLE_FILES.stage);
    writeFileSync(stage, "controlled foreign stage");
    rmSync(gate);
    await wait(
      async () =>
        (
          await call(
            owner.client,
            "job_get_status",
            { jobId: faultJob.job?.jobId },
            jobSchema
          )
        ).status === "completed"
    );
    const uncertain = (await record(owner.client, faultId)).request;
    equal(uncertain?.state, "uncertain");
    equal(uncertain.uncertainty, "metadata_unconfirmed");
    equal(uncertain.outcome, null);
    const published = join(exports, "original-terminal-fault.mp4");
    ok(statSync(published).size > 0);
    if (native) {
      nativeOracle(ffmpeg, ffprobe, published, "mp4", true);
    }
    const publishedExports = [
      "original-export.mp4",
      "original-terminal-fault.mp4",
    ].map((name) => ({ name, sha256: sha(readFileSync(join(exports, name))) }));
    equal(readFileSync(stage, "utf8"), "controlled foreign stage");
    const encodedBeforeClose = readFileSync(invocations, "utf8");
    rmSync(stage); // Explicitly remove only the stage created by this fixture.
    await owner.close();
    await wait(() => !existsSync(join(work, JOB_RUNTIME_MARKER)));
    equal(readFileSync(invocations, "utf8"), encodedBeforeClose);
    const closedCatalog = catalogSchema.parse(
      JSON.parse(readFileSync(catalogPath, "utf8"))
    );
    equal(
      closedCatalog.records.find((row) => row.requestId === faultId)?.state,
      "completed"
    );
    deepStrictEqual(inventory(join(projects, created.projectId)), before);
    owner = await connect(parameters);
    equal((await record(owner.client, queuedId)).request?.state, "queued");
    const retainedTerminal = await call(
      owner.client,
      "production_request_start",
      { requestId: ids[3], version: 1 },
      durableOutputSchemas.start
    );
    equal(retainedTerminal.request.state, "completed");
    equal(retainedTerminal.job, null);
    // Fresh queued admission acquires guards, then actual owner death leaves them intact.
    const crashId = randomUUID();
    await call(
      owner.client,
      "production_request_admit",
      { intent: intents[0], requestId: crashId, version: 1 },
      durableOutputSchemas.admit
    );
    const markers = [
      join(work, JOB_RUNTIME_MARKER),
      join(directory, JOB_RUNTIME_MARKER),
    ].map((path) => ({ nonce: readFileSync(path, "utf8"), path }));
    const persisted = readFileSync(catalogPath);
    const crash = async () => {
      const { pid } = owner.transport;
      ok(pid);
      const closed = new Promise<void>((resolveClosed) => {
        owner.client.onclose = resolveClosed;
      });
      process.kill(pid, "SIGKILL");
      await closed;
      await owner.close();
    };
    await crash();
    owner = await connect(parameters);
    const cold = await call(
      owner.client,
      "production_request_list",
      { version: 1 },
      durableOutputSchemas.list
    );
    equal(cold.storage.reason, "guard_occupied");
    equal(cold.storage.writable, false);
    equal(
      cold.requests.find((row) => row.requestId === crashId)?.state,
      "queued"
    );
    const blocked = await owner.client.callTool({
      arguments: { requestId: crashId, version: 1 },
      name: "production_request_start",
    });
    equal(blocked.isError, true);
    const oldHandle = await owner.client.callTool({
      arguments: { jobId: faultJob.job.jobId },
      name: "job_get_status",
    });
    equal(oldHandle.isError, true);
    await crash();
    owner = await connect(parameters);
    equal((await record(owner.client, crashId)).request?.state, "queued");
    equal(sha(readFileSync(catalogPath)), sha(persisted));
    for (const marker of markers) {
      equal(readFileSync(marker.path, "utf8"), marker.nonce);
    }
    deepStrictEqual(
      readdirSync(directory).sort(),
      [DURABLE_FILES.catalog, JOB_RUNTIME_MARKER].sort()
    );
    deepStrictEqual(inventory(join(projects, created.projectId)), before);
    await runningCrash({
      common,
      entered,
      gate,
      invocations,
      parameters,
      work: join(root, "running-work"),
    });
    deepStrictEqual(inventory(join(projects, created.projectId)), before);
    equal(sha(readFileSync(source)), sourceSha256);
    for (const output of publishedExports) {
      equal(sha(readFileSync(join(exports, output.name))), output.sha256);
    }
    const projectDirectory = join(projects, created.projectId),
      backup = join(root, "temporarily-unavailable-project");
    renameSync(projectDirectory, backup);
    try {
      equal(existsSync(projectDirectory), false);
      const repeated = await call(
        owner.client,
        "production_request_admit",
        { intent: intents[3], requestId: ids[3], version: 1 },
        durableOutputSchemas.admit
      );
      equal(repeated.created, false);
      equal(repeated.request.state, "completed");
      equal(
        (
          await call(
            owner.client,
            "production_request_start",
            { requestId: ids[3], version: 1 },
            durableOutputSchemas.start
          )
        ).request.state,
        "completed"
      );
    } finally {
      renameSync(backup, projectDirectory);
    }
    return {
      authoritativeFiles: Object.keys(before).length,
      native,
      nativeSha256: sha(
        readFileSync(parameters.env?.OPENCUT_HEADLESS_PATH ?? "")
      ),
      oldHandlesAdopted: false,
      outputs: outcomes,
      publishedExports,
      queuedRetained: true,
      repeatedCrashes: 2,
      result: "PASS",
      runningCrash:
        "actual native entry / uncertain after owner death / zero replay",
      sourceSha256,
      terminalFault:
        "published export / unconfirmed metadata / metadata-only close",
    };
  } finally {
    rmSync(gate, { force: true });
    await owner.close();
    rmSync(root, { force: true, recursive: true });
  }
};

if (import.meta.main) {
  const binary = process.env.OPENCUT_HEADLESS_PATH;
  ok(binary, "Provide the actual native executable; no readiness skip");
  const environment = Object.fromEntries(
    Object.entries(process.env).filter(
      (entry): entry is [string, string] => entry[1] !== undefined
    )
  );
  const parameters: Parameters = {
    args: process.env.OPENCUT_TEST_BRIDGE_PATH
      ? []
      : [resolve(import.meta.dirname, "../src/index.ts")],
    command: process.env.OPENCUT_TEST_BRIDGE_PATH ?? process.execPath,
    env: {
      ...environment,
      OPENCUT_HEADLESS_PATH: binary,
      OPENCUT_KOKORO_PYTHON: "python3",
      OPENCUT_KOKORO_WORKER: resolve(
        import.meta.dirname,
        "fixtures/fake_tts_worker.py"
      ),
      OPENCUT_TRANSCRIPTION_PYTHON: "python3",
      OPENCUT_TRANSCRIPTION_WORKER: resolve(
        import.meta.dirname,
        "fixtures/fake_transcription_worker.py"
      ),
    },
    stderr: "inherit",
  };
  process.stdout.write(
    `${JSON.stringify(await verifyDurableProductionWorkflow(parameters, true))}\n`
  );
}
