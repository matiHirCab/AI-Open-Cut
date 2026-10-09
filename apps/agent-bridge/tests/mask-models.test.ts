import { describe, expect, it } from "vitest";
import contract from "../../../contracts/mask-models-v1.json";
import masterNormalization from "../../../contracts/master-normalization-v1.json";
import {
  headlessEditSchema,
  maskSchema,
  maskStackSchema,
  schemas,
  timelineItemSchema,
} from "../src/schemas";
import { restoreAudioBusCatalogMarker } from "./fixtures/audio-buses-projection";

const mask = () => maskSchema.parse(contract.cases[0]?.value);
const update = (masks: unknown) => ({
  itemId: "leaf",
  masks,
  operation: "update_item",
});

describe("mask model structural contracts", () => {
  it("preserves every canonical field and rejects every invalid record", () => {
    for (const fixture of contract.cases) {
      const parsed = maskSchema.safeParse(fixture.value);
      expect(parsed.success, fixture.name).toBe(fixture.accepted);
      if (parsed.success) {
        expect(parsed.data).toEqual(fixture.value);
      }
    }
    expect(contract.status).toBe("authoring_with_active_rendering_contract");
    expect(contract.projectSchemaVersion).toBe(
      masterNormalization.projectSchemaVersion
    );
    expect(restoreAudioBusCatalogMarker(contract).projectSchemaVersion).toBe(
      38
    );
  });

  it("rejects nonfinite coverage controls and nested canonical inputs", () => {
    for (const value of [
      Number.NaN,
      Number.POSITIVE_INFINITY,
      Number.NEGATIVE_INFINITY,
    ]) {
      for (const field of ["featherPx", "expansionPx"] as const) {
        expect(
          maskSchema.safeParse({ ...mask(), [field]: value }).success
        ).toBe(false);
      }
      const invalid = mask();
      invalid.transform.opacity = value;
      expect(maskSchema.safeParse(invalid).success).toBe(false);
      invalid.transform.opacity = 1;
      invalid.source.path.commands = [
        { to: { x: value, y: 0 }, type: "moveTo" },
      ];
      expect(maskSchema.safeParse(invalid).success).toBe(false);
    }
  });

  it("preserves ordered replacement, omission and explicit clearing in both edit schemas", () => {
    const ordered = contract.stackCases[1]?.value;
    expect(headlessEditSchema.parse(update(ordered))).toEqual(update(ordered));
    expect(headlessEditSchema.parse(update([]))).toEqual(update([]));
    expect(
      headlessEditSchema.parse({
        color: "#123456",
        itemId: "leaf",
        operation: "update_item",
      })
    ).not.toHaveProperty("masks");
    expect(headlessEditSchema.safeParse(update(null)).success).toBe(false);
    const input = { expectedRevision: 0, itemId: "leaf", projectId: "p" };
    expect(
      schemas.timelineUpdateItem.parse({ ...input, masks: ordered })
    ).toEqual({ ...input, masks: ordered });
    expect(schemas.timelineUpdateItem.parse({ ...input, masks: [] })).toEqual({
      ...input,
      masks: [],
    });
    expect(
      schemas.timelineUpdateItem.safeParse({ ...input, masks: null }).success
    ).toBe(false);
  });

  it("enforces structural per-item limits while core owns scoped identity and aggregate budgets", () => {
    const full = Array.from({ length: 16 }, (_, index) => ({
      ...mask(),
      id: `m${index}`,
    }));
    expect(maskStackSchema.parse(full)).toEqual(full);
    expect(maskStackSchema.safeParse([...full, mask()]).success).toBe(false);
    // Transport does not infer item ownership or duplicate domain identity from an update target.
    expect(maskStackSchema.safeParse([mask(), mask()]).success).toBe(true);
    const common = {
      color: "#ffffff",
      durationMs: 1000,
      hidden: false,
      id: "leaf",
      keyframes: [],
      stackOrder: 0,
      startMs: 0,
      transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
      type: "solid_color",
      zIndex: 0,
    };
    expect(timelineItemSchema.parse({ ...common, masks: full })).toEqual({
      ...common,
      masks: full,
    });
    const { color: _color, keyframes: _keyframes, ...groupCommon } = common;
    expect(
      timelineItemSchema.safeParse({ ...groupCommon, masks: [], type: "group" })
        .success
    ).toBe(true);
    expect(
      timelineItemSchema.safeParse({
        ...groupCommon,
        masks: [mask()],
        type: "group",
      }).success
    ).toBe(false);
  });
});
