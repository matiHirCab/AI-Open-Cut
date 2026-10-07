import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import mcp from "../../../contracts/mcp-surface-v1.json";
import contract from "../../../contracts/speech-alignment-v1.json";
import {
  generatedAssetOriginSchema,
  speechAlignmentSchema,
} from "../src/schemas";
import pins from "./fixtures/speech-alignment-predecessor-pins.json";
import {
  projectSpeechAlignmentCatalogPredecessor,
  projectSpeechAlignmentMcpPredecessor,
} from "./fixtures/speech-alignment-projection";

it.each(contract.valid)(
  "preserves canonical alignment $id exactly",
  ({ alignment }) => {
    expect(speechAlignmentSchema.parse(alignment)).toEqual(alignment);
  }
);
it.each(
  contract.invalid.filter((fixture) => fixture.failureKind === "structural")
)("rejects canonical structural case $id", ({ alignment }) => {
  expect(speechAlignmentSchema.safeParse(alignment).success).toBe(false);
});
it("keeps absent legacy alignment valid and refuses explicit null", () => {
  const generation = {
    generatedAtMs: 1,
    modelId: "synthesis-model",
    modelVersion: null,
    providerId: "synthesis",
    request: {
      language: "en",
      speed: 1,
      text: "Hello",
      textOptions: {},
      voiceId: "af_heart",
    },
    sampleRateHz: 24_000,
  };
  const legacy = generatedAssetOriginSchema.parse({
    generation,
    type: "speech_synthesis",
  });
  expect(Object.hasOwn(legacy.generation, "alignment")).toBe(false);
  expect(
    generatedAssetOriginSchema.safeParse({
      generation: { ...generation, alignment: null },
      type: "speech_synthesis",
    }).success
  ).toBe(false);
});
it("proves each manually updated current catalog against independently pinned schema37", () => {
  for (const name of pins.currentMarkerCatalogs) {
    const current: unknown = JSON.parse(
      readFileSync(
        resolve(import.meta.dirname, "../../../contracts", name),
        "utf8"
      )
    );
    expect(
      projectSpeechAlignmentCatalogPredecessor(name, current)
        .projectSchemaVersion
    ).toBe(37);
    expect(() =>
      projectSpeechAlignmentCatalogPredecessor(name, {
        ...(current as object),
        unrelated: true,
      })
    ).toThrow("Unrelated verified predecessor catalog drift");
  }
  expect(projectSpeechAlignmentMcpPredecessor(mcp)).toBeDefined();
  expect(() =>
    projectSpeechAlignmentMcpPredecessor({ ...mcp, unrelated: true })
  ).toThrow("Unrelated verified predecessor MCP drift");
});
