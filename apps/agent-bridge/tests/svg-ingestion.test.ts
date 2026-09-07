import { expect, it } from "vitest";
import CATALOG from "../../../contracts/svg-ingestion-v1.json";
import { headlessEditSchema, schemas } from "../src/schemas";
import { svgDocumentSchema } from "../src/svg-ingestion";

it("carries every SVG fixture unchanged to the canonical core validator", () => {
  for (const fixture of [...CATALOG.valid, ...CATALOG.invalid]) {
    const fields = {
      durationMs: 1000,
      startMs: 0,
      svg: fixture.svg,
      trackId: "overlay",
    };
    const operation = { ...fields, operation: "add_svg" };
    expect(headlessEditSchema.parse(operation)).toEqual(operation);
    expect(
      schemas.timelineAddSvg.parse({
        ...fields,
        expectedRevision: 0,
        projectId: "project",
      }).svg
    ).toBe(fixture.svg);
    expect(
      schemas.timelineBatchEdit.safeParse({
        expectedRevision: 0,
        operations: [
          { ...operation, resultAlias: "icon" },
          { itemId: "@icon", operation: "item_set_z_index", zIndex: 2 },
        ],
        projectId: "project",
      }).success
    ).toBe(true);
  }
});
it("rejects malformed SVG transport and normalized records", () => {
  const fields = {
    durationMs: 1000,
    expectedRevision: 0,
    projectId: "project",
    startMs: 0,
    svg: "<svg/>",
    trackId: "overlay",
  };
  for (const svg of [null, 5, {}, []]) {
    expect(schemas.timelineAddSvg.safeParse({ ...fields, svg }).success).toBe(
      false
    );
  }
  expect(
    schemas.timelineAddSvg.safeParse({ ...fields, sourcePath: "outside.svg" })
      .success
  ).toBe(false);
  const document = {
    height: 20,
    shapes: [],
    version: 1,
    viewBox: [0, 0, 40, 20],
    width: 40,
  };
  expect(svgDocumentSchema.parse(document)).toEqual(document);
  for (const value of [
    { ...document, version: 2 },
    { ...document, width: Number.NaN },
    { ...document, source: "<svg/>" },
    Object.values(document),
  ]) {
    expect(svgDocumentSchema.safeParse(value).success).toBe(false);
  }
});
