import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import catalog from "../../../contracts/audio-buses-v1.json";
import ownership from "../../../contracts/contract-ownership-v1.json";
import headless from "../../../contracts/headless-protocol-v1.json";
import mcp from "../../../contracts/mcp-surface-v1.json";
import {
  audioBusSchema,
  editDraftSchema,
  headlessEditSchema,
  schemas,
} from "../src/schemas";
import pins from "./fixtures/audio-buses-predecessor-pins.json";
import {
  projectAudioBusMcpPredecessor,
  removeAudioBusHeadlessAdditions,
  removeAudioBusOwnershipAddition,
} from "./fixtures/audio-buses-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";

import { projectSoundEventMcpPredecessor } from "./fixtures/semantic-sound-events-projection";

const hash = (value: Uint8Array | string) =>
  createHash("sha256").update(value).digest("hex");

it("copies restored schema records so older projections cannot mutate independent pins", () => {
  const first = projectAudioBusMcpPredecessor(mcp) as typeof mcp;
  first.$defs.ProjectGetStateOutputPropertiesProjectProperties.schemaVersion.const = 37;
  const second = projectAudioBusMcpPredecessor(mcp) as typeof mcp;
  expect(
    second.$defs.ProjectGetStateOutputPropertiesProjectProperties.schemaVersion
      .const
  ).toBe(38);
  expect(
    (projectSoundEventMcpPredecessor(mcp) as typeof mcp).$defs
      .ProjectGetStateOutputPropertiesProjectProperties.schemaVersion.const
  ).toBe(39);
});

it("keeps independently pinned issue62 raw and semantic contracts plus all historical catalogs", () => {
  expect(pins.predecessorCommit).toBe(
    "d8168ad6631d834891df9303485e1d8c1b9b1116"
  );
  expect(pins.predecessorCiRun).toBe(37_706_235_074);
  expect(pins.predecessorAll11Success).toBe(true);
  for (const [path, expected] of Object.entries(pins.rawSha256)) {
    const name = path.replace("contracts/", "").replace(".json", "");
    const raw = readFileSync(
      resolve(
        import.meta.dirname,
        "fixtures",
        `audio-buses-${name}-predecessor.raw`
      )
    );
    expect(hash(raw)).toBe(expected);
    const original: unknown = JSON.parse(raw.toString());
    if (name === "mcp-surface-v1") {
      expect(projectAudioBusMcpPredecessor(mcp)).toEqual(original);
      expect(
        hash(
          JSON.stringify(expandMcpSurfaceCatalog(JSON.parse(raw.toString())))
        )
      ).toBe(pins.mcpExpandedSha256);
    } else if (name === "headless-protocol-v1") {
      expect(removeAudioBusHeadlessAdditions(headless)).toEqual(original);
    } else {
      expect(removeAudioBusOwnershipAddition(ownership)).toEqual(original);
    }
  }
  const changedSurfaces = new Set([
    "contracts/mcp-surface-v1.json",
    "contracts/headless-protocol-v1.json",
    "contracts/contract-ownership-v1.json",
  ]);
  const currentHeaders = new Set([
    "animation-channels-v1.json",
    "animation-presets-v1.json",
    "extended-visual-animation-v1.json",
    "inherited-animation-timing-v1.json",
    "motion-blur-sampling-v1.json",
    "initial-motion-preset-pack-v1.json",
    "mask-models-v1.json",
  ]);
  expect(Object.keys(pins.catalogRawSha256)).toHaveLength(43);
  for (const [path, expected] of Object.entries(pins.catalogRawSha256)) {
    if (changedSurfaces.has(path)) {
      continue;
    }
    const currentRaw = readFileSync(
      resolve(import.meta.dirname, "../../..", path),
      "utf8"
    );
    const raw = currentHeaders.has(path.replace("contracts/", ""))
      ? currentRaw.replace(
          '"projectSchemaVersion": 40',
          '"projectSchemaVersion": 39'
        )
      : currentRaw;
    if (currentHeaders.has(path.replace("contracts/", ""))) {
      expect(raw.match(/"projectSchemaVersion": 39/g)).toHaveLength(1);
      expect(
        hash(
          raw.replace(
            '"projectSchemaVersion": 39',
            '"projectSchemaVersion": 38'
          )
        )
      ).toBe(expected);
    } else {
      expect(hash(raw)).toBe(expected);
    }
  }
});

