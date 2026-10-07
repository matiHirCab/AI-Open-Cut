const contractTestsSuffix =
  /tests\/desktop-compositing\.test\.ts tests\/masked-hero-reveal\.test\.ts$/;

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import type { ZodType } from "zod/v4";
import ownership from "../../../contracts/contract-ownership-v1.json";
import catalog from "../../../contracts/masked-hero-reveal-v1.json";
import surface from "../../../contracts/mcp-surface-v1.json";
import {
  headlessEditSchema,
  schemas,
  visualEffectSchema,
} from "../src/schemas";
import { stereoRms } from "./masked-hero-reveal-oracle";

const root = resolve(import.meta.dirname, "../../..");
const digest = (bytes: Uint8Array) =>
  createHash("sha256").update(bytes).digest("hex");
it("pins the complete approved hero recipe and immutable mathematical plates and stereo source", () => {
  expect(
    digest(
      Buffer.from(
        readFileSync(
          resolve(root, "contracts/masked-hero-reveal-v1.json"),
          "utf8"
        ).replace('"projectSchemaVersion": 38', '"projectSchemaVersion": 37')
      )
    )
  ).toBe("a5756dd3ebf95255c2fe72e7711c7e713f0b5683f4f8c2eb2cfe41fb60079541");
  expect(catalog.projectSchemaVersion).toBe(38);
  expect(catalog.headlessProtocolVersion).toBe(1);
  expect(surface.tools).toHaveLength(78);
  expect(catalog.operationTranscript.aliasBatch).toHaveLength(14);
  for (const [name, hash] of Object.entries({
    "reference-audio-provenance.json":
      "4c820c658261701b24bab9c6b68bcbd3143fe69e9428eb04b5d0c5be5d88d4de",
    "reference-conversion-provenance.json":
      "75084162b91f2ae1538ad3c62bef5f28ece0412414fb456912ef85cf4f054dff",
    "reference-film-provenance.json":
      "d1bc6781b6dbcdc84324f28a34f827b88d3987ef1c2d660189fdeed83de7daa2",
  })) {
    expect(
      digest(readFileSync(resolve(root, catalog.referenceDirectory, name)))
    ).toBe(hash);
  }
  for (const plate of catalog.plates.records) {
    const bytes = readFileSync(
      resolve(root, catalog.referenceDirectory, plate.path)
    );
    expect(bytes.length).toBe(64 * 64 * 4);
    expect(digest(bytes)).toBe(plate.sha256);
    const changed = Buffer.from(bytes);
    changed[0] = ((changed[0] ?? 0) + 1) % 256;
    expect(digest(changed)).not.toBe(plate.sha256);
  }
  const wav = readFileSync(
    resolve(root, catalog.referenceDirectory, "source.wav")
  );
  expect(digest(wav)).toBe(catalog.audio.sourceSha256);
  expect(wav.length).toBe(44 + 38_400 * 4);
  expect(wav.readUInt16LE(22)).toBe(2);
  expect(wav.readUInt32LE(24)).toBe(48_000);
  expect(wav.readUInt16LE(34)).toBe(16);
  const independent = spawnSync(
    process.platform === "win32" ? "python" : "python3",
    [resolve(root, catalog.oraclePath)]
  );
  expect(independent.status, independent.stderr.toString()).toBe(0);
  expect(independent.stdout.toString()).toContain(
    "16 complete RGBA plates, 8 counterfactual witnesses and 38400 stereo PCM frames"
  );
  const destination = mkdtempSync(resolve(tmpdir(), "hero-oracle-refusal-"));
  try {
    const rejected = spawnSync(
      process.platform === "win32" ? "python" : "python3",
      [resolve(root, catalog.oraclePath), "--output", destination]
    );
    expect(rejected.status).not.toBe(0);
    expect(rejected.stderr.toString()).toContain("FileExistsError");
  } finally {
    rmSync(destination, { force: true, recursive: true });
  }
});
it("accepts every exact typed recipe field in both alias batch and mapped standalone MCP inputs", () => {
  const operations: unknown = JSON.parse(
    JSON.stringify(catalog.operationTranscript.aliasBatch)
      .replaceAll("{{overlayTrackId}}", "overlay-track")
      .replaceAll("{{audioTrackId}}", "audio-track")
      .replaceAll("{{audioAssetId}}", "audio-asset")
  );
  const parsed = schemas.timelineBatchEdit.parse({
    expectedRevision: 1,
    operations,
    projectId: "p",
  });
  const mappings: Record<string, ZodType> = {
    add_group: schemas.addGroup,
    add_media: schemas.timelineAddMedia,
    add_shape: schemas.timelineAddShape,
    item_set_parent: schemas.itemSetParent,
    item_set_z_index: schemas.itemSetZIndex,
    set_animation_channels: schemas.timelineSetAnimationChannels,
    update_item: schemas.timelineUpdateItem,
  };
  for (const record of parsed.operations) {
    const edit = headlessEditSchema.parse(
      Object.fromEntries(
        Object.entries(record).filter(([key]) => key !== "resultAlias")
      )
    );
    const { operation, ...input } = edit;
    const schema = mappings[operation];
    if (!schema) {
      throw new Error(`Unmapped canonical operation ${operation}`);
    }
    schema.parse({ expectedRevision: 1, projectId: "p", ...input });
  }
  const { hero } = catalog.roles;
  expect(hero.masks[0]?.transform.position).toEqual({
    unit: "pixels",
    x: 1,
    y: 1,
  });
  expect(hero.animationChannels[0]?.keyframes.at(-1)?.timeMs).toBe(799);
  for (const effect of hero.effects) {
    visualEffectSchema.parse(effect);
  }
  expect(
    schemas.timelineUpdateItem.safeParse({
      expectedRevision: 1,
      itemId: "hero",
      projectId: "p",
      transform2d: { ...hero.transform2d, scaleX: Number.NaN },
    }).success
  ).toBe(false);
  expect(
    schemas.timelineUpdateItem.safeParse({
      clip: { type: "unknown" },
      expectedRevision: 1,
      itemId: "hero",
      projectId: "p",
    }).success
  ).toBe(false);
});
const requiredConsumers = [
  "contracts/fixtures/masked-hero-reveal-v1/",
  "scripts/create-masked-hero-reveal-references.py",
  "crates/editor-core/tests/support/masked_hero_reveal.rs",
  "crates/editor-core/tests/support/masked_hero_media.rs",
  "crates/editor-core/tests/support/masked_hero_authoring.rs",
  "crates/editor-core/tests/masked_hero_reveal_models.rs",
  "crates/editor-core/tests/masked_hero_reveal_native.rs",
  "apps/headless/tests/masked_hero_reveal_native.rs",
  "apps/agent-bridge/tests/masked-hero-reveal.test.ts",
  "apps/agent-bridge/tests/masked-hero-reveal-native.test.ts",
  "apps/agent-bridge/tests/masked-hero-reveal-oracle.ts",
  "apps/agent-bridge/package.json",
  "scripts/validate-ci-gates.ts",
  "scripts/validate-ci-gates.test.ts",
  ".github/workflows/bun-ci.yml",
  "docs/masked-hero-reveal.md",
];
function validateHeroOwnership(value: {
  canonical: string;
  consumers: readonly string[];
}) {
  if (value.canonical !== "contracts/masked-hero-reveal-v1.json") {
    throw new Error("canonical hero owner changed");
  }
  if (new Set(value.consumers).size !== value.consumers.length) {
    throw new Error("duplicate hero consumer");
  }
  for (const path of requiredConsumers) {
    if (!value.consumers.includes(path)) {
      throw new Error(`required hero consumer omitted: ${path}`);
    }
    if (!existsSync(resolve(root, path))) {
      throw new Error(`required hero consumer missing: ${path}`);
    }
  }
}
it("rejects each omitted or substituted governed consumer through the actual contracts gate", () => {
  const governed = ownership.categories.maskedHeroReveal;
  expect(() => validateHeroOwnership(governed)).not.toThrow();
  for (const path of requiredConsumers) {
    expect(() =>
      validateHeroOwnership({
        ...governed,
        consumers: governed.consumers.filter((candidate) => candidate !== path),
      })
    ).toThrow("required hero consumer omitted");
    expect(() =>
      validateHeroOwnership({
        ...governed,
        consumers: governed.consumers.map((candidate) =>
          candidate === path ? "substituted-consumer" : candidate
        ),
      })
    ).toThrow("required hero consumer omitted");
  }
  const packageJson = JSON.parse(
    readFileSync(resolve(root, "apps/agent-bridge/package.json"), "utf8")
  ) as { scripts: { "contracts:check": string } };
  expect(packageJson.scripts["contracts:check"]).toContain(
    "--test font_resolution --test masked_hero_reveal_models --test speech_alignment &&"
  );
  expect(packageJson.scripts["contracts:check"]).toMatch(contractTestsSuffix);
});

it("rejects stereo nonfinite samples, excessive appended tails and insufficient overlap", () => {
  const valid = Buffer.alloc(9600 * 8);
  for (let n = 0; n < 9600; n += 1) {
    for (const [channel, hz] of [437, 659].entries()) {
      valid.writeFloatLE(
        0.1 * Math.sin((2 * Math.PI * hz * n) / 48_000),
        n * 8 + channel * 4
      );
    }
  }
  expect(stereoRms(valid, valid)).toEqual([0, 0, 0]);
  expect(() =>
    stereoRms(Buffer.concat([valid, Buffer.alloc(4801 * 8)]), valid)
  ).toThrow();
  const nonfinite = Buffer.from(valid);
  nonfinite.writeFloatLE(Number.NaN);
  expect(() => stereoRms(nonfinite, valid)).toThrow();
  expect(() =>
    stereoRms(valid.subarray(0, 1000 * 8), valid.subarray(0, 1000 * 8))
  ).toThrow();
});
