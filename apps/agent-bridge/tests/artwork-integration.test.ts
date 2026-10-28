import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import { restoreArtworkBranchRaw } from "./fixtures/artwork-integration-history";
import pins from "./fixtures/artwork-integration-pins.json";
import { restoreArtworkIntegrationRaw } from "./fixtures/artwork-integration-projection";
import { restoreMain47Raw } from "./fixtures/durable-main47-projection";

const digest = (raw: string) => createHash("sha256").update(raw).digest("hex");
for (const [name, expected] of Object.entries(pins.main47)) {
  it(`recovers exact independently fetched main47 and artwork47: ${name}`, () => {
    const raw = readFileSync(
      new URL(`../../../contracts/${name}`, import.meta.url),
      "utf8"
    );
    expect(digest(restoreMain47Raw(raw))).toBe(expected);
    expect(digest(restoreArtworkBranchRaw(raw))).toBe(
      pins.artwork47[name as keyof typeof pins.artwork47]
    );
    expect(digest(restoreMain47Raw(`${raw}\n`))).not.toBe(expected);
  });
}
it("rejects missing duplicate or malformed owned schema and operation additions", () => {
  const raw = readFileSync(
    new URL("../../../contracts/headless-protocol-v1.json", import.meta.url),
    "utf8"
  );
  for (const bad of [
    raw.replace('    "commit_artwork",\n', ""),
    raw.replace(
      '    "commit_artwork",\n',
      '    "commit_artwork",\n    "commit_artwork",\n'
    ),
  ]) {
    expect(() => restoreArtworkIntegrationRaw(bad)).toThrow();
  }
  const mcp = JSON.parse(
    readFileSync(
      new URL("../../../contracts/mcp-surface-v1.json", import.meta.url),
      "utf8"
    )
  );
  for (const mutate of [
    (value: typeof mcp) =>
      Reflect.deleteProperty(
        value.$defs.ProjectPropertiesAssetsItemsProperties,
        "artworkDerivation"
      ),
    (value: typeof mcp) => {
      value.toolDefinitions.editor_get_status.outputSchema.properties.projectSchemaVersion.const = 49;
    },
    (value: typeof mcp) => {
      value.$defs.ProjectPropertiesAssetsItemsProperties.artworkDerivation.properties.sourceWidth.maximum = 511;
    },
  ]) {
    const bad = structuredClone(mcp);
    mutate(bad);
    expect(() => restoreArtworkIntegrationRaw(JSON.stringify(bad))).toThrow();
  }
});
