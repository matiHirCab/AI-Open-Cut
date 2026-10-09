import { existsSync, mkdirSync, mkdtempSync } from "node:fs";
import { rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

import { afterEach, beforeEach, expect, it } from "vitest";
import { z } from "zod/v4";

import { type BridgeConfig, loadBridgeConfig } from "../src/config";
import { callHeadless, HeadlessClient } from "../src/headless";
import type { HeadlessRequest } from "../src/headless-contract";

const fixtureRequest = (operation: string, extra = {}) =>
  ({ operation, ...extra }) as unknown as HeadlessRequest;

let root = "";
let client: HeadlessClient | undefined;

const createConfig = (timeoutMs = 1000): BridgeConfig => {
  const projectsDirectory = join(root, "projects");
  const exportsDirectory = join(root, "exports");
  mkdirSync(projectsDirectory, { recursive: true });
  mkdirSync(exportsDirectory, { recursive: true });
  const base = loadBridgeConfig({
    ...process.env,
    OPENCUT_EXPORTS_DIR: exportsDirectory,
    OPENCUT_PROJECTS_DIR: projectsDirectory,
  });
  return {
    ...base,
    environment: {
      ...base.environment,
      OPENCUT_EXPORTS_DIR: exportsDirectory,
      OPENCUT_PROJECTS_DIR: projectsDirectory,
    },
    exportsDirectory,
    headlessArguments: [
      resolve(import.meta.dirname, "fixtures/fake-headless.ts"),
    ],
    headlessPath: process.execPath,
    headlessRequestTimeoutMs: timeoutMs,
    projectsDirectory,
  };
};

beforeEach(() => {
  root = mkdtempSync(join(tmpdir(), "opencut-headless-client-"));
});

afterEach(async () => {
  await client?.close();
  await rm(root, {
    force: true,
    maxRetries: 5,
    recursive: true,
    retryDelay: 50,
  });
});

it("accepts protocol events split across stdout chunks", async () => {
  client = new HeadlessClient(createConfig());
  await expect(
    client.call(fixtureRequest("partial"), z.object({ ok: z.literal(true) }))
  ).resolves.toEqual({ ok: true });
});

it("terminates timed-out requests and removes owned preview output", async () => {
  const config = createConfig(100);
  client = new HeadlessClient(config);
  await expect(
    client.call(
      fixtureRequest("render_preview", {
        projectId: "project",
        testMode: "hang",
      }),
      z.object({ ok: z.boolean() })
    )
  ).rejects.toMatchObject({ code: "HEADLESS_TIMEOUT", retryable: true });
  const previews = join(config.projectsDirectory ?? "", "project", "previews");
  const files = existsSync(previews)
    ? (await import("node:fs/promises")).readdir(previews)
    : Promise.resolve([]);
  await expect(files).resolves.toEqual([]);
});

it("maps malformed output and cancellation to stable errors", async () => {
  client = new HeadlessClient(createConfig());
  await expect(
    client.call(fixtureRequest("malformed"), z.object({ ok: z.boolean() }))
  ).rejects.toMatchObject({ code: "INTERNAL_ERROR" });

  const controller = new AbortController();
  const request = client.call(
    fixtureRequest("hang"),
    z.object({ ok: z.boolean() }),
    { signal: controller.signal }
  );
  controller.abort();
  await expect(request).rejects.toMatchObject({
    code: "JOB_CANCELLED",
    retryable: true,
  });
});

it("refuses cleanup outside the configured root for malformed export paths", async () => {
  const config = createConfig();
  const { readFile, writeFile } = await import("node:fs/promises");
  const victim = join(root, ".opencut-cleanup-safe.mp4");
  await writeFile(victim, "preserve");
  await expect(
    callHeadless(
      config,
      fixtureRequest("export_video", {
        projectId: "project",
        relativePath: "../output.mp4",
      }),
      z.object({ ok: z.boolean() }),
      { requestId: "cleanup-safe" }
    )
  ).rejects.toMatchObject({ code: "INTERNAL_ERROR" });
  expect((await readFile(victim)).toString()).toBe("preserve");
});
