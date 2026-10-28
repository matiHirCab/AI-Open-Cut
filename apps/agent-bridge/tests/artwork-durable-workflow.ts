/** Actual native artwork/narration and durable rendering through source or packaged MCP. */
import { spawnSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { setTimeout } from "node:timers/promises";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { expect, vi } from "vitest";
import type { z } from "zod/v4";
import { artworkPreviewSchema } from "../src/artwork/contract";
import { loadBridgeConfig } from "../src/config";
import { HeadlessClient } from "../src/headless";
import { NarrationBlocksWorkflow } from "../src/narration-blocks/workflow";
import { durableOutputSchemas } from "../src/production-requests/contract";
import {
  jobSchema,
  narrationProfileSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";
import { SpeechApplicationService } from "../src/speech";
import { memoizedSdkValidator } from "./fixtures/memoized-sdk-validator";
import { createNarrationTestProvider } from "./fixtures/narration-blocks-provider";

type Parameters = ConstructorParameters<typeof StdioClientTransport>[0];
const hash = (value: Buffer) =>
  createHash("sha256").update(value).digest("hex");
const sourcePng = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAMAAAADCAYAAABWKLW/AAAAFElEQVR4nGP4jwQYQMQdETkEBwYAlZkh7kUp3TIAAAAASUVORK5CYII=",
  "base64"
);

export async function verifyArtworkDurableWorkflow(base: Parameters) {
  const root = mkdtempSync(join(tmpdir(), "opencut-artwork-durable-"));
  const projects = join(root, "projects"),
    work = join(root, "work"),
    exports = join(root, "exports"),
    narration = join(root, "narration");
  for (const path of [projects, work, exports, narration]) {
    mkdirSync(path);
  }
  const environment = {
    ...base.env,
    OPENCUT_ALLOWED_MEDIA_DIRS: root,
    OPENCUT_CONFIG_ROOT: root,
    OPENCUT_DURABLE_PRODUCTION_REQUESTS: "1",
    OPENCUT_EXPORTS_DIR: exports,
    OPENCUT_FFMPEG_PATH: "ffmpeg",
    OPENCUT_FFPROBE_PATH: "ffprobe",
    OPENCUT_PROJECTS_DIR: projects,
    OPENCUT_TTS_WORK_DIR: work,
  };
  const native = new HeadlessClient(loadBridgeConfig(environment));
  const fake = createNarrationTestProvider(work);
  const speech = new SpeechApplicationService(fake.provider, native, 60_000);
  const workflow = new NarrationBlocksWorkflow(native, speech);
  const context = () => ({
    markNonCancellable: vi.fn(),
    onProgress: vi.fn(),
    signal: new AbortController().signal,
  });
  const validator = memoizedSdkValidator();
  let client: Client | undefined;
  const connect = async () => {
    client = new Client(
      { name: "artwork-durable-acceptance", version: "1" },
      { jsonSchemaValidator: validator }
    );
    await client.connect(
      new StdioClientTransport({ ...base, env: environment })
    );
  };
  const call = async <T>(
    name: string,
    args: Record<string, unknown>,
    schema: z.ZodType<T>
  ) => {
    if (!client) {
      throw new Error("Fixture MCP client is not connected");
    }
    const result = await client.callTool({ arguments: args, name });
    expect(result.isError, JSON.stringify(result.structuredContent)).not.toBe(
      true
    );
    return schema.parse(result.structuredContent);
  };
  try {
    const created = await native.call(
      {
        name: "Artwork narration durable compatibility",
        operation: "create_project",
        settings: { fps: 10, height: 64, width: 64 },
      },
      writeResultSchema
    );
    const read = () =>
      native.call(
        { operation: "get_state", projectId: created.projectId },
        projectStateSchema
      );
    const initial = await read();
    const track = initial.project.tracks.find(
      (value) => value.trackType === "audio"
    );
    const overlay = initial.project.tracks.find(
      (value) => value.trackType === "overlay"
    );
    if (!(track && overlay)) {
      throw new Error("Native project default tracks are absent");
    }
    const profile = await native.call(
      {
        operation: "narration_profile_create",
        settings: {
          language: "en-US",
          name: "Original synthetic narration",
          speed: 1,
          textOptions: {
            chunking: "sentence",
            normalization: "basic",
            pronunciations: [],
            sentencePauseMs: 120,
          },
          voiceId: "contract_voice",
        },
      },
      narrationProfileSchema
    );
    const plan = await workflow.plan({
      boundaries: [2],
      expectedVersion: 1,
      maxTextCharacters: 4096,
      profileId: profile.id,
      sourceText: "A.",
    });
    const preview = await workflow.preview(plan, context());
    expect(preview.ready).toBe(true);
    const committed = await workflow.commit(
      {
        expectedRevision: created.revision,
        groupId: preview.groupId,
        projectId: created.projectId,
        startMs: 0,
        trackId: track.id,
      },
      context()
    );
    const original = join(root, "original.png");
    writeFileSync(original, sourcePng);
    const imported = await native.call(
      {
        expectedRevision: committed.write.revision,
        mediaType: "image",
        operation: "import_asset",
        path: original,
        projectId: created.projectId,
      },
      writeResultSchema
    );
    const [sourceId] = imported.changedIds;
    if (!sourceId) {
      throw new Error("Native artwork source asset is missing");
    }
    const prepared = await native.call(
      {
        expectedRevision: imported.revision,
        operation: "prepare_artwork",
        parameters: {
          cleanup: { color: [255, 255, 255], tolerance: 0 },
          outline: { color: [20, 30, 40, 255], radius: 1 },
        },
        projectId: created.projectId,
        sourceAssetId: sourceId,
      },
      artworkPreviewSchema
    );
    expect(prepared.manifest.sourceHash).toBe(hash(sourcePng));
    const artwork = await native.call(
      { manifest: prepared.manifest, operation: "commit_artwork" },
      writeResultSchema
    );
    const [assetId] = artwork.changedIds;
    if (!assetId) {
      throw new Error("Native artwork derivative is missing");
    }
    const placed = await native.call(
      {
        edit: {
          assetId,
          durationMs: 400,
          operation: "add_media",
          sourceInMs: 0,
          startMs: 0,
          trackId: overlay.id,
        },
        expectedRevision: artwork.revision,
        operation: "edit",
        projectId: created.projectId,
      },
      writeResultSchema
    );
    const before = await read();
    expect(before.project.schemaVersion).toBe(48);
    expect(before.project.narrationManifests).toEqual([committed.manifest]);
    expect(
      before.project.assets.find((value) => value.id === assetId)
        ?.artworkDerivation?.sourceAssetId
    ).toBe(sourceId);
    const projectDirectory = join(projects, created.projectId);
    const files = ["project.json", "history.json"].map((name) => ({
      hash: hash(readFileSync(join(projectDirectory, name))),
      name,
    }));
    const assets = before.project.assets.map((asset) => ({
      hash: hash(
        readFileSync(join(projectDirectory, asset.projectRelativePath))
      ),
      path: join(projectDirectory, asset.projectRelativePath),
    }));
    await connect();
    const requestId = randomUUID();
    const identity = { requestId, version: 1 };
    const intent = {
      kind: "preview_frame",
      request: {
        expectedRevision: placed.revision,
        projectId: created.projectId,
        timeMs: 0,
      },
    };
    const admission = await call(
      "production_request_admit",
      { ...identity, intent },
      durableOutputSchemas.admit
    );
    expect(admission.created).toBe(true);
    expect(admission.request.state).toBe("queued");
    const started = await call(
      "production_request_start",
      identity,
      durableOutputSchemas.start
    );
    expect(started.job).not.toBeNull();
    let terminal = (
      await call("production_request_get", identity, durableOutputSchemas.get)
    ).request;
    for (
      let attempt = 0;
      terminal?.state === "running" && attempt < 200;
      attempt += 1
    ) {
      // biome-ignore lint/performance/noAwaitInLoops: Observe terminal state sequentially without overlapping native jobs.
      await setTimeout(20);
      terminal = (
        await call("production_request_get", identity, durableOutputSchemas.get)
      ).request;
    }
    expect(terminal?.state).toBe("completed");
    expect(terminal?.outcome?.kind).toBe("rendered");
    const job = await call(
      "job_get_status",
      { jobId: started.job?.jobId },
      jobSchema
    );
    if (!job.artifact) {
      throw new Error("Native durable render did not publish an artifact");
    }
    const rendered = join(projectDirectory, job.artifact.relativePath);
    const decoded = spawnSync("ffmpeg", [
      "-v",
      "error",
      "-i",
      rendered,
      "-frames:v",
      "1",
      "-f",
      "rawvideo",
      "-pix_fmt",
      "rgba",
      "pipe:1",
    ]);
    expect(decoded.status, decoded.stderr.toString()).toBe(0);
    expect(decoded.stdout).toHaveLength(64 * 64 * 4);
    expect(readFileSync(rendered)).toHaveLength(job.artifact.sizeBytes);
    await client?.close();
    await connect();
    expect(
      (await call("production_request_get", identity, durableOutputSchemas.get))
        .request
    ).toEqual(terminal);
    const repeated = await call(
      "production_request_start",
      identity,
      durableOutputSchemas.start
    );
    expect(repeated.request).toEqual(terminal);
    expect(repeated.job).toBeNull();
    if (!client) {
      throw new Error("Fixture reconnect is absent");
    }
    const oldJob = await client.callTool({
      arguments: { jobId: started.job?.jobId },
      name: "job_get_status",
    });
    expect(oldJob.isError).toBe(true);
    const stale = await client.callTool({
      arguments: {
        intent: {
          ...intent,
          request: { ...intent.request, expectedRevision: placed.revision - 1 },
        },
        requestId: randomUUID(),
        version: 1,
      },
      name: "production_request_admit",
    });
    expect(stale.isError).toBe(true);
    expect(stale.structuredContent).toMatchObject({
      error: { code: "REVISION_CONFLICT" },
    });
    expect(await read()).toEqual(before);
    for (const file of files) {
      expect(hash(readFileSync(join(projectDirectory, file.name)))).toBe(
        file.hash
      );
    }
    for (const asset of assets) {
      expect(hash(readFileSync(asset.path))).toBe(asset.hash);
    }
    const undone = await native.call(
      {
        expectedRevision: placed.revision,
        operation: "undo",
        projectId: created.projectId,
      },
      writeResultSchema
    );
    await native.call(
      {
        expectedRevision: undone.revision,
        operation: "redo",
        projectId: created.projectId,
      },
      writeResultSchema
    );
    const reopened = await read();
    expect(reopened.project.assets).toEqual(before.project.assets);
    expect(reopened.project.narrationManifests).toEqual(
      before.project.narrationManifests
    );
    expect(hash(readFileSync(original))).toBe(hash(sourcePng));
    for (const asset of assets) {
      expect(hash(readFileSync(asset.path))).toBe(asset.hash);
    }
  } finally {
    await client?.close();
    validator.clear();
    await workflow.close();
    await speech.close();
    await native.close();
    rmSync(root, { force: true, recursive: true });
  }
}
