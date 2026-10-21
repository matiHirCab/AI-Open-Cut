import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import { restoreDurable228Raw } from "./fixtures/durable-main47-peers";
import pins from "./fixtures/durable-main47-pins.json";
import { restoreMain47Raw } from "./fixtures/durable-main47-projection";

const versionPattern = /"version":\s*1/;
const digest = (raw: string) => createHash("sha256").update(raw).digest("hex");
for (const name of [
  "mcp-surface-v1",
  "contract-ownership-v1",
  "headless-protocol-v1",
] as const) {
  const raw = readFileSync(
    new URL(`../../../contracts/${name}.json`, import.meta.url),
    "utf8"
  );
  for (const [input, key] of [
    [raw, "rawSha256"],
    [JSON.stringify(JSON.parse(raw)), "compactSha256"],
  ] as const) {
    for (const [predecessor, restore] of [
      ["main47", restoreMain47Raw],
      ["durable228", restoreDurable228Raw],
    ] as const) {
      it(`recovers independently pinned ${predecessor} ${name} ${key}`, () => {
        const restored = restore(input);
        expect(digest(restored)).toBe(pins[predecessor][name][key]);
        expect(restore(restored)).toBe(restored);
        const changed = input.replace(versionPattern, '"version":2');
        expect(changed).not.toBe(input);
        expect(digest(restore(changed))).not.toBe(pins[predecessor][name][key]);
        const foreign = '"foreign":9007199254740993,';
        expect(restore(input.replace("{", `{${foreign}`))).toBe(
          restored.replace("{", `{${foreign}`)
        );
      });
    }
  }
}
it("rejects missing, duplicate and narrowed durable additions", () => {
  const raw = readFileSync(
    new URL("../../../contracts/mcp-surface-v1.json", import.meta.url),
    "utf8"
  );
  const source = JSON.parse(raw);
  const name = "production_request_get";
  expect(source.toolDefinitions[name]).toBeDefined();
  const missing = structuredClone(source);
  delete missing.toolDefinitions[name];
  expect(() => restoreMain47Raw(JSON.stringify(missing))).toThrow();
  const duplicate = structuredClone(source);
  duplicate.tools.push(name);
  expect(() => restoreMain47Raw(JSON.stringify(duplicate))).toThrow();
  const narrowed = structuredClone(source);
  narrowed.toolDefinitions[name].inputSchema = {};
  expect(() => restoreMain47Raw(JSON.stringify(narrowed))).toThrow();
});
