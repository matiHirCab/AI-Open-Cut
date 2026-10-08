import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import catalog from "../../../contracts/audio-bus-ducking-v1.json";
import ownership from "../../../contracts/contract-ownership-v1.json";
import headless from "../../../contracts/headless-protocol-v1.json";
import mcp from "../../../contracts/mcp-surface-v1.json";
import {
  audioBusDuckingSchema,
  audioBusSchema,
  headlessEditSchema,
  schemas,
} from "../src/schemas";
import pins from "./fixtures/audio-bus-ducking-predecessor-pins.json";
import {
  audioBusDuckingDigest,
  removeAudioBusDuckingHeadlessAdditions,
  removeAudioBusDuckingMcpAdditions,
  removeAudioBusDuckingOwnershipAddition,
  restoreAudioBusDuckingRawHeader,
} from "./fixtures/audio-bus-ducking-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";

const hash = (value: string | Uint8Array) =>
  createHash("sha256").update(value).digest("hex");
it("preserves exact verified issue66 raw, semantic, expanded and all47 independent catalogs", () => {
  expect(pins.predecessorCommit).toBe(
    "a3138a447f83c0c3c86f1636ab16abca27cb2308"
  );
  expect(pins.predecessorCiRun).toBe(37_796_389_148);
  expect(pins.predecessorAll11Success).toBe(true);
  for (const [path, pin] of Object.entries(pins.rawSha256)) {
    const name = path.replace("contracts/", "").replace(".json", "");
    const raw = readFileSync(
      resolve(
        import.meta.dirname,
        "fixtures",
        `audio-bus-ducking-${name}-predecessor.raw`
      )
    );
    expect(hash(raw)).toBe(pin);
    const predecessor: unknown = JSON.parse(raw.toString());
    let projected: unknown;
    if (name === "mcp-surface-v1") {
      projected = removeAudioBusDuckingMcpAdditions(mcp);
    } else if (name === "headless-protocol-v1") {
      projected = removeAudioBusDuckingHeadlessAdditions(headless);
    } else {
      projected = removeAudioBusDuckingOwnershipAddition(ownership);
    }
    expect(projected).toEqual(predecessor);
    if (name === "mcp-surface-v1") {
      expect(audioBusDuckingDigest(projected)).toBe(pins.mcpSemanticSha256);
      expect(audioBusDuckingDigest(expandMcpSurfaceCatalog(projected))).toBe(
        pins.mcpExpandedSha256
      );
    }
  }
  expect(Object.keys(pins.catalogRawSha256)).toHaveLength(47);
  for (const [path, pin] of Object.entries(pins.catalogRawSha256)) {
    if (Object.hasOwn(pins.rawSha256, path)) {
      continue;
    }
    expect(
      hash(
        restoreAudioBusDuckingRawHeader(
          readFileSync(resolve(import.meta.dirname, "../../..", path), "utf8")
        )
      )
    ).toBe(pin);
  }
});
it("shares the manually reviewed closed finite bounded normalized ducking contract", () => {
  expect(headlessEditSchema.parse(catalog.input)).toEqual(catalog.input);
  const { operation: _operation, ...input } = catalog.input;
  expect(
    schemas.audioBusSetDucking.parse({
      ...input,
      expectedRevision: 0,
      projectId: "project-1",
    })
  ).toEqual({ ...input, expectedRevision: 0, projectId: "project-1" });
  expect(
    audioBusSchema.safeParse({
      ducking: null,
      id: "music",
      outputBusId: "master",
    }).success
  ).toBe(false);
  for (const invalid of [
    null,
    { ...catalog.identity, rawFilter: "volume=2" },
    { ...catalog.identity, gain: Number.NaN },
    { ...catalog.identity, gain: -0.01 },
    { ...catalog.identity, gain: 1.01 },
    { ...catalog.identity, attackMs: -1 },
    { ...catalog.identity, attackMs: 2001 },
    { ...catalog.identity, attackMs: 0.5 },
    { ...catalog.identity, releaseMs: -1 },
    { ...catalog.identity, releaseMs: 9001 },
    { ...catalog.identity, releaseMs: 0.5 },
  ]) {
    expect(audioBusDuckingSchema.safeParse(invalid).success).toBe(false);
  }
  for (const key of Object.keys(catalog.identity)) {
    const invalid: Record<string, unknown> = { ...catalog.identity };
    Reflect.deleteProperty(invalid, key);
    expect(audioBusDuckingSchema.safeParse(invalid).success).toBe(false);
  }
});
it("rejects addition tampering and leaves the current43 catalog immutable", () => {
  const changed = structuredClone(mcp);
  changed.toolDefinitions.audio_bus_set_ducking.annotations.readOnlyHint = true;
  expect(() => removeAudioBusDuckingMcpAdditions(changed)).toThrow();
  expect(
    removeAudioBusDuckingMcpAdditions(mcp).$defs
      .ProjectGetStateOutputPropertiesProjectProperties.schemaVersion.const
  ).toBe(42);
  expect(
    mcp.$defs.ProjectGetStateOutputPropertiesProjectProperties.schemaVersion
      .const
  ).toBe(43);
  expect(audioBusDuckingDigest(expandMcpSurfaceCatalog(mcp))).toBe(
    pins.manuallyReviewedCurrentExpandedSha256
  );
});
