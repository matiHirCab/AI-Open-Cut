import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import CATALOG from "../../../contracts/render-budget-diagnostics-v1.json";
import { errorBody } from "../src/headless";
import { BridgeError, eventSchema } from "../src/headless-events";
import { RENDER_BUDGETS, renderBudgetSchema } from "../src/render-budget";
import { publicErrorSchema } from "../src/schemas";
import pins from "./fixtures/render-budget-predecessor-pins.json";
import {
  removeRenderBudgetMcpAddition,
  restoreRenderBudgetRaw,
} from "./fixtures/render-budget-projection";

it("preserves governed native charges through headless and public bridge errors", () => {
  expect(RENDER_BUDGETS).toEqual(CATALOG.budgets);
  for (const fixture of CATALOG.fixtures) {
    expect(
      renderBudgetSchema.safeParse(fixture.value).success,
      fixture.name
    ).toBe(fixture.valid);
    if (!fixture.valid) {
      continue;
    }
    const diagnostic = renderBudgetSchema.parse(fixture.value);
    const value = {
      ...CATALOG.error,
      failedStage: null,
      ffmpegExitCode: null,
      ffmpegStderrExcerpt: null,
      message: "native admission limit exceeded",
      renderBudget: diagnostic,
    };
    const event = eventSchema.parse({ error: value, type: "error" });
    expect(event.type).toBe("error");
    const error = new BridgeError(
      value.code,
      value.message,
      value.retryable,
      undefined,
      value
    );
    expect(publicErrorSchema.parse(errorBody(error))).toEqual(value);
    expect(errorBody(error).message).not.toContain("FFmpeg");
  }
  const ordinary = errorBody(
    new BridgeError("REVISION_CONFLICT", "stale revision", true)
  );
  expect(Object.hasOwn(ordinary, "renderBudget")).toBe(false);
  expect(ordinary.retryable).toBe(true);
});

it("rejects diagnostic schema drift without suppressing unrelated predecessor changes", () => {
  for (const [name, hash] of Object.entries(pins.rawSha256)) {
    const raw = readFileSync(
      new URL(`../../../contracts/${name}.json`, import.meta.url),
      "utf8"
    );
    expect(
      createHash("sha256").update(restoreRenderBudgetRaw(raw)).digest("hex")
    ).toBe(hash);
  }
  const source = JSON.parse(
    readFileSync(
      new URL("../../../contracts/mcp-surface-v1.json", import.meta.url),
      "utf8"
    )
  );
  source.$defs.RenderBudgetDiagnostic.properties.contributors.maxItems = 9;
  expect(() => removeRenderBudgetMcpAddition(source)).toThrow("Unapproved");
  const ordinary = JSON.parse(
    readFileSync(
      new URL("../../../contracts/mcp-surface-v1.json", import.meta.url),
      "utf8"
    )
  );
  ordinary.version = 99;
  const projected = removeRenderBudgetMcpAddition(ordinary);
  expect(projected.version).toBe(99);
  expect(() =>
    restoreRenderBudgetRaw(`${JSON.stringify(ordinary, null, 2)}\n`)
  ).toThrow("Unrelated");
});

it("rejects all missing budget fields with a retained definition and preserves historical catalogs", () => {
  const source = JSON.parse(
    readFileSync(
      new URL("../../../contracts/mcp-surface-v1.json", import.meta.url),
      "utf8"
    )
  );
  const incomplete = structuredClone(source);
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
  expect(incomplete.$defs.RenderBudgetDiagnostic).toEqual(CATALOG.schema);
  expect(() => removeRenderBudgetMcpAddition(incomplete)).toThrow(
    "Missing native budget schema addition"
  );
  expect(() => restoreRenderBudgetRaw(JSON.stringify(incomplete))).toThrow(
    "Missing native budget schema addition"
  );
  const historicalRaw = restoreRenderBudgetRaw(JSON.stringify(source));
  const historical = JSON.parse(historicalRaw);
  expect(historical.$defs?.RenderBudgetDiagnostic).toBeUndefined();
  expect(removeRenderBudgetMcpAddition(historical)).toEqual(historical);
  expect(restoreRenderBudgetRaw(historicalRaw)).toBe(historicalRaw);
  expect(createHash("sha256").update(historicalRaw).digest("hex")).toBe(
    pins.rawSha256["mcp-surface-v1"]
  );
});
