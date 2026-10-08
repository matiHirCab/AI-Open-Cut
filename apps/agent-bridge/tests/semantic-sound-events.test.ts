import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import ownership from "../../../contracts/contract-ownership-v1.json";
import headless from "../../../contracts/headless-protocol-v1.json";
import mcp from "../../../contracts/mcp-surface-v1.json";
import catalog from "../../../contracts/semantic-sound-events-v1.json";
import {
  editDraftSchema,
  headlessEditSchema,
  schemas,
  soundEventDefinitionSchema,
} from "../src/schemas";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";
import pins from "./fixtures/semantic-sound-events-predecessor-pins.json";
import {
  projectSoundEventMcpPredecessor,
  removeSoundEventHeadlessAdditions,
  removeSoundEventOwnershipAddition,
} from "./fixtures/semantic-sound-events-projection";

const hash = (value: Uint8Array | string) =>
  createHash("sha256").update(value).digest("hex");

it("preserves independently captured issue65 raw, expanded and semantic predecessor contracts", () => {
  expect(pins.predecessorCommit).toBe(
    "321dc3583df2d95380efb613153b0135bb32268d"
  );
  expect(pins.predecessorCiRun).toBe(37_721_029_887);
  expect(pins.predecessorAll11Success).toBe(true);
  for (const [path, expected] of Object.entries(pins.rawSha256)) {
    const name = path.replace("contracts/", "").replace(".json", "");
    const raw = readFileSync(
      resolve(
        import.meta.dirname,
        "fixtures",
        `semantic-sound-events-${name}-predecessor.raw`
      )
    );
    expect(hash(raw)).toBe(expected);
    const original: unknown = JSON.parse(raw.toString());
    if (name === "mcp-surface-v1") {
      expect(projectSoundEventMcpPredecessor(mcp)).toEqual(original);
      expect(hash(JSON.stringify(expandMcpSurfaceCatalog(original)))).toBe(
        pins.mcpExpandedSha256
      );
    } else if (name === "headless-protocol-v1") {
      expect(removeSoundEventHeadlessAdditions(headless)).toEqual(original);
    } else {
      expect(removeSoundEventOwnershipAddition(ownership)).toEqual(original);
    }
  }
  const changed = new Set(Object.keys(pins.rawSha256));
  const headers = new Set([
    "animation-channels-v1.json",
    "animation-presets-v1.json",
    "extended-visual-animation-v1.json",
    "inherited-animation-timing-v1.json",
    "motion-blur-sampling-v1.json",
    "initial-motion-preset-pack-v1.json",
    "mask-models-v1.json",
  ]);
  expect(Object.keys(pins.catalogRawSha256)).toHaveLength(44);
  for (const [path, expected] of Object.entries(pins.catalogRawSha256)) {
    if (changed.has(path)) {
      continue;
    }
    const raw = readFileSync(
      resolve(import.meta.dirname, "../../..", path),
      "utf8"
    );
    if (headers.has(path.replace("contracts/", ""))) {
      expect(raw.match(/"projectSchemaVersion": 40/g)).toHaveLength(1);
      expect(
        hash(
          raw.replace(
            '"projectSchemaVersion": 40',
            '"projectSchemaVersion": 39'
          )
        )
      ).toBe(expected);
    } else {
      expect(hash(raw)).toBe(expected);
    }
  }
});

