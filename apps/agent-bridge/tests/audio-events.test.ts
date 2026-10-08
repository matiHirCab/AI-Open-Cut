import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import ownership from "../../../contracts/contract-ownership-v1.json";
import headless from "../../../contracts/headless-protocol-v1.json";
import mcp from "../../../contracts/mcp-surface-v1.json";
import catalog from "../../../contracts/timeline-audio-events-v1.json";
import {
  audioEventItemSchema,
  headlessEditSchema,
  schemas,
} from "../src/schemas";
import pins from "./fixtures/audio-events-predecessor-pins.json";
import {
  projectAudioEventMcpPredecessor,
  removeAudioEventHeadlessAdditions,
  removeAudioEventOwnershipAddition,
} from "./fixtures/audio-events-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";

const hash = (value: string | Uint8Array) =>
  createHash("sha256").update(value).digest("hex");
it("preserves independent verified issue63 raw, expanded, semantic and frozen catalog proofs", () => {
  expect(pins.predecessorCommit).toBe(
    "337329198fcd40173a1b401e39ae7782610cbb94"
  );
  expect(pins.predecessorCiRun).toBe(37_735_454_429);
  expect(pins.predecessorAll11Success).toBe(true);
  for (const [path, pin] of Object.entries(pins.rawSha256)) {
    const name = path.replace("contracts/", "").replace(".json", "");
    const raw = readFileSync(
      resolve(
        import.meta.dirname,
        "fixtures",
        `audio-events-${name}-predecessor.raw`
      )
    );
    expect(hash(raw)).toBe(pin);
    const old: unknown = JSON.parse(raw.toString());
    let projected: unknown;
    if (name === "mcp-surface-v1") {
      projected = projectAudioEventMcpPredecessor(mcp);
    } else if (name === "headless-protocol-v1") {
      projected = removeAudioEventHeadlessAdditions(headless);
    } else {
      projected = removeAudioEventOwnershipAddition(ownership);
    }
    expect(projected).toEqual(old);
    if (name === "mcp-surface-v1") {
      expect(hash(JSON.stringify(expandMcpSurfaceCatalog(old)))).toBe(
        pins.mcpExpandedSha256
      );
    }
  }
  const headers = new Set([
    "animation-channels-v1.json",
    "animation-presets-v1.json",
    "extended-visual-animation-v1.json",
    "inherited-animation-timing-v1.json",
    "motion-blur-sampling-v1.json",
    "initial-motion-preset-pack-v1.json",
    "mask-models-v1.json",
  ]);
  expect(Object.keys(pins.catalogRawSha256)).toHaveLength(45);
  for (const [path, pin] of Object.entries(pins.catalogRawSha256)) {
    if (Object.hasOwn(pins.rawSha256, path)) {
      continue;
    }
    const raw = readFileSync(
      resolve(import.meta.dirname, "../../..", path),
      "utf8"
    );
    if (headers.has(path.replace("contracts/", ""))) {
      expect(raw.match(/"projectSchemaVersion": 41/g)).toHaveLength(1);
      expect(
        hash(
          raw.replace(
            '"projectSchemaVersion": 41',
            '"projectSchemaVersion": 40'
          )
        )
      ).toBe(pin);
    } else {
      expect(hash(raw)).toBe(pin);
    }
  }
});
it("shares closed placement and provenance contracts without weakening old fields", () => {
  const { input } = catalog;
  expect(headlessEditSchema.parse(input)).toEqual(input);
  const publicInput = { ...input, expectedRevision: 0, projectId: "project-1" };
  Reflect.deleteProperty(publicInput, "operation");
  expect(schemas.timelineAddAudioEvent.parse(publicInput)).toEqual(publicInput);
  expect(audioEventItemSchema.parse(catalog.provenance)).toEqual(
    catalog.provenance
  );
  for (const at of catalog.at) {
    expect(headlessEditSchema.safeParse({ ...input, at }).success).toBe(true);
  }
  for (const field of ["at", "gainDb", "variantSeed", "durationMs"]) {
    expect(
      headlessEditSchema.safeParse({ ...input, [field]: null }).success
    ).toBe(false);
  }
  expect(
    headlessEditSchema.safeParse({ ...input, at: { ...input.at, extra: 1 } })
      .success
  ).toBe(false);
  expect(headlessEditSchema.safeParse({ ...input, extra: 1 }).success).toBe(
    false
  );
  expect(
    audioEventItemSchema.safeParse({ ...catalog.provenance, extra: 1 }).success
  ).toBe(false);
  expect(
    audioEventItemSchema.safeParse({
      ...catalog.provenance,
      gainDb: Number.POSITIVE_INFINITY,
    }).success
  ).toBe(false);
});
it("rejects unrelated predecessor and approved addition mutations", () => {
  for (const mutate of [
    (v: typeof mcp) => {
      v.toolDefinitions.timeline_add_audio_event.annotations.readOnlyHint = true;
    },
    (v: typeof mcp) => {
      v.toolDefinitions.sound_event_register.annotations.readOnlyHint = true;
    },
    (v: typeof mcp) => {
      v.tools.push("unrelated");
    },
  ]) {
    const changed = structuredClone(mcp);
    mutate(changed);
    expect(() => projectAudioEventMcpPredecessor(changed)).toThrow();
  }
  const changed = structuredClone(headless);
  changed.requests.timelineAddAudioEvent.edit.event = "different";
  expect(() => removeAudioEventHeadlessAdditions(changed)).toThrow();
});
