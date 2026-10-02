import { describe, expect, it } from "vitest";
import contract from "../../../contracts/motion-blur-sampling-v1.json";
import { motionBlurSchema } from "../src/schemas";

describe("motion blur structural contracts", () => {
  it("agrees with all canonical accepted and rejected records", () => {
    for (const fixture of contract.cases) {
      expect(
        motionBlurSchema.safeParse(fixture.value).success,
        fixture.name
      ).toBe(fixture.accepted);
    }
    for (const shutterAngleDeg of [
      Number.NaN,
      Number.POSITIVE_INFINITY,
      Number.NEGATIVE_INFINITY,
    ]) {
      expect(
        motionBlurSchema.safeParse({ sampleCount: 8, shutterAngleDeg }).success
      ).toBe(false);
    }
  });
});
