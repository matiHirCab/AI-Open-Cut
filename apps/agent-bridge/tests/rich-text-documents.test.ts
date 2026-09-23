import { describe, expect, it } from "vitest";
import CATALOG from "../../../contracts/rich-text-documents-v1.json";
import {
  headlessEditSchema,
  richTextDocumentSchema,
  schemas,
} from "../src/schemas";

describe("canonical rich text documents", () => {
  it("accepts the governed additive font-size edit in standalone and batch schemas", () => {
    const fixture = CATALOG.fontSizeUpdate;
    expect(fixture.field).toBe("fontSize");
    for (const fontSize of [fixture.minimum, fixture.maximum]) {
      const edit = { fontSize, itemId: "title", operation: "update_item" };
      expect(headlessEditSchema.parse(edit)).toMatchObject(edit);
      expect(
        schemas.timelineUpdateItem.parse({
          expectedRevision: 1,
          fontSize,
          itemId: "title",
          projectId: "project",
        })
      ).toMatchObject({ fontSize });
    }
    for (const fontSize of [null, 0, 1001, 1.5]) {
      expect(
        headlessEditSchema.safeParse({
          fontSize,
          itemId: "title",
          operation: "update_item",
        }).success
      ).toBe(false);
      expect(
        schemas.timelineUpdateItem.safeParse({
          expectedRevision: 1,
          fontSize,
          itemId: "title",
          projectId: "project",
        }).success
      ).toBe(false);
    }
  });
  it("preserves every run through standalone, batch and draft inputs", () => {
    for (const fixture of CATALOG.valid) {
      const edit = {
        color: "#ffffff",
        document: fixture.document,
        durationMs: 1000,
        fontSize: 48,
        operation: "add_text",
        startMs: 0,
        trackId: "overlay",
        transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
      };
      expect(richTextDocumentSchema.parse(fixture.document)).toEqual(
        fixture.document
      );
      expect(headlessEditSchema.parse(edit)).toMatchObject(edit);
      const { operation: _, ...input } = edit;
      expect(
        schemas.timelineAddText.parse({
          ...input,
          expectedRevision: 0,
          projectId: "project",
        })
      ).toMatchObject(input);
      expect(
        schemas.timelineUpdateItem.parse({
          document: fixture.document,
          expectedRevision: 0,
          itemId: "item",
          projectId: "project",
        })
      ).toMatchObject({ document: fixture.document });
    }
  });

  it("rejects malformed closed documents without losing unknown fields", () => {
    for (const fixture of CATALOG.invalid.filter(
      (v) => v.stage === "structure"
    )) {
      expect(
        richTextDocumentSchema.safeParse(fixture.document).success,
        fixture.id
      ).toBe(false);
    }
    for (const key of ["__proto__", "constructor", "toString", "fontPath"]) {
      const document = JSON.parse(`{"runs":[{"text":"x","${key}":true}]}`);
      expect(richTextDocumentSchema.safeParse(document).success).toBe(false);
    }
    expect(
      richTextDocumentSchema.safeParse({ runs: [{ text: "\ud800" }] }).success
    ).toBe(false);
  });

  it("retains simple inputs and rejects explicit null", () => {
    const input = {
      durationMs: 1000,
      expectedRevision: 0,
      projectId: "project",
      startMs: 0,
      text: "Hello",
      trackId: "overlay",
    };
    expect(schemas.timelineAddText.parse(input)).toMatchObject(input);
    expect(
      schemas.timelineAddText.safeParse({ ...input, document: null }).success
    ).toBe(false);
    expect(
      schemas.timelineUpdateItem.safeParse({
        expectedRevision: 0,
        itemId: "item",
        projectId: "project",
        text: null,
      }).success
    ).toBe(false);
  });
});
