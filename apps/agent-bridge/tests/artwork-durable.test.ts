import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import {
  assertArtworkDurableConsumer,
  verifyArtworkDurableContract,
} from "./artwork-durable-contract";
import pins from "./fixtures/artwork-durable-input-pins.json";
import { restoreDurableIntegrationRaw } from "./fixtures/artwork-durable-projection";
import { restoreArtworkIntegrationRaw } from "./fixtures/artwork-integration-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";

it(
  "recovers both frozen artwork and durable inputs with exact independent inverses",
  verifyArtworkDurableContract
);
it("rejects missing duplicate narrowed and unrelated integration additions", () => {
  const original = JSON.parse(
    readFileSync(
      new URL("../../../contracts/mcp-surface-v1.json", import.meta.url),
      "utf8"
    )
  );
  for (const mutate of [
    (value: typeof original) =>
      Reflect.deleteProperty(
        value.$defs.ProjectPropertiesAssetsItemsProperties,
        "artworkDerivation"
      ),
    (value: typeof original) => {
      value.toolDefinitions.editor_get_status.outputSchema.properties.projectSchemaVersion.const = 49;
    },
    (value: typeof original) => {
      value.$defs.ProjectPropertiesAssetsItemsProperties.artworkDerivation.properties.sourceWidth.maximum = 511;
    },
  ]) {
    const value = structuredClone(original);
    mutate(value);
    expect(() => restoreArtworkIntegrationRaw(JSON.stringify(value))).toThrow();
  }
  for (const mutate of [
    (value: typeof original) =>
      Reflect.deleteProperty(value.toolDefinitions, "production_request_get"),
    (value: typeof original) => {
      value.tools.push("production_request_get");
    },
    (value: typeof original) => {
      value.toolDefinitions.production_request_get.inputSchema = {};
    },
  ]) {
    const value = structuredClone(original);
    mutate(value);
    expect(() => restoreDurableIntegrationRaw(JSON.stringify(value))).toThrow();
  }
  const ownership = JSON.parse(
    readFileSync(
      new URL("../../../contracts/contract-ownership-v1.json", import.meta.url),
      "utf8"
    )
  );
  for (const [category, restore] of [
    ["artworkPreparation", restoreArtworkIntegrationRaw],
    ["durableProductionRequests", restoreDurableIntegrationRaw],
    ["sourceGroundedShortPlanning", restoreDurableIntegrationRaw],
  ] as const) {
    const value = structuredClone(ownership);
    value.categories[category].consumers.pop();
    expect(() => restore(JSON.stringify(value))).toThrow();
  }
});
it("keeps the new integration evidence mandatory in the unchanged contracts gate", () => {
  const source = readFileSync(
    new URL("./contracts.test.ts", import.meta.url),
    "utf8"
  );
  assertArtworkDurableConsumer(source);
  const registration = /it\(\s*"[^"]+",\s*verifyArtworkDurableContract\s*\)/g;
  for (const replace of [
    (s: string) => s.replace("it(", "it.skip("),
    (s: string) => `/* ${s} */`,
    () => "",
  ]) {
    const changed = source.replace(registration, replace);
    expect(changed).not.toBe(source);
    expect(() => assertArtworkDurableConsumer(changed)).toThrow();
  }
});

for (const [label, restore] of [
  ["main", restoreArtworkIntegrationRaw],
  ["artwork", restoreDurableIntegrationRaw],
] as const) {
  it(`preserves independently frozen ${label} expanded MCP and unrelated raw lexemes`, () => {
    const raw = readFileSync(
      new URL("../../../contracts/mcp-surface-v1.json", import.meta.url),
      "utf8"
    );
    const restored = restore(raw);
    const expanded = expandMcpSurfaceCatalog(JSON.parse(restored));
    expect(
      createHash("sha256").update(JSON.stringify(expanded)).digest("hex")
    ).toBe(
      pins.catalogs[label]["contracts/mcp-surface-v1.json"].expandedSha256
    );
    const foreign = '"reviewer":9007199254740993,';
    expect(restore(raw.replace("{", `{${foreign}`))).toBe(
      restored.replace("{", `{${foreign}`)
    );
    expect(restore(`${raw} `)).toBe(`${restored} `);
  });
}
