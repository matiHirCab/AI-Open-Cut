import { describe, expect, it } from "vitest";
import CATALOG from "../../../contracts/text-layout-v2.json";
import { schemas } from "../src/schemas";
import {
  fontBindingSchema,
  fontCatalogSchema,
  fontRecordSchema,
} from "../src/text-layout";

describe("canonical text layout v2", () => {
  it("preserves exact managed identities and closed bindings", () => {
    const font = CATALOG.valid.find((v) => v.id === "regular-face")?.font;
    expect(fontRecordSchema.parse(font)).toEqual(font);
    expect(fontCatalogSchema.parse({})).toEqual({});
    const { regular, bold, italic, boldItalic } = CATALOG.defaultFamily;
    const binding = {
      bold,
      boldItalic,
      italic,
      profile: CATALOG.profile,
      regular,
      warnings: [],
    };
    expect(fontBindingSchema.parse(binding)).toEqual(binding);
    for (const field of CATALOG.bindingFields) {
      const incomplete: Record<string, unknown> = { ...binding };
      delete incomplete[field];
      expect(fontBindingSchema.safeParse(incomplete).success).toBe(false);
    }
    expect(
      fontBindingSchema.safeParse({ ...binding, profile: "future" }).success
    ).toBe(false);
    expect(fontRecordSchema.safeParse({ ...font, faceIndex: 1 }).success).toBe(
      false
    );
    expect(
      fontRecordSchema.safeParse({
        ...font,
        sizeBytes: CATALOG.limits.fontBytes + 1,
      }).success
    ).toBe(false);
    expect(
      fontRecordSchema.safeParse({ ...font, injected: true }).success
    ).toBe(false);
  });

  it("negotiates layout without changing the default transport version", () => {
    expect(schemas.editorGetStatus.parse({})).toEqual({});
    expect(schemas.editorGetStatus.parse({ textLayoutVersion: 2 })).toEqual({
      textLayoutVersion: 2,
    });
    expect(
      schemas.editorGetStatus.safeParse({ textLayoutVersion: 3 }).success
    ).toBe(false);
  });
});
