import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { z } from "zod/v4";
import catalog from "../../../contracts/master-normalization-v1.json";
import mcp from "../../../contracts/mcp-surface-v1.json";
import {
  headlessEditSchema,
  masterNormalizationSchema,
  schemas,
} from "../src/schemas";
import dtos from "./fixtures/master-normalization-dtos.json";
import pins from "./fixtures/master-normalization-predecessor-pins.json";
import {
  masterNormalizationDigest,
  removeMasterNormalizationMcpAdditions,
  restoreMasterNormalizationRaw,
} from "./fixtures/master-normalization-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";

it("matches independently authored closed finite normalization DTOs and operation without aliases", () => {
  expect(z.toJSONSchema(masterNormalizationSchema)).toEqual(dtos.settings);
  expect(headlessEditSchema.parse(catalog.input)).toEqual(catalog.input);
  expect(
    schemas.audioMasterSetNormalization.parse({
      expectedRevision: 0,
      normalization: catalog.settingsExample,
      projectId: "project-1",
    })
  ).toEqual({
    expectedRevision: 0,
    normalization: catalog.settingsExample,
    projectId: "project-1",
  });
  for (const key of Object.keys(catalog.settingsExample)) {
    const invalid: Record<string, unknown> = { ...catalog.settingsExample };
    Reflect.deleteProperty(invalid, key);
    expect(masterNormalizationSchema.safeParse(invalid).success).toBe(false);
  }
  for (const enabled of [true, false]) {
    for (const [key, value] of [
      ["targetIntegratedLufs", -70.01],
      ["targetIntegratedLufs", -4.99],
      ["targetLoudnessRangeLu", 0.99],
      ["targetLoudnessRangeLu", 50.01],
      ["targetTruePeakDbtp", -9.01],
      ["targetTruePeakDbtp", 0.01],
      ["targetIntegratedLufs", Number.NaN],
      ["targetTruePeakDbtp", Number.POSITIVE_INFINITY],
    ] as const) {
      expect(
        masterNormalizationSchema.safeParse({
          ...catalog.settingsExample,
          enabled,
          [key]: value,
        }).success
      ).toBe(false);
    }
  }
  for (const normalization of [
    null,
    { ...catalog.disabledExample, rawFilter: "loudnorm" },
  ]) {
    expect(
      headlessEditSchema.safeParse({ ...catalog.input, normalization }).success
    ).toBe(false);
  }
  expect(
    headlessEditSchema.safeParse({ ...catalog.input, resultAlias: "master" })
      .success
  ).toBe(false);
});

it("preserves the exact independently captured final68 86-tool expansion and all52 raw catalogs", () => {
  expect(pins.predecessorCommit).toBe(
    "f1972d3fe8ff03c9b43519589f6283e461b63124"
  );
  expect(pins.predecessorAll11Success).toBe(true);
  expect(Object.keys(pins.catalogRawSha256)).toHaveLength(52);
  for (const [path, sha] of Object.entries(pins.catalogRawSha256)) {
    const current = readFileSync(
      resolve(import.meta.dirname, "../../..", path),
      "utf8"
    );
    expect(
      createHash("sha256")
        .update(restoreMasterNormalizationRaw(current))
        .digest("hex"),
      path
    ).toBe(sha);
  }
  const previous = removeMasterNormalizationMcpAdditions(mcp);
  expect(masterNormalizationDigest(previous)).toBe(pins.mcpSemanticSha256);
  const expanded = expandMcpSurfaceCatalog(previous);
  expect(expanded.tools).toHaveLength(86);
  expect(masterNormalizationDigest(expanded)).toBe(pins.mcpExpandedSha256);
  expect(expandMcpSurfaceCatalog(mcp).tools).toHaveLength(87);
  expect(masterNormalizationDigest(expandMcpSurfaceCatalog(mcp))).toBe(
    pins.manuallyReviewedCurrentExpandedSha256
  );
  const tampered = structuredClone(mcp);
  tampered.toolDefinitions.audio_master_set_normalization.annotations.readOnlyHint = true;
  expect(() => removeMasterNormalizationMcpAdditions(tampered)).toThrow();
});
