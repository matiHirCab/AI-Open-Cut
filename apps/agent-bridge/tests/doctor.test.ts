import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { afterEach, expect, it } from "vitest";

import { loadBridgeConfig } from "../src/config";
import { type DoctorPlatform, runDoctor } from "../src/doctor";
import type { HeadlessClient } from "../src/headless";
import type { SpeechApplicationService } from "../src/speech";

let temporary = "";

afterEach(async () => {
  if (temporary) {
    await rm(temporary, { force: true, recursive: true });
  }
});

const headless = (renderingReady = true) =>
  ({
    call: () =>
      Promise.resolve({
        capabilities: renderingReady
          ? ["projects", "timeline", "preview", "export"]
          : ["projects", "timeline"],
        ready: true,
        subsystems: {
          editor: {
            capabilities: ["projects", "timeline"],
            error: null,
            ready: true,
          },
          rendering: {
            capabilities: renderingReady ? ["preview", "export"] : [],
            error: renderingReady
              ? null
              : {
                  code: "DEPENDENCY_UNAVAILABLE",
                  message: "rendering unavailable",
                  retryable: false,
                },
            ready: renderingReady,
          },
        },
        version: "0.1.0",
      }),
  }) as unknown as HeadlessClient;

const speech = (ready = true, discardFails = false) =>
  ({
    discardPreview: () =>
      discardFails
        ? Promise.reject(new Error("cleanup failed"))
        : Promise.resolve({ discarded: true }),
    preview: () => Promise.resolve({ token: "doctor-token" }),
    status: () =>
      Promise.resolve({
        defaultLanguage: "en-US",
        defaultVoiceId: "af_heart",
        modelCached: ready,
        modelLoaded: ready,
        queue: {
          active: 0,
          concurrency: 1,
          fairness: "fifo",
          maxQueued: 8,
          queued: 0,
        },
        ready,
      }),
  }) as unknown as SpeechApplicationService;

const platform = (overrides: Partial<DoctorPlatform> = {}): DoctorPlatform => ({
  capture: () =>
    Promise.resolve({ code: 0, stderr: "", stdout: "Python 3.11" }),
  freeBytes: () => Promise.resolve(10 * 1024 * 1024 * 1024),
  writable: () => Promise.resolve(),
  ...overrides,
});

it("reports a successful fake-provider diagnosis", async () => {
  temporary = await mkdtemp(join(tmpdir(), "opencut-doctor-test-"));
  const config = loadBridgeConfig({
    OPENCUT_CONFIG_ROOT: temporary,
    OPENCUT_KOKORO_PYTHON: process.execPath,
  });
  const report = await runDoctor(config, headless(), speech(), platform());
  expect(report.ready).toBe(true);
  expect(report.checks).toEqual(
    expect.arrayContaining([
      expect.objectContaining({ id: "python", status: "ok" }),
      expect.objectContaining({ id: "writeability", status: "ok" }),
      expect.objectContaining({ id: "rendering", status: "ok" }),
      expect.objectContaining({ id: "speech", status: "ok" }),
      expect.objectContaining({ id: "synthesis", status: "ok" }),
    ])
  );
});

it("fails doctor for rendering, marker, synthesis, or cleanup errors", async () => {
  temporary = await mkdtemp(join(tmpdir(), "opencut-doctor-test-"));
  const config = loadBridgeConfig({
    OPENCUT_CONFIG_ROOT: temporary,
    OPENCUT_KOKORO_PYTHON: process.execPath,
  });
  const rendering = await runDoctor(
    config,
    headless(false),
    speech(),
    platform()
  );
  expect(rendering.ready).toBe(false);
  expect(rendering.checks).toContainEqual(
    expect.objectContaining({ id: "rendering", status: "error" })
  );

  const marker = await runDoctor(config, headless(), speech(false), platform());
  expect(marker.ready).toBe(false);
  expect(marker.checks).toContainEqual(
    expect.objectContaining({ id: "speech", status: "error" })
  );

  const cleanup = await runDoctor(
    config,
    headless(),
    speech(true, true),
    platform()
  );
  expect(cleanup.ready).toBe(false);
  expect(cleanup.checks).toContainEqual(
    expect.objectContaining({ id: "synthesis", status: "error" })
  );
});

it("reports Python, disk, and permission diagnostics deterministically", async () => {
  temporary = await mkdtemp(join(tmpdir(), "opencut-doctor-test-"));
  const config = loadBridgeConfig({
    OPENCUT_CONFIG_ROOT: temporary,
    OPENCUT_KOKORO_PYTHON: process.execPath,
  });
  const report = await runDoctor(
    config,
    headless(),
    speech(),
    platform({
      capture: () => Promise.resolve({ code: null, stderr: "", stdout: "" }),
      freeBytes: () => Promise.reject(new Error("disk unavailable")),
      writable: () => Promise.reject(new Error("permission denied")),
    })
  );
  expect(report.ready).toBe(false);
  expect(report.checks).toEqual(
    expect.arrayContaining([
      expect.objectContaining({ id: "python", status: "error" }),
      expect.objectContaining({ id: "disk", status: "error" }),
      expect.objectContaining({ id: "writeability", status: "error" }),
    ])
  );

  const lowDisk = await runDoctor(
    config,
    headless(),
    speech(),
    platform({ freeBytes: () => Promise.resolve(4 * 1024 * 1024 * 1024) })
  );
  expect(lowDisk.ready).toBe(true);
  expect(lowDisk.checks).toContainEqual(
    expect.objectContaining({ id: "disk", status: "warning" })
  );
});