it("keeps definition, standalone, batch and draft contracts closed and bounded", () => {
  expect(soundEventDefinitionSchema.parse(catalog.definition)).toEqual(
    catalog.definition
  );
  expect(headlessEditSchema.parse(catalog.registrationInput)).toEqual(
    catalog.registrationInput
  );
  const { operation, ...fields } = catalog.registrationInput;
  expect(operation).toBe(catalog.operation);
  const input = { expectedRevision: 0, projectId: "p", ...fields };
  expect(schemas.soundEventRegister.parse(input)).toEqual(input);
  expect(
    headlessEditSchema.parse({
      ...catalog.registrationInput,
      resultAlias: "impactAlias",
    })
  ).toMatchObject({ resultAlias: "impactAlias" });
  expect(
    schemas.timelineBatchEdit.parse({
      expectedRevision: 0,
      operations: [catalog.registrationInput],
      projectId: "p",
    }).operations
  ).toEqual([catalog.registrationInput]);
  expect(
    editDraftSchema.parse({
      baseRevision: 0,
      createdAtMs: 1,
      fontCatalog: {},
      fontSteps: [{}],
      id: "draft",
      label: null,
      operations: [catalog.registrationInput],
      projectId: "p",
      updatedAtMs: 1,
      version: 2,
    }).operations
  ).toEqual([catalog.registrationInput]);
  for (const key of Object.keys(catalog.definition)) {
    const missing = { ...catalog.definition };
    Reflect.deleteProperty(missing, key);
    expect(soundEventDefinitionSchema.safeParse(missing).success, key).toBe(
      false
    );
  }
  for (const value of [
    Number.NaN,
    Number.POSITIVE_INFINITY,
    Number.NEGATIVE_INFINITY,
    -120.001,
    24.001,
  ]) {
    expect(
      schemas.soundEventRegister.safeParse({ ...input, defaultGainDb: value })
        .success
    ).toBe(false);
  }
  for (const value of catalog.gainBoundaryInputs) {
    expect(
      schemas.soundEventRegister.safeParse({ ...input, defaultGainDb: value })
        .success
    ).toBe(true);
  }
  for (const value of catalog.invalidSeedInputs) {
    expect(
      schemas.soundEventRegister.safeParse({ ...input, variantSeed: value })
        .success
    ).toBe(false);
  }
  for (const extra of [{ expression: "bad" }, { resultAlias: "alias" }]) {
    expect(
      schemas.soundEventRegister.safeParse({ ...input, ...extra }).success
    ).toBe(false);
  }
  expect(
    soundEventDefinitionSchema.safeParse({
      ...catalog.definition,
      expression: "bad",
    }).success
  ).toBe(false);
  for (const value of ["", "1impact", "../impact"]) {
    expect(
      soundEventDefinitionSchema.safeParse({
        ...catalog.definition,
        event: value,
      }).success
    ).toBe(false);
  }
  for (const value of [[], Array.from({ length: 33 }, () => "asset")]) {
    expect(
      soundEventDefinitionSchema.safeParse({
        ...catalog.definition,
        variantAssetIds: value,
      }).success
    ).toBe(false);
  }
  expect(
    headless.status.editorCapabilities.filter(
      (value) => value === catalog.capability
    )
  ).toHaveLength(1);
  expect(mcp.tools.filter((value) => value === catalog.operation)).toHaveLength(
    1
  );
});

it("rejects unrelated, missing or malformed additions and keeps restored replacement nodes independent", () => {
  for (const mutate of [
    (value: typeof mcp) => {
      value.toolDefinitions.marker_create.annotations.readOnlyHint = true;
    },
    (value: typeof mcp) => {
      Reflect.deleteProperty(value.toolDefinitions, "sound_event_register");
    },
    (value: typeof mcp) => {
      value.$defs.ProjectGetStateOutputPropertiesProjectProperties.schemaVersion.const = 41;
    },
    (value: typeof mcp) => {
      value.tools.push("unexpected");
    },
  ]) {
    const changed = structuredClone(mcp);
    mutate(changed);
    expect(() => projectSoundEventMcpPredecessor(changed)).toThrow();
  }
  const first = projectSoundEventMcpPredecessor(mcp) as typeof mcp;
  first.$defs.ProjectGetStateOutputPropertiesProjectProperties.schemaVersion.const = 37;
  const second = projectSoundEventMcpPredecessor(mcp) as typeof mcp;
  expect(
    second.$defs.ProjectGetStateOutputPropertiesProjectProperties.schemaVersion
      .const
  ).toBe(39);
  expect(
    mcp.$defs.ProjectGetStateOutputPropertiesProjectProperties.schemaVersion
      .const
  ).toBe(40);
  const protocol = structuredClone(headless);
  protocol.requests.soundEventRegister.edit.variantSeed = 1;
  expect(() => removeSoundEventHeadlessAdditions(protocol)).toThrow();
  const owners = structuredClone(ownership);
  owners.categories.semanticSoundEventDefinitions.consumers.pop();
  expect(() => removeSoundEventOwnershipAddition(owners)).toThrow();
});
