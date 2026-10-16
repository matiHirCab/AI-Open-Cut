import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import MCP from "../../../contracts/mcp-surface-v1.json";
import contract from "../../../contracts/narration-profiles-v1.json";
import {
  narrationProfileAttributionSchema,
  narrationProfileSettingsSchema,
  schemas,
} from "../src/schemas";
import markerPins from "./fixtures/narration-profiles-marker-pins.json";
import pins from "./fixtures/narration-profiles-predecessor-pins.json";
import {
  removeNarrationProfileAdditions,
  restoreNarrationProfileRaw,
} from "./fixtures/narration-profiles-projection";

it("accepts canonical profile settings and attribution without introducing provider ownership", () => {
  expect(narrationProfileSettingsSchema.parse(contract.settings)).toEqual(
    contract.settings
  );
  expect(narrationProfileAttributionSchema.parse(contract.attribution)).toEqual(
    contract.attribution
  );
  expect(
    schemas.narrationProfileApply.parse({
      application: {
        expectedVersion: 1,
        profileId: contract.attribution.id,
        text: "Hola",
      },
    }).application.text
  ).toBe("Hola");
});

it("keeps omitted profile options absent while preserving explicit empty replacement", () => {
  const base = {
    expectedVersion: 1,
    profileId: contract.attribution.id,
    text: "Hola",
  };
  expect(
    schemas.narrationProfileApply.parse({ application: base }).application
  ).not.toHaveProperty("textOptions");
  const empty = { ...contract.settings.textOptions, pronunciations: [] };
  expect(
    schemas.narrationProfileApply.parse({
      application: { ...base, textOptions: empty },
    }).application.textOptions
  ).toEqual(empty);
});

it("retains unrelated matching fields while inverting only approved profile additions", () => {
  const source = structuredClone(MCP);
  (
    source.toolDefinitions.editor_get_status.annotations as Record<
      string,
      unknown
    >
  ).narrationProfile = {
    name: "keep",
    version: 46,
  };
  const projected = removeNarrationProfileAdditions(source, "mcp-surface-v1");
  expect(projected.toolDefinitions.editor_get_status.annotations).toMatchObject(
    { narrationProfile: { name: "keep", version: 46 } }
  );
  expect(projected.toolDefinitions.narration_profile_apply).toBeUndefined();
  expect(projected.tools).toHaveLength(87);
  expect(source.toolDefinitions.narration_profile_apply).toBeDefined();
  source.tools.push("narration_profile_apply");
  expect(() =>
    removeNarrationProfileAdditions(source, "mcp-surface-v1")
  ).toThrow("Incorrect narration profile addition");
});
it.each([
  "mcp-surface-v1",
  "headless-protocol-v1",
  "contract-ownership-v1",
] as const)("preserves independently pinned raw predecessor for %s", (name) => {
  const raw = readFileSync(
    new URL(`../../../contracts/${name}.json`, import.meta.url),
    "utf8"
  );
  expect(
    createHash("sha256").update(restoreNarrationProfileRaw(raw)).digest("hex")
  ).toBe(pins[name]);
});

it.each(Object.entries(markerPins))(
  "preserves the exact prior current-schema marker for %s",
  (name, digest) => {
    const raw = readFileSync(
      new URL(`../../../contracts/${name}`, import.meta.url),
      "utf8"
    );
    expect(JSON.parse(raw).projectSchemaVersion).toBe(46);
    expect(
      createHash("sha256").update(restoreNarrationProfileRaw(raw)).digest("hex")
    ).toBe(digest);
  }
);

it("pins the independently captured pre-profile headless lexemes", () => {
  const raw = readFileSync(
    new URL(
      "./fixtures/narration-profiles-predecessor-headless.raw.txt",
      import.meta.url
    )
  );
  expect(createHash("sha256").update(raw).digest("hex")).toBe(
    pins["headless-protocol-v1"]
  );
});

it("preserves the existing explicit in-place projection mode without changing its default copy mode", () => {
  const source = structuredClone(MCP);
  expect(removeNarrationProfileAdditions(source, "mcp-surface-v1", false)).toBe(
    source
  );
  expect(source.toolDefinitions.narration_profile_apply).toBeUndefined();
  expect(MCP.toolDefinitions.narration_profile_apply).toBeDefined();
});
