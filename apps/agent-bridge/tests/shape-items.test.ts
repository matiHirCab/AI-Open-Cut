import { expect, it } from "vitest";
import CATALOG from "../../../contracts/shape-items-v1.json";
import { headlessEditSchema, schemas } from "../src/schemas";
import { shapeGeometrySchema } from "../src/shape-items";

it("accepts canonical shape creation through standalone, batch and typed headless contracts", () => {
  for (const fixture of CATALOG.valid) {
    expect(
      headlessEditSchema.safeParse(fixture.value).success,
      fixture.id
    ).toBe(true);
    const { operation: _operation, ...fields } = fixture.value;
    expect(
      schemas.timelineAddShape.safeParse({
        ...fields,
        expectedRevision: 0,
        projectId: "project",
      }).success,
      fixture.id
    ).toBe(true);
    expect(
      schemas.timelineBatchEdit.safeParse({
        expectedRevision: 0,
        operations: [
          { ...fixture.value, resultAlias: "shape" },
          { itemId: "@shape", operation: "item_set_z_index", zIndex: 2 },
        ],
        projectId: "project",
      }).success,
      fixture.id
    ).toBe(true);
  }
});
it("preserves canonical structural validation and delegates cross-field paint semantics to core", () => {
  for (const fixture of CATALOG.invalid) {
    const coreSemanticOnly = ["no-paint", "filled-line"].includes(fixture.id);
    expect(
      headlessEditSchema.safeParse(fixture.value).success,
      fixture.id
    ).toBe(coreSemanticOnly);
    const { operation: _operation, ...fields } = fixture.value;
    expect(
      schemas.timelineAddShape.safeParse({
        ...fields,
        expectedRevision: 0,
        projectId: "project",
      }).success,
      fixture.id
    ).toBe(coreSemanticOnly);
  }
});
it("enforces numeric and collection boundaries without normalizing input", () => {
  for (const width of [
    Number.NaN,
    Number.POSITIVE_INFINITY,
    Number.NEGATIVE_INFINITY,
    0,
    -1,
    16_385,
  ]) {
    expect(
      shapeGeometrySchema.safeParse({ height: 1, type: "rectangle", width })
        .success
    ).toBe(false);
  }
  expect(
    shapeGeometrySchema.safeParse({
      height: 0.001,
      type: "rectangle",
      width: 16_384,
    }).success
  ).toBe(true);
  for (const n of [2, 3, 4096, 4097]) {
    expect(
      shapeGeometrySchema.safeParse({
        points: Array.from({ length: n }, () => ({ x: 0, y: 0 })),
        type: "polygon",
      }).success
    ).toBe(n >= 3 && n <= 4096);
  }
});
