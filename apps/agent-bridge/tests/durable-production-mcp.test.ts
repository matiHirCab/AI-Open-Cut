import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";
import fixture from "../../../contracts/durable-production-requests-v1.json";
import { loadBridgeConfig } from "../src/config";
import { durableHttpHealth } from "../src/production-requests/contract";
import { registerProductionRequestTools } from "../src/server/production-requests";
import type { Server, ServerDependencies } from "../src/server/shared";
import { verifyDurableProductionContract } from "./durable-production-contract";

it(
  "governs independently authored durable MCP and exact raw predecessors",
  verifyDurableProductionContract
);

it("registers all five disabled tools without opening or creating durable storage", async () => {
  const root = mkdtempSync(join(tmpdir(), "opencut-durable-disabled-"));
  try {
    const config = loadBridgeConfig({ OPENCUT_TTS_WORK_DIR: root });
    const handlers = new Map<
      string,
      (
        input: unknown
      ) => Promise<{ isError?: boolean; structuredContent: unknown }>
    >();
    const server = {
      registerTool: (
        name: string,
        _definition: unknown,
        handler: (input: unknown) => Promise<{ structuredContent: unknown }>
      ) => handlers.set(name, handler),
    } as unknown as Server;
    registerProductionRequestTools(server, { config } as ServerDependencies);
    expect([...handlers.keys()].sort()).toEqual([...fixture.toolNames].sort());
    const responses = await Promise.all(
      [...handlers.values()].map((handler) => handler(fixture.input))
    );
    for (const response of responses) {
      expect(response.isError).toBe(true);
      expect(response.structuredContent).toMatchObject({
        error: { code: "DEPENDENCY_UNAVAILABLE", retryable: false },
      });
    }
    expect(readdirSync(root)).toEqual([]);
  } finally {
    rmSync(root, { force: true, recursive: true });
  }
});

it("enables durable requests only with the exact opt-in configuration value", () => {
  for (const value of [undefined, "0"]) {
    expect(
      loadBridgeConfig({ OPENCUT_DURABLE_PRODUCTION_REQUESTS: value })
        .durableProductionRequests
    ).toBe(false);
  }
  expect(
    loadBridgeConfig({ OPENCUT_DURABLE_PRODUCTION_REQUESTS: "1" })
      .durableProductionRequests
  ).toBe(true);
  for (const value of ["", "true", "yes", " 1", "2"]) {
    expect(() =>
      loadBridgeConfig({ OPENCUT_DURABLE_PRODUCTION_REQUESTS: value })
    ).toThrow(expect.objectContaining({ code: "INVALID_CONFIGURATION" }));
  }
});

it("preserves the legacy HTTP health shape and adds only enabled API support", () => {
  const legacy = fixture.httpHealthExamples.disabled;
  expect(durableHttpHealth(false, legacy)).toBe(legacy);
  expect(JSON.stringify(durableHttpHealth(false, legacy))).toBe(
    JSON.stringify(legacy)
  );
  expect(durableHttpHealth(true, legacy)).toEqual(
    fixture.httpHealthExamples.enabled
  );
});
