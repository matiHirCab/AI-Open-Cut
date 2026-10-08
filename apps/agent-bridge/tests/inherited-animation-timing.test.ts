import { describe, expect, it } from "vitest";
import CONTRACT from "../../../contracts/inherited-animation-timing-v1.json";
import soundEvents from "../../../contracts/semantic-sound-events-v1.json";
import { repeaterDescriptorSchema } from "../src/repeaters";
import { headlessEditSchema, schemas } from "../src/schemas";
import { restoreAudioBusCatalogMarker } from "./fixtures/audio-buses-projection";

const instance = {
  componentId: "definition",
  durationMs: 1000,
  startMs: 0,
  timeScale: 1,
  trimStartMs: 0,
};
const repeater = {
  copies: 2,
  opacityOffset: 0,
  source: { id: "source", scope: "root" },
  transformOffset: {
    position: { unit: "pixels" as const, x: 0, y: 0 },
    rotationDeg: 0,
    scaleX: 1,
    scaleY: 1,
    skewXDeg: 0,
    skewYDeg: 0,
  },
};

describe("inherited animation timing contract", () => {
  it("matches schema 26 and accepts additive fields in standalone and batch edits", () => {
    expect(CONTRACT.projectSchemaVersion).toBe(
      soundEvents.projectSchemaVersion
    );
    expect(restoreAudioBusCatalogMarker(CONTRACT).projectSchemaVersion).toBe(
      38
    );
    expect(CONTRACT.capability).toBe("inherited_animation_timing_v1");
    for (const staggerMs of [0, 60_000]) {
      const group = {
        durationMs: 1000,
        staggerMs,
        startMs: 0,
        trackId: "overlay",
      };
      expect(
        schemas.addGroup.safeParse({
          ...group,
          expectedRevision: 0,
          projectId: "project",
        }).success
      ).toBe(true);
      expect(
        headlessEditSchema.safeParse({
          ...group,
          operation: "add_group",
          resultAlias: "parent",
        }).success
      ).toBe(true);
      expect(
        headlessEditSchema.safeParse({
          ...instance,
          operation: "add_component_instance",
          resultAlias: "child",
          staggerMs,
          trackId: "overlay",
        }).success
      ).toBe(true);
      expect(
        schemas.componentInstanceUpdate.safeParse({
          ...instance,
          expectedRevision: 0,
          itemId: "child",
          projectId: "project",
          staggerMs,
        }).success
      ).toBe(true);
      expect(
        headlessEditSchema.safeParse({
          itemId: "@parent",
          operation: "update_item",
          staggerMs,
        }).success
      ).toBe(true);
    }
    for (const timeOffsetMs of [-60_000, 0, 60_000]) {
      expect(
        repeaterDescriptorSchema.safeParse({
          ...repeater,
          timeOffsetMs,
        }).success
      ).toBe(true);
    }
    expect(repeaterDescriptorSchema.safeParse(repeater).success).toBe(true);
  });

  it("rejects malformed and out-of-bounds timing values", () => {
    for (const staggerMs of [-1, 60_001, 0.5, "1", null]) {
      expect(
        schemas.addGroup.safeParse({
          durationMs: 1000,
          expectedRevision: 0,
          projectId: "project",
          staggerMs,
          startMs: 0,
          trackId: "overlay",
        }).success
      ).toBe(false);
    }
    for (const timeOffsetMs of [-60_001, 60_001, 0.5, "1", null]) {
      expect(
        repeaterDescriptorSchema.safeParse({
          ...repeater,
          timeOffsetMs,
        }).success
      ).toBe(false);
    }
  });
});
