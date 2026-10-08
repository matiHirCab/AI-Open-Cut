import { existsSync, mkdtempSync, readFileSync } from "node:fs";
import { rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, expect, it, vi } from "vitest";
import { z } from "zod/v4";
import { loadBridgeConfig } from "../src/config";
import { HeadlessClient } from "../src/headless";
import type { HeadlessRequest } from "../src/headless-contract";

const output = z
  .object({ count: z.number(), pid: z.number(), worker: z.boolean() })
  .strict();
const request = (testMode = "ok") =>
  ({
    endMs: 1000,
    expectedRevision: 0,
    operation: "analyze_audio",
    projectId: "project",
    startMs: 0,
    testMode,
    waveformBins: 2,
  }) as unknown as HeadlessRequest;
const resources: { root: string; client: HeadlessClient }[] = [];
const create = () => {
  const root = mkdtempSync(join(tmpdir(), "analysis-lifetime-"));
  const config = loadBridgeConfig({
    ...process.env,
    OPENCUT_EXPORTS_DIR: join(root, "exports"),
    OPENCUT_PROJECTS_DIR: root,
  });
  const client = new HeadlessClient({
    ...config,
    headlessArguments: [
      resolve(import.meta.dirname, "fixtures/fake-render-worker.ts"),
    ],
    headlessPath: process.execPath,
    headlessRequestTimeoutMs: 10_000,
  });
  resources.push({ client, root });
  return { client, root };
};
afterEach(async () => {
  await Promise.all(
    resources.splice(0).map(async ({ root, client }) => {
      await client.close();
      await rm(root, {
        force: true,
        maxRetries: 10,
        recursive: true,
        retryDelay: 50,
      });
    })
  );
});
it.each(["cancel", "deadline", "shutdown"])(
  "reaps analysis descendants and owned JSON for %s while retaining published files",
  async (mode) => {
    const { root, client } = create();
    const warm = await client.call(request(), output);
    const controller = new AbortController();
    const pending = client.call(request("hang-tree"), output, {
      requestId: "analysis-owned",
      signal: controller.signal,
      ...(mode === "deadline" ? { timeoutMs: 300 } : {}),
    });
    const rejected = expect(pending).rejects.toMatchObject({
      code: mode === "deadline" ? "HEADLESS_TIMEOUT" : "JOB_CANCELLED",
    });
    const pidFile = join(root, "project/descendant.pid");
    await vi.waitFor(() => expect(existsSync(pidFile)).toBe(true));
    const pid = Number(readFileSync(pidFile, "utf8"));
    const overlap =
      mode === "shutdown" ? null : client.call(request("slow"), output);
    if (mode === "cancel") {
      controller.abort();
    }
    if (mode === "shutdown") {
      await client.close();
    }
    await rejected;
    expect(() => process.kill(pid, 0)).toThrow();
    expect(existsSync(join(root, "project/.opencut-work-analysis-owned"))).toBe(
      false
    );
    expect(
      existsSync(join(root, "project/previews/.opencut-analysis-owned.json"))
    ).toBe(false);
    expect(
      readFileSync(join(root, "project/previews/published.png"), "utf8")
    ).toBe("published");
    if (overlap) {
      const other = await overlap;
      expect(other.worker).toBe(false);
      expect(other.pid).not.toBe(warm.pid);
      expect((await client.call(request(), output)).pid).not.toBe(warm.pid);
    }
  }
);
it("reuses sequential analysis work, preserving typed conflicts and subsequent rendering", async () => {
  const { client } = create();
  const first = await client.call(request(), output);
  const second = await client.call(request(), output);
  expect(second).toEqual({ ...first, count: 2 });
  await expect(client.call(request("error"), output)).rejects.toMatchObject({
    code: "REVISION_CONFLICT",
    retryable: true,
  });
  const rendered = await client.call(
    {
      expectedRevision: 0,
      operation: "render_preview",
      projectId: "project",
      timeMs: 0,
    },
    output
  );
  expect(rendered).toEqual({ ...first, count: 4 });
});
