import { expect, it } from "vitest";
import CATALOG from "../../../contracts/procedural-grids-v1.json";
import { gridDescriptorSchema } from "../src/procedural-grids";
import { headlessEditSchema, schemas } from "../src/schemas";

it("matches canonical grid descriptor fixtures and transport schemas", () => {
  for (const fixture of CATALOG.valid) {
    expect(gridDescriptorSchema.parse(fixture.grid)).toEqual(fixture.grid);
    const fields = {
      durationMs: 1000,
      grid: fixture.grid,
      startMs: 0,
      trackId: "overlay",
    };
    expect(
      headlessEditSchema.parse({ operation: "add_grid", ...fields })
    ).toEqual({ operation: "add_grid", ...fields });
    expect(
      schemas.timelineAddGrid.parse({
        ...fields,
        expectedRevision: 0,
        projectId: "project",
      }).grid
    ).toEqual(fixture.grid);
    expect(
      schemas.timelineBatchEdit.safeParse({
        expectedRevision: 0,
        operations: [
          { ...fields, operation: "add_grid", resultAlias: "grid" },
          { grid: fixture.grid, itemId: "@grid", operation: "update_item" },
        ],
        projectId: "project",
      }).success
    ).toBe(true);
  }
  for (const fixture of CATALOG.invalid) {
    expect(
      gridDescriptorSchema.safeParse(fixture.grid).success,
      fixture.id
    ).toBe(false);
  }
});

it("rejects null updates, non-finite values and exact mark overflow", () => {
  expect(
    headlessEditSchema.safeParse({
      grid: null,
      itemId: "grid",
      operation: "update_item",
    }).success
  ).toBe(false);
  const [, , fixture] = CATALOG.valid;
  if (!fixture) {
    throw new Error("Missing dot fixture");
  }
  const base = fixture.grid;
  const pattern = { ...base.pattern, radius: 0.25, spacingX: 1, spacingY: 1 };
  expect(
    gridDescriptorSchema.safeParse({ height: 63, pattern, width: 63 }).success
  ).toBe(true);
  for (const width of [64, Number.NaN, Number.POSITIVE_INFINITY, 0, -1]) {
    expect(
      gridDescriptorSchema.safeParse({ height: 63, pattern, width }).success
    ).toBe(false);
  }
  expect(CATALOG.schemaVersion).toBe(16);
  expect(CATALOG.capabilities).toEqual(["grid_items", "grid_rendering"]);
});
