import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import catalog from "../../../contracts/audio-bus-dsp-v1.json";
import ownership from "../../../contracts/contract-ownership-v1.json";
import headless from "../../../contracts/headless-protocol-v1.json";
import mcp from "../../../contracts/mcp-surface-v1.json";
import {
  audioBusDspSchema,
  audioBusSchema,
  headlessEditSchema,
  schemas,
} from "../src/schemas";
import pins from "./fixtures/audio-bus-dsp-predecessor-pins.json";
import {
  audioBusDspDigest,
  removeAudioBusDspHeadlessAdditions,
  removeAudioBusDspMcpAdditions,
  removeAudioBusDspOwnershipAddition,
  restoreAudioBusDspRawHeader,
} from "./fixtures/audio-bus-dsp-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";

const hash = (value: string | Uint8Array) =>
  createHash("sha256").update(value).digest("hex");
it("preserves exact verified issue64 raw, semantic, expanded and all46 independent catalogs", () => {
  expect(pins.predecessorCommit).toBe(
    "6b5d4fa57c63603606019e02b81a3282cdbc41b0"
  );
  expect(pins.predecessorCiRun).toBe(37_754_444_838);
  expect(pins.predecessorAll11Success).toBe(true);
  for (const [path, pin] of Object.entries(pins.rawSha256)) {
    const name = path.replace("contracts/", "").replace(".json", "");
    const raw = readFileSync(
      resolve(
        import.meta.dirname,
        "fixtures",
        `audio-bus-dsp-${name}-predecessor.raw`
      )
    );
    expect(hash(raw)).toBe(pin);
    const predecessor: unknown = JSON.parse(raw.toString());
    let projected: unknown;
    if (name === "mcp-surface-v1") {
      projected = removeAudioBusDspMcpAdditions(mcp);
    } else if (name === "headless-protocol-v1") {
      projected = removeAudioBusDspHeadlessAdditions(headless);
    } else {
      projected = removeAudioBusDspOwnershipAddition(ownership);
    }
    expect(projected).toEqual(predecessor);
    if (name === "mcp-surface-v1") {
      expect(audioBusDspDigest(projected)).toBe(pins.mcpSemanticSha256);
      expect(audioBusDspDigest(expandMcpSurfaceCatalog(projected))).toBe(
        pins.mcpExpandedSha256
      );
    }
  }
  expect(Object.keys(pins.catalogRawSha256)).toHaveLength(46);
  for (const [path, pin] of Object.entries(pins.catalogRawSha256)) {
    if (Object.hasOwn(pins.rawSha256, path)) {
      continue;
    }
    expect(
      hash(
        restoreAudioBusDspRawHeader(
          readFileSync(resolve(import.meta.dirname, "../../..", path), "utf8")
        )
      )
    ).toBe(pin);
  }
});
it("shares the manually reviewed closed finite bounded normalized DSP contract", () => {
  expect(headlessEditSchema.parse(catalog.input)).toEqual(catalog.input);
  const { operation: _operation, ...input } = catalog.input;
  expect(
    schemas.audioBusSetDsp.parse({
      ...input,
      expectedRevision: 0,
      projectId: "project-1",
    })
  ).toEqual({ ...input, expectedRevision: 0, projectId: "project-1" });
  expect(
    audioBusSchema.safeParse({ dsp: null, id: "music", outputBusId: "master" })
      .success
  ).toBe(false);
  for (const invalid of [
    null,
    { ...catalog.identity, rawFilter: "volume=2" },
    { ...catalog.identity, gainDb: Number.NaN },
    { ...catalog.identity, pan: 1.01 },
    {
      ...catalog.identity,
      eq: Array.from({ length: 9 }, () => ({
        frequencyHz: 1000,
        gainDb: 0,
        q: 1,
      })),
    },
  ]) {
    expect(audioBusDspSchema.safeParse(invalid).success).toBe(false);
  }
  for (const key of Object.keys(catalog.identity)) {
    const invalid: Record<string, unknown> = { ...catalog.identity };
    Reflect.deleteProperty(invalid, key);
    expect(audioBusDspSchema.safeParse(invalid).success).toBe(false);
  }
});
it("rejects addition tampering and leaves the current42 catalog immutable", () => {
  const changed = structuredClone(mcp);
  changed.toolDefinitions.audio_bus_set_dsp.annotations.readOnlyHint = true;
  expect(() => removeAudioBusDspMcpAdditions(changed)).toThrow();
  expect(
    removeAudioBusDspMcpAdditions(mcp).$defs
      .ProjectGetStateOutputPropertiesProjectProperties.schemaVersion.const
  ).toBe(41);
  expect(
    mcp.$defs.ProjectGetStateOutputPropertiesProjectProperties.schemaVersion
      .const
  ).toBe(42);
  expect(audioBusDspDigest(expandMcpSurfaceCatalog(mcp))).toBe(
    pins.manuallyReviewedCurrentExpandedSha256
  );
});
