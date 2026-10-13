import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { expect, it } from "vitest";
import CATALOG from "../../../contracts/ffmpeg-spawn-diagnostics-v1.json";
import { loadBridgeConfig } from "../src/config";
import { ffmpegSpawnSchema } from "../src/ffmpeg-spawn";
import { errorBody, HeadlessClient } from "../src/headless";
import { BridgeError, eventSchema } from "../src/headless-events";
import { JobRegistry } from "../src/jobs";
import { artifactSchema, publicErrorSchema } from "../src/schemas";
import { restoreFfmpegSpawnGraphicRaw } from "./fixtures/ffmpeg-spawn-graphic-projection";
import pins from "./fixtures/ffmpeg-spawn-predecessor-pins.json";
import {
  removeFfmpegSpawnMcpAddition,
  restoreFfmpegSpawnRaw,
} from "./fixtures/ffmpeg-spawn-projection";
import { restoreSolidGraphicRaw } from "./fixtures/solid-graphic-color-slots-projection";

it("retains canonical startup detail across overlapping worker/one-shot transport and failed jobs", async () => {
  const root = mkdtempSync(join(tmpdir(), "opencut-spawn-ipc-"));
  const marker = join(root, "marker");
  const config = loadBridgeConfig({
    ...process.env,
    OPENCUT_EXPORTS_DIR: join(root, "exports"),
    OPENCUT_PROJECTS_DIR: join(root, "projects"),
    OPENCUT_TEST_FFMPEG_SPAWN_MARKER: marker,
  });
  const client = new HeadlessClient({
    ...config,
    headlessArguments: [
      resolve(import.meta.dirname, "fixtures/ffmpeg-spawn-headless.ts"),
    ],
    headlessPath: process.execPath,
  });
  const jobs = new JobRegistry({ headless: client });
  const request = {
    expectedRevision: 0,
    operation: "render_preview" as const,
    projectId: "transport-fixture",
    timeMs: 0,
  };
  try {
    const results = await Promise.allSettled([
      client.call(request, artifactSchema),
      client.call(request, artifactSchema),
    ]);
    for (const result of results) {
      expect(result.status).toBe("rejected");
      if (result.status === "rejected") {
        expect(errorBody(result.reason)).toEqual(
          CATALOG.headlessErrorExample.error
        );
      }
    }
    expect(readFileSync(marker, "utf8").trim().split("\n").sort()).toEqual([
      "one-shot",
      "worker",
    ]);
    const queued = jobs.start("preview", "transport-fixture", 0, request);
    const waitForTerminal = async (
      remaining: number
    ): Promise<ReturnType<typeof jobs.get>> => {
      const current = jobs.get(queued.jobId);
      if (current.status === "failed" || current.status === "completed") {
        return current;
      }
      if (remaining === 0) {
        throw new Error("Owned IPC fixture job did not finish");
      }
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 10));
      return waitForTerminal(remaining - 1);
    };
    const failed = await waitForTerminal(100);
    expect(failed.status).toBe("failed");
    expect(failed.error).toEqual(CATALOG.headlessErrorExample.error);
    expect(failed.artifact).toBeUndefined();
    expect(failed.progress).toBe(0);
    expect(readFileSync(marker, "utf8").trim().split("\n").sort()).toEqual([
      "one-shot",
      "worker",
      "worker",
    ]);
  } finally {
    await jobs.close();
    await client.close();
    rmSync(root, { force: true, recursive: true });
  }
});

it("preserves canonical startup metadata through headless and public bridge errors", () => {
  for (const fixture of CATALOG.fixtures) {
    expect(
      ffmpegSpawnSchema.safeParse(fixture.value).success,
      fixture.name
    ).toBe(fixture.valid);
    if (!fixture.valid) {
      continue;
    }
    const value = {
      ...CATALOG.headlessErrorExample.error,
      ffmpegSpawn: ffmpegSpawnSchema.parse(fixture.value),
    };
    expect(eventSchema.parse({ error: value, type: "error" })).toEqual({
      error: value,
      type: "error",
    });
    const error = new BridgeError(
      value.code,
      value.message,
      value.retryable,
      undefined,
      value
    );
    expect(publicErrorSchema.parse(errorBody(error))).toEqual(value);
    expect(errorBody(error).ffmpegExitCode).toBeNull();
  }
  const ordinary = errorBody(
    new BridgeError("REVISION_CONFLICT", "stale revision", true)
  );
  expect(Object.hasOwn(ordinary, "ffmpegSpawn")).toBe(false);
  expect(ordinary.retryable).toBe(true);
});

it("retains exact predecessors and rejects malformed additions and unrelated drift", () => {
  for (const [name, hash] of Object.entries(pins.rawSha256)) {
    const raw = readFileSync(
      new URL(`../../../contracts/${name}.json`, import.meta.url),
      "utf8"
    );
    expect(
      createHash("sha256").update(restoreFfmpegSpawnRaw(raw)).digest("hex")
    ).toBe(hash);
  }
  const raw = readFileSync(
    new URL("../../../contracts/mcp-surface-v1.json", import.meta.url),
    "utf8"
  );
  const malformed = JSON.parse(raw);
  malformed.$defs.FfmpegSpawnDiagnostic.properties.nativeCode.anyOf[0].maximum += 1;
  expect(() => removeFfmpegSpawnMcpAddition(malformed)).toThrow("Unapproved");
  const drift = JSON.parse(raw);
  drift.version = 99;
  expect(removeFfmpegSpawnMcpAddition(drift).version).toBe(99);
  expect(() =>
    restoreFfmpegSpawnRaw(`${JSON.stringify(drift, null, 2)}\n`)
  ).toThrow("Unrelated");
  const missing = JSON.parse(raw);
  const schema =
    missing.$defs.DraftPreviewFrameOutputProperties.error.properties;
  Reflect.deleteProperty(schema, "ffmpegSpawn");
  expect(() => removeFfmpegSpawnMcpAddition(missing)).toThrow("Missing");
  for (const name of ["editor", "rendering", "speech", "transcription"]) {
    Reflect.deleteProperty(
      missing.toolDefinitions.editor_get_status.outputSchema.properties
        .subsystems.properties[name].properties.error.anyOf[0].properties,
      "ffmpegSpawn"
    );
  }
  Reflect.deleteProperty(
    missing.toolDefinitions.tts_get_status.outputSchema.properties.startupError
      .anyOf[0].properties,
    "ffmpegSpawn"
  );
  expect(() => removeFfmpegSpawnMcpAddition(missing)).toThrow("Missing");
  for (const restore of [
    restoreFfmpegSpawnRaw,
    restoreFfmpegSpawnGraphicRaw,
    restoreSolidGraphicRaw,
  ]) {
    expect(() => restore(JSON.stringify(missing))).toThrow(
      "Missing FFmpeg spawn schema addition"
    );
  }
});

it("preserves both merged owner proofs and historical identity without concealing drift", () => {
  for (const [name, hash] of Object.entries(pins.rawSha256)) {
    const source = readFileSync(
      new URL(`../../../contracts/${name}.json`, import.meta.url),
      "utf8"
    );
    for (const restore of [
      restoreFfmpegSpawnRaw,
      restoreFfmpegSpawnGraphicRaw,
      restoreSolidGraphicRaw,
    ]) {
      const historical = restore(source);
      expect(createHash("sha256").update(historical).digest("hex")).toBe(hash);
      expect(restore(historical)).toBe(historical);
      const drift = { ...JSON.parse(source), unauthorized: "unrelated drift" };
      expect(() => restore(JSON.stringify(drift))).toThrow(
        "predecessor contract drift"
      );
    }
  }
});
