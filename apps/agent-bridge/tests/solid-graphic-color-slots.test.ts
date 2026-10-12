import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import catalog from "../../../contracts/solid-graphic-color-slots-v1.json";
import {
  headlessEditSchema,
  slotValueSchema,
  templateSlotSchema,
} from "../src/schemas";
import pins from "./fixtures/solid-graphic-color-slots-predecessor-pins.json";
import { restoreSolidGraphicRaw } from "./fixtures/solid-graphic-color-slots-projection";
import { assertSolidGraphicPixels } from "./solid-graphic-color-slots-workflow";

it("accepts the canonical solid binding matrix without extending color values", () => {
  expect(catalog.properties).toEqual([
    "shape.fill.color",
    "shape.stroke.color",
    "grid.dot.paint.color",
  ]);
  expect(headlessEditSchema.safeParse(catalog.definition).success).toBe(true);
  for (const slot of catalog.definition.slots) {
    expect(templateSlotSchema.parse(slot)).toEqual(slot);
    for (const value of catalog.invalidValues) {
      if (value?.type === "number") {
        // Structural parsing accepts another existing slot kind; core owns
        // kind/property compatibility and rejects this color-slot candidate.
        expect(slotValueSchema.parse(value)).toEqual(value);
        continue;
      }
      expect(slotValueSchema.safeParse(value).success).toBe(false);
      expect(
        templateSlotSchema.safeParse({ ...slot, defaultValue: value }).success
      ).toBe(false);
    }
    for (const property of [
      "shape.fill",
      "svg.paint.color",
      "grid.paint.color",
      "shape.fill.color.r",
    ]) {
      expect(
        templateSlotSchema.safeParse({
          ...slot,
          binding: { ...slot.binding, property },
        }).success
      ).toBe(false);
    }
    expect(
      templateSlotSchema.safeParse({
        ...slot,
        binding: { ...slot.binding, scope: "root" },
      }).success
    ).toBe(false);
  }
});

it("restores exactly captured predecessors while preserving unrelated contract drift", () => {
  for (const [path, expected] of Object.entries(pins.rawSha256)) {
    const raw = readFileSync(
      new URL(`../../../${path}`, import.meta.url),
      "utf8"
    );
    const restored = restoreSolidGraphicRaw(raw);
    expect(createHash("sha256").update(restored).digest("hex"), path).toBe(
      expected
    );
    expect(restoreSolidGraphicRaw(restored), path).toBe(restored);
    const drift = { ...JSON.parse(raw), unauthorized: true };
    expect(() => restoreSolidGraphicRaw(JSON.stringify(drift)), path).toThrow(
      "predecessor contract drift"
    );
  }
});

it("rejects blank and wrong-size native evidence independently of fixture data", () => {
  expect(() => assertSolidGraphicPixels(Buffer.alloc(320 * 180 * 3))).toThrow();
  expect(() =>
    assertSolidGraphicPixels(Buffer.alloc(320 * 180 * 3), true)
  ).toThrow();
  expect(() => assertSolidGraphicPixels(Buffer.alloc(1))).toThrow();
});
