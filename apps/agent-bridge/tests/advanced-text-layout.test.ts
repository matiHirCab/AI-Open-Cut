import { describe, expect, it } from "vitest";
import CATALOG from "../../../contracts/advanced-text-layout-v1.json";
import { artifactSchema, textStyleSchema } from "../src/schemas";

describe("advanced text layout contract", () => {
  it("accepts canonical boundaries and rejects malformed structure", () => {
    for (const fixture of CATALOG.valid) {
      expect(
        textStyleSchema.safeParse({ layout: fixture.layout }).success,
        fixture.id
      ).toBe(true);
    }
    for (const fixture of CATALOG.invalid) {
      expect(
        textStyleSchema.safeParse({ layout: fixture.layout }).success,
        fixture.id
      ).toBe(false);
    }
    for (const value of [
      Number.NaN,
      Number.POSITIVE_INFINITY,
      Number.NEGATIVE_INFINITY,
    ]) {
      expect(
        textStyleSchema.safeParse({ layout: { trackingPx: value } }).success
      ).toBe(false);
    }
    expect(textStyleSchema.parse({}).layout).toBeUndefined();
  });
  it("preserves additive fit diagnostics without changing legacy artifacts", () => {
    const artifact = {
      mimeType: "image/png",
      relativePath: "previews/frame.png",
      sizeBytes: 1,
      warnings: [],
    };
    expect(artifactSchema.parse(artifact)).toEqual(artifact);
    const diagnostic = {
      contentHeightPx: 40,
      contentWidthPx: 100.5,
      itemId: "instance/text",
      lineCount: 1,
      overflowX: false,
      overflowY: false,
      resolvedFontSize: 32,
    };
    expect(Object.keys(diagnostic).sort()).toEqual(
      [...CATALOG.diagnosticFields].sort()
    );
    expect(
      artifactSchema.parse({ ...artifact, textLayouts: [diagnostic] })
        .textLayouts
    ).toEqual([diagnostic]);
  });
});
