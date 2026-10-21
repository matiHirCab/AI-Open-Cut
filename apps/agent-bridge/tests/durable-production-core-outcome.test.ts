import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, expect, it } from "vitest";
import { z } from "zod/v4";
import { loadBridgeConfig } from "../src/config";
import { BridgeError, HeadlessClient } from "../src/headless";
import type { HeadlessRequest } from "../src/headless-contract";
import { isConfirmedCoreError } from "../src/production-requests/core-outcome";

const resources: { client: HeadlessClient; root: string }[] = [];
afterEach(async () => {
  await Promise.all(
    resources.splice(0).map(async ({ client, root }) => {
      await client.close();
      rmSync(root, { force: true, recursive: true });
    })
  );
});

it.each(["get_state", "render_preview"])(
  "marks only a validated core error in %s transport",
  async (operation) => {
    const root = mkdtempSync(join(tmpdir(), "opencut-durable-core-"));
    const config = loadBridgeConfig({
      OPENCUT_EXPORTS_DIR: join(root, "exports"),
      OPENCUT_PROJECTS_DIR: root,
    });
    const client = new HeadlessClient({
      ...config,
      headlessArguments: [
        resolve(import.meta.dirname, "fixtures/fake-render-worker.ts"),
      ],
      headlessPath: process.execPath,
    });
    resources.push({ client, root });
    const request = (testMode: string) =>
      ({
        expectedRevision: 4,
        operation,
        projectId: "project-original",
        testMode,
        timeMs: 250,
      }) as unknown as HeadlessRequest;
    const nativeError = await client
      .call(request("error"), z.unknown())
      .catch((error: unknown) => error);
    expect(nativeError).toMatchObject({
      code: "REVISION_CONFLICT",
      retryable: true,
    });
    expect(isConfirmedCoreError(nativeError)).toBe(true);
    expect(
      isConfirmedCoreError(new BridgeError("REVISION_CONFLICT", "stale", true))
    ).toBe(false);
    const protocolError = await client
      .call(request("malformed"), z.unknown())
      .catch((error: unknown) => error);
    expect(protocolError).toBeInstanceOf(BridgeError);
    expect(isConfirmedCoreError(protocolError)).toBe(false);
  }
);
