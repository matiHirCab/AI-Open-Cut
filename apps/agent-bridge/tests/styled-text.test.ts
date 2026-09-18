import { describe, expect, it } from "vitest";
import CATALOG from "../../../contracts/styled-text-layers-v1.json";
import {
  richTextDocumentSchema,
  schemas,
  textStyleSchema,
} from "../src/schemas";
import { textPaintLayersSchema } from "../src/styled-text";

describe("styled text layers v1 transport", () => {
  it("preserves canonical optional fields without semantic revalidation", () => {
    for (const fixture of CATALOG.valid) {
      expect(richTextDocumentSchema.parse(fixture.document)).toEqual(
        fixture.document
      );
      if ("paintLayers" in fixture) {
        expect(
          textStyleSchema.parse({ paintLayers: fixture.paintLayers })
            .paintLayers
        ).toEqual(fixture.paintLayers);
      }
    }
    // Core owns range ordering, grapheme counts and nonempty effective styles.
    for (const fixture of CATALOG.invalid) {
      if ("document" in fixture) {
        expect(richTextDocumentSchema.safeParse(fixture.document).success).toBe(
          true
        );
      } else {
        expect(
          textPaintLayersSchema.safeParse(fixture.paintLayers).success
        ).toBe(false);
      }
    }
  });

  it("rejects malformed closed fields and negotiates runtime support", () => {
    for (const spans of [
      null,
      [{ end: 1, start: -1, style: { bold: true } }],
      [{ end: 1, start: 0, style: { bold: null } }],
      [{ end: 1, start: 0, style: { unknown: true } }],
    ]) {
      expect(
        richTextDocumentSchema.safeParse({ runs: [{ text: "a" }], spans })
          .success
      ).toBe(false);
    }
    expect(textStyleSchema.parse({ paintLayers: [] }).paintLayers).toEqual([]);
    expect(textStyleSchema.safeParse({ paintLayers: null }).success).toBe(
      false
    );
    expect(
      schemas.editorGetStatus.parse({ styledTextLayersVersion: 1 })
    ).toEqual({ styledTextLayersVersion: 1 });
    expect(
      schemas.editorGetStatus.safeParse({ styledTextLayersVersion: 2 }).success
    ).toBe(false);
  });
});
