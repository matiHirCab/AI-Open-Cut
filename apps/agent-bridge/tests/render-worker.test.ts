import { existsSync, mkdtempSync, readFileSync } from "node:fs";
import { rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, expect, it, vi } from "vitest";
import { z } from "zod/v4";
import FIXTURE from "../../../contracts/render-worker-v1.json";
import { loadBridgeConfig } from "../src/config";
import { HeadlessClient } from "../src/headless";
import type { HeadlessRequest } from "../src/headless-contract";
import {
  RenderWorker,
  WORKER_LINE_BYTES,
  WORKER_STARTUP_MS,
  workerEventSchema,
  workerReadySchema,
} from "../src/render-worker";

const resources: { client: HeadlessClient; root: string }[] = [];
const resultSchema = z
  .object({ count: z.number(), pid: z.number(), worker: z.boolean() })
  .strict();
const request = (testMode = "ok", operation = "render_preview") =>
  ({
    expectedRevision: 0,
    operation,
    projectId: "project",
    testMode,
    timeMs: 0,
  }) as unknown as HeadlessRequest;
const create = (startup = "normal", timeout = 10_000) => {
  const root = mkdtempSync(join(tmpdir(), "opencut-worker-"));
  const config = loadBridgeConfig({
    ...process.env,
    OPENCUT_EXPORTS_DIR: join(root, "exports"),
    OPENCUT_PROJECTS_DIR: root,
  });
  const client = new HeadlessClient({
    ...config,
    environment: {
      ...config.environment,
      OPENCUT_TEST_WORKER_STARTUP: startup,
    },
    headlessArguments: [
      resolve(import.meta.dirname, "fixtures/fake-render-worker.ts"),
    ],
    headlessPath: process.execPath,
    headlessRequestTimeoutMs: timeout,
  });
  resources.push({ client, root });
  return { client, root };
};
afterEach(async () => {
  await Promise.all(
    resources.splice(0).map(async ({ client, root }) => {
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

it("matches canonical worker events, version, limits and render operation routing", () => {
  expect(workerReadySchema.parse(FIXTURE.ready)).toEqual(FIXTURE.ready);
  expect(WORKER_LINE_BYTES).toBe(FIXTURE.maxLineBytes);
  expect(WORKER_STARTUP_MS).toBe(FIXTURE.startupTimeoutMs);
  for (const event of FIXTURE.events) {
    expect(workerEventSchema.parse(event)).toEqual(event);
  }
  for (const envelope of FIXTURE.requests) {
    expect(RenderWorker.accepts(envelope.request as HeadlessRequest)).toBe(
      true
    );
  }
  expect(
    RenderWorker.accepts(FIXTURE.rejectedOperation.request as HeadlessRequest)
  ).toBe(false);
  expect(
    workerReadySchema.safeParse({ ...FIXTURE.ready, protocolVersion: 2 })
      .success
  ).toBe(false);
  expect(
    workerEventSchema.safeParse({ ...FIXTURE.events[0], extra: true }).success
  ).toBe(false);
});

it("reuses the warm worker, handles split output and keeps typed errors reusable", async () => {
  const { client } = create();
  const first = await client.call(request("partial"), resultSchema);
  const second = await client.call(request(), resultSchema);
  expect(second).toEqual({ ...first, count: 2 });
  await expect(
    client.call(request("error"), resultSchema)
  ).rejects.toMatchObject({ code: "REVISION_CONFLICT", retryable: true });
  expect(await client.call(request(), resultSchema)).toEqual({
    ...first,
    count: 4,
  });
});

it("uses one-shot overflow and cancellation cleans draft temporaries before replacement", async () => {
  const { client, root } = create();
  const warm = await client.call(request(), resultSchema);
  const controller = new AbortController();
  const pending = client.call(
    request("hang-tree", "render_draft_preview"),
    resultSchema,
    { requestId: "cancelled", signal: controller.signal }
  );
  const rejected = expect(pending).rejects.toMatchObject({
    code: "JOB_CANCELLED",
  });
  const overflowPending = client.call(request("slow"), resultSchema);
  const descendantPath = join(root, "project", "descendant.pid");
  await vi.waitFor(() => expect(existsSync(descendantPath)).toBe(true));
  const descendant = Number(readFileSync(descendantPath, "utf8"));
  controller.abort();
  const overflow = await overflowPending;
  expect(overflow.worker).toBe(false);
  expect(overflow.pid).not.toBe(warm.pid);
  await rejected;
  expect(() => process.kill(descendant, 0)).toThrow();
  expect(
    readFileSync(join(root, "project", "previews", "published.png"), "utf8")
  ).toBe("published");
  expect(existsSync(join(root, "project", ".opencut-work-cancelled"))).toBe(
    false
  );
  expect(
    existsSync(join(root, "project", "previews", ".opencut-cancelled.png"))
  ).toBe(false);
  expect((await client.call(request(), resultSchema)).pid).not.toBe(warm.pid);
});

it.each([
  "malformed",
  "wrong-id",
  "duplicate",
  "crash",
  "invalid-utf8",
  "oversized",
])("discards %s workers without replay", async (mode) => {
  const { client } = create();
  const warm = await client.call(request(), resultSchema);
  await expect(client.call(request(mode), resultSchema)).rejects.toMatchObject({
    code: "INTERNAL_ERROR",
  });
  expect((await client.call(request(), resultSchema)).pid).not.toBe(warm.pid);
});

it("times out active work and refuses calls after shutdown", async () => {
  const { client } = create();
  await client.call(request(), resultSchema);
  await expect(
    client.call(request("hang"), resultSchema, { timeoutMs: 50 })
  ).rejects.toMatchObject({ code: "HEADLESS_TIMEOUT", retryable: true });
  await client.close();
  await expect(client.call(request(), resultSchema)).rejects.toMatchObject({
    code: "BRIDGE_SHUTTING_DOWN",
  });
});

it("accepts the inclusive event limit", async () => {
  const { client } = create();
  const result = await client.call(
    request("limit"),
    resultSchema.extend({ padding: z.string() })
  );
  expect(result.padding.length).toBeGreaterThan(WORKER_LINE_BYTES - 200);
  expect((await client.call(request(), resultSchema)).pid).toBe(result.pid);
});

it.each(["version", "exit"])(
  "rejects %s startup without fallback",
  async (mode) => {
    const { client } = create(mode);
    await expect(client.call(request(), resultSchema)).rejects.toMatchObject({
      code: "DEPENDENCY_UNAVAILABLE",
    });
  }
);

it("includes startup in the request deadline", async () => {
  const { client } = create("hang", 50);
  await expect(client.call(request(), resultSchema)).rejects.toMatchObject({
    code: "HEADLESS_TIMEOUT",
    retryable: true,
  });
});

it("enforces the worker readiness deadline", async () => {
  const { client } = create("hang", 10_000);
  await expect(client.call(request(), resultSchema)).rejects.toMatchObject({
    code: "DEPENDENCY_UNAVAILABLE",
  });
}, 10_000);

it("closes an active worker without a replacement reservation", async () => {
  const { client } = create();
  await client.call(request(), resultSchema);
  const pending = client.call(request("hang"), resultSchema);
  const rejected = expect(pending).rejects.toMatchObject({
    code: "JOB_CANCELLED",
  });
  await client.close();
  await rejected;
  await expect(client.call(request(), resultSchema)).rejects.toMatchObject({
    code: "BRIDGE_SHUTTING_DOWN",
  });
});
