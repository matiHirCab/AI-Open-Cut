import { describe, expect, it } from "vitest";
import fixtures from "../../../contracts/transform2d-v1.json";
import { schemas, transform2dSchema } from "../src/schemas";

describe("canonical Transform2D runtime contract", () => {
  it("accepts complete fixtures and rejects malformed values", () => {
    for (const entry of fixtures.valid) {
      expect(transform2dSchema.parse(entry.value)).toEqual(entry.value);
    }
    for (const entry of fixtures.invalid) {
      expect(transform2dSchema.safeParse(entry.value).success, entry.id).toBe(
        false
      );
    }
  });
  it("preserves omitted versus explicit reset updates", () => {
    const base = { expectedRevision: 0, itemId: "box", projectId: "project" };
    expect(
      schemas.timelineUpdateItem.parse({ ...base, transform2d: null })
        .transform2d
    ).toBeNull();
    expect(
      schemas.timelineUpdateItem.parse({ ...base, text: "unchanged transform" })
        .transform2d
    ).toBeUndefined();
    expect(
      schemas.timelineUpdateItem.parse({
        ...base,
        transform2d: fixtures.identity,
      }).transform2d
    ).toEqual(fixtures.identity);
  });
});
