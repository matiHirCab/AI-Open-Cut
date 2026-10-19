// Disposable-only integration witnesses; not imported into either publishing branch.
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import { restoreDiagnosticUnionRaw } from "./fixtures/diagnostic-union-projection";
import { removeDurableMcpAdditions } from "./fixtures/durable-production-projection";
import pins from "./fixtures/ffmpeg-spawn-predecessor-pins.json";

it("rejects a complete loss of budget fields through combined rollback and retains every historical pin", () => {
  for (const [name, hash] of Object.entries(pins.rawSha256)) {
    const source = readFileSync(
      new URL(`../../../contracts/${name}.json`, import.meta.url),
      "utf8"
    );
    const historical = restoreDiagnosticUnionRaw(source);
    expect(createHash("sha256").update(historical).digest("hex")).toBe(hash);
    expect(restoreDiagnosticUnionRaw(historical)).toBe(historical);
  }
  const incomplete = removeDurableMcpAdditions(
    JSON.parse(
      readFileSync(
        new URL("../../../contracts/mcp-surface-v1.json", import.meta.url),
        "utf8"
      )
    )
  );
  let removed = 0;
  const walk = (value: unknown) => {
    if (!value || typeof value !== "object") {
      return;
    }
    const { properties } = value as {
      properties?: Record<string, unknown>;
    };
    if (properties && Object.hasOwn(properties, "renderBudget")) {
      Reflect.deleteProperty(properties, "renderBudget");
      removed += 1;
    }
    for (const child of Object.values(value)) {
      walk(child);
    }
  };
  walk(incomplete);
  expect(removed).toBe(6);
  expect(incomplete.$defs.RenderBudgetDiagnostic).toBeDefined();
  expect(incomplete.$defs.FfmpegSpawnDiagnostic).toBeDefined();
  expect(() => restoreDiagnosticUnionRaw(JSON.stringify(incomplete))).toThrow(
    "Missing native budget schema addition"
  );
});
