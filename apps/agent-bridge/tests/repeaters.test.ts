import { describe, expect, it } from "vitest";
import CATALOG from "../../../contracts/repeaters-v1.json";
import { repeaterDescriptorSchema } from "../src/repeaters";
import { headlessEditSchema, schemas } from "../src/schemas";

describe("canonical repeaters", () => {
  it("accepts canonical descriptors and public operations", () => {
    for (const fixture of CATALOG.valid) {
      expect(
        repeaterDescriptorSchema.safeParse(fixture.repeater).success,
        fixture.id
      ).toBe(true);
      const fields = {
        durationMs: 1000,
        repeater: fixture.repeater,
        startMs: 0,
        trackId: "overlay",
      };
      expect(
        headlessEditSchema.safeParse({ operation: "add_repeater", ...fields })
          .success
      ).toBe(true);
      expect(
        schemas.timelineAddRepeater.safeParse({
          expectedRevision: 0,
          projectId: "project",
          ...fields,
        }).success
      ).toBe(true);
    }
  });

  it("rejects structural and numeric boundary failures", () => {
    const [fixture] = CATALOG.valid;
    if (!fixture) {
      throw new Error("Repeater fixtures are missing");
    }
    const base = structuredClone(fixture.repeater) as Record<string, any>;
    for (const [path, value] of [
      ["copies", 0],
      ["copies", 257],
      ["copies", 1.5],
      ["opacityOffset", 1.01],
      ["transformOffset.scaleX", 0],
      ["transformOffset.position.unit", "percent"],
      ["transformOffset.position.x", 1_000_001],
    ] as const) {
      const candidate = structuredClone(base);
      const parts = path.split(".");
      let target = candidate;
      for (const part of parts.slice(0, -1)) {
        target = target[part];
      }
      target[parts.at(-1) ?? ""] = value;
      expect(repeaterDescriptorSchema.safeParse(candidate).success, path).toBe(
        false
      );
    }
    expect(
      repeaterDescriptorSchema.safeParse({ ...base, extra: true }).success
    ).toBe(false);
    expect(repeaterDescriptorSchema.safeParse(null).success).toBe(false);
    expect(repeaterDescriptorSchema.safeParse([]).success).toBe(false);
    for (const invalidFixture of CATALOG.invalid) {
      let candidate: unknown = structuredClone(base);
      if ("repeater" in invalidFixture) {
        candidate = invalidFixture.repeater;
      } else if ("remove" in invalidFixture) {
        const parts = invalidFixture.remove.split(".");
        let target = candidate as Record<string, any>;
        for (const part of parts.slice(0, -1)) {
          target = target[part];
        }
        delete target[parts.at(-1) ?? ""];
      } else if ("path" in invalidFixture && "value" in invalidFixture) {
        const parts = invalidFixture.path.split(".");
        let target = candidate as Record<string, any>;
        for (const part of parts.slice(0, -1)) {
          target = target[part];
        }
        target[parts.at(-1) ?? ""] = invalidFixture.value;
      }
      expect(
        repeaterDescriptorSchema.safeParse(candidate).success,
        invalidFixture.id
      ).toBe(false);
    }
  });

  it("accepts source aliases only inside atomic batch edits", () => {
    const [fixture] = CATALOG.valid;
    if (!fixture) {
      throw new Error("Repeater fixtures are missing");
    }
    const repeater = structuredClone(fixture.repeater);
    repeater.source.id = "@source";

    expect(
      headlessEditSchema.safeParse({
        itemId: "@copies",
        operation: "update_item",
        repeater,
      }).success
    ).toBe(true);

    expect(
      headlessEditSchema.safeParse({
        durationMs: 1000,
        operation: "add_repeater",
        repeater,
        startMs: 0,
        trackId: "overlay",
      }).success
    ).toBe(true);
    expect(
      schemas.timelineAddRepeater.safeParse({
        durationMs: 1000,
        expectedRevision: 0,
        projectId: "project",
        repeater,
        startMs: 0,
        trackId: "overlay",
      }).success
    ).toBe(false);
  });
});