it("keeps routing inputs closed and exact across standalone, batches and draft outputs", () => {
  expect(catalog.busIds).toEqual(["voiceover", "music", "sfx", "master"]);
  expect(catalog.maxBuses).toBe(4);
  expect(catalog.maxRouteNodes).toBe(4);
  for (const bus of catalog.defaultBuses) {
    expect(audioBusSchema.parse(bus)).toEqual(bus);
  }
  expect(audioBusSchema.safeParse({ id: "master" }).success).toBe(false);
  expect(
    audioBusSchema.safeParse({
      expression: "bad",
      id: "master",
      outputBusId: null,
    }).success
  ).toBe(false);
  const busEdit = headless.requests.audioBusSetRoute.edit;
  const trackEdit = headless.requests.audioTrackRoute.edit;
  for (const edit of [busEdit, trackEdit, { ...trackEdit, busId: null }]) {
    expect(headlessEditSchema.parse(edit)).toEqual(edit);
    expect(
      schemas.timelineBatchEdit.parse({
        expectedRevision: 1,
        operations: [edit],
        projectId: "p",
      }).operations
    ).toEqual([edit]);
    expect(
      editDraftSchema.parse({
        baseRevision: 1,
        createdAtMs: 1,
        fontCatalog: {},
        fontSteps: [{}],
        id: "draft",
        label: null,
        operations: [edit],
        projectId: "p",
        updatedAtMs: 1,
        version: 2,
      }).operations
    ).toEqual([edit]);
    expect(
      headlessEditSchema.safeParse({ ...edit, expression: "bad" }).success
    ).toBe(false);
    expect(
      headlessEditSchema.safeParse({ ...edit, resultAlias: "unused" }).success
    ).toBe(false);
  }
  const { operation: busOperation, ...busInput } = busEdit;
  const { operation: trackOperation, ...trackInput } = trackEdit;
  expect(busOperation).toBe(catalog.operations[0]);
  expect(trackOperation).toBe(catalog.operations[1]);
  expect(
    schemas.audioBusSetRoute.parse({
      expectedRevision: 1,
      projectId: "p",
      ...busInput,
    })
  ).toMatchObject(busInput);
  expect(
    schemas.audioTrackRoute.parse({
      expectedRevision: 1,
      projectId: "p",
      ...trackInput,
    })
  ).toMatchObject(trackInput);
  const { busId: discarded, ...missing } = trackEdit;
  expect(discarded).toBe("music");
  expect(headlessEditSchema.safeParse(missing).success).toBe(false);
  expect(
    headlessEditSchema.safeParse({ ...busEdit, outputBusId: null }).success
  ).toBe(false);
  expect(
    headless.status.editorCapabilities.filter((v) => v === catalog.capability)
  ).toHaveLength(1);
});

it("rejects unrelated tool schema, field, version, annotation and predecessor drift", () => {
  for (const mutate of [
    (value: typeof mcp) => {
      value.toolDefinitions.audio_bus_set_route.annotations.readOnlyHint = true;
    },
    (value: typeof mcp) => {
      value.toolDefinitions.marker_create.annotations.readOnlyHint = true;
    },
    (value: typeof mcp) => {
      Reflect.deleteProperty(value.toolDefinitions, "audio_track_route");
    },
    (value: typeof mcp) => {
      value.tools.push("unexpected");
    },
    (value: typeof mcp) => {
      const properties =
        value.$defs.ProjectGetStateOutputPropertiesProjectProperties;
      properties.schemaVersion.const = 41;
    },
  ]) {
    const changed = structuredClone(mcp);
    mutate(changed);
    expect(() => projectAudioBusMcpPredecessor(changed)).toThrow();
  }
  const badProtocol = structuredClone(headless);
  badProtocol.requests.audioTrackRoute.edit.busId = "sfx";
  expect(() => removeAudioBusHeadlessAdditions(badProtocol)).toThrow();
  const badOwnership = structuredClone(ownership);
  badOwnership.categories.audioBuses.consumers.pop();
  expect(() => removeAudioBusOwnershipAddition(badOwnership)).toThrow();
});
