import { describe, expect, it } from "vitest";
import animation from "../../../contracts/animation-channels-v1.json";
import audioBuses from "../../../contracts/audio-buses-v1.json";
import contract from "../../../contracts/extended-visual-animation-v1.json";
import {
  animationChannelSchema,
  mediaCropSchema,
  visualEffectSchema,
} from "../src/schemas";
import { restoreAudioBusCatalogMarker } from "./fixtures/audio-buses-projection";

describe("extended visual contracts", () => {
  it("governs the activated properties and bounded core certification", () => {
    expect(contract.projectSchemaVersion).toBe(audioBuses.projectSchemaVersion);
    expect(restoreAudioBusCatalogMarker(contract).projectSchemaVersion).toBe(
      38
    );
    expect(contract.limits.maxCandidateAnalysisNodes).toBe(65_536);
    expect(contract.candidateCertification.unresolvedAtLimit).toBe(
      "INVALID_ARGUMENT"
    );
    for (const property of contract.activeProperties) {
      expect(Object.hasOwn(animation.active, property)).toBe(true);
      expect(Object.hasOwn(animation.inactive, property)).toBe(false);
    }
  });

  it("accepts closed effect fixtures and rejects unsupported provider fields", () => {
    for (const fixture of contract.effectCases) {
      expect(visualEffectSchema.safeParse(fixture.value).success).toBe(
        fixture.accepted
      );
    }
  });

  it("parses governed compound samples without discarding scoped targets", () => {
    for (const fixture of contract.sampleCases) {
      expect(
        animationChannelSchema.safeParse(fixture.channel).success,
        fixture.name
      ).toBe(true);
    }
  });

  it("preserves canonical channel cases for authoritative core validation", () => {
    for (const fixture of contract.channelCases) {
      expect(
        animationChannelSchema.safeParse(fixture.channel).success,
        fixture.name
      ).toBe(true);
    }
  });

  it("parses crop structure while leaving coupled source semantics to core", () => {
    for (const fixture of contract.cropCases) {
      const structural = !("url" in fixture.value) && fixture.value.width > 0;
      expect(mediaCropSchema.safeParse(fixture.value).success).toBe(structural);
    }
  });
});
