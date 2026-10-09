import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { z } from "zod/v4";
import delivery from "../../../contracts/artifact-delivery-v2.json";
import catalog from "../../../contracts/audio-analysis-v1.json";
import ownership from "../../../contracts/contract-ownership-v1.json";
import headless from "../../../contracts/headless-protocol-v1.json";
import mcp from "../../../contracts/mcp-surface-v1.json";
import worker from "../../../contracts/render-worker-v1.json";
import {
  audioAnalysisArtifactSchema,
  audioAnalysisResultSchema,
  audioAnalysisSummarySchema,
  audioWaveformBinSchema,
  schemas,
} from "../src/schemas";
import dtos from "./fixtures/audio-analysis-dtos.json";
import pins from "./fixtures/audio-analysis-predecessor-pins.json";
import {
  alignAudioAnalysisRuntimeObjectOrder,
  audioAnalysisDigest,
  removeAudioAnalysisDeliveryAddition,
  removeAudioAnalysisHeadlessAdditions,
  removeAudioAnalysisMcpAdditions,
  removeAudioAnalysisOwnershipAddition,
  removeAudioAnalysisWorkerAdditions,
  restoreAudioAnalysisRaw,
} from "./fixtures/audio-analysis-projection";
import { removeMasterNormalizationMcpAdditions } from "./fixtures/master-normalization-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";

const hash = (value: string | Uint8Array) =>
  createHash("sha256").update(value).digest("hex");
it("preserves all51 independently committed catalogs and exact85→86 expanded contracts", () => {
  expect(pins.predecessorCommit).toBe(
    "27f3fb2812a23d1aa8a5ccd73d60c4cae17c1625"
  );
  expect(pins.predecessorAll11Success).toBe(true);
  expect(pins.predecessorCiRun).toBe(37_830_821_588);
  expect(Object.keys(pins.catalogRawSha256)).toHaveLength(51);
  for (const [path, pin] of Object.entries(pins.catalogRawSha256)) {
    expect(
      hash(
        restoreAudioAnalysisRaw(
          readFileSync(resolve(import.meta.dirname, "../../..", path), "utf8")
        )
      ),
      path
    ).toBe(pin);
  }
  const projections = [
    ["mcp-surface-v1", removeAudioAnalysisMcpAdditions(mcp)],
    ["headless-protocol-v1", removeAudioAnalysisHeadlessAdditions(headless)],
    ["contract-ownership-v1", removeAudioAnalysisOwnershipAddition(ownership)],
    ["render-worker-v1", removeAudioAnalysisWorkerAdditions(worker)],
    ["artifact-delivery-v2", removeAudioAnalysisDeliveryAddition(delivery)],
  ] as const;
  for (const [name, value] of projections) {
    const raw = readFileSync(
      resolve(
        import.meta.dirname,
        "fixtures",
        `audio-analysis-${name}-predecessor.raw`
      ),
      "utf8"
    );
    expect(value, name).toEqual(JSON.parse(raw));
  }
  expect(audioAnalysisDigest(removeAudioAnalysisMcpAdditions(mcp))).toBe(
    pins.mcpSemanticSha256
  );
  expect(
    audioAnalysisDigest(
      expandMcpSurfaceCatalog(removeAudioAnalysisMcpAdditions(mcp))
    )
  ).toBe(pins.mcpExpandedSha256);
  expect(
    audioAnalysisDigest(
      expandMcpSurfaceCatalog(removeMasterNormalizationMcpAdditions(mcp))
    )
  ).toBe(pins.manuallyReviewedCurrentExpandedSha256);
  expect(
    expandMcpSurfaceCatalog(removeMasterNormalizationMcpAdditions(mcp)).tools
  ).toHaveLength(86);
  // Canonical object ordering must preserve every key/scalar/array and all pins.
  expect(
    alignAudioAnalysisRuntimeObjectOrder(
      expandMcpSurfaceCatalog(removeMasterNormalizationMcpAdditions(mcp))
    )
  ).toEqual(
    expandMcpSurfaceCatalog(removeMasterNormalizationMcpAdditions(mcp))
  );
});
it("matches independently authored closed bounded DTO schemas", () => {
  for (const [schema, expected] of [
    [audioAnalysisSummarySchema, dtos.summary],
    [audioWaveformBinSchema, dtos.waveformBin],
    [audioAnalysisArtifactSchema, dtos.artifact],
    [audioAnalysisResultSchema, dtos.result],
  ] as const) {
    expect(z.toJSONSchema(schema)).toEqual(expected);
  }
  const { operation: _operation, ...input } = catalog.input;
  expect(schemas.audioAnalyzeMix.parse(input)).toEqual(input);
  expect(headless.requests.analyzeAudio).toEqual(catalog.input);
  expect(audioAnalysisSummarySchema.parse(catalog.silenceExample)).toEqual(
    catalog.silenceExample
  );
  expect(audioAnalysisSummarySchema.parse(catalog.shortAudibleExample)).toEqual(
    catalog.shortAudibleExample
  );
  expect(catalog.shortAudibleExample.integratedLufs).toBeNull();
  expect(catalog.shortAudibleExample.truePeakDbtp).not.toBeNull();
});
it("rejects unbounded nonfinite unknown and incomplete public values", () => {
  const { operation: _operation, ...input } = catalog.input;
  for (const value of [
    { ...input, waveformBins: 0 },
    { ...input, waveformBins: 4097 },
    { ...input, endMs: 600_001 },
    { ...input, startMs: -1 },
    { ...input, waveformBins: 1.5 },
    { ...input, rawFilter: "volume=3" },
  ]) {
    expect(schemas.audioAnalyzeMix.safeParse(value).success).toBe(false);
  }
  for (const key of Object.keys(input)) {
    const missing: Record<string, unknown> = { ...input };
    Reflect.deleteProperty(missing, key);
    expect(schemas.audioAnalyzeMix.safeParse(missing).success).toBe(false);
  }
  for (const value of [
    { ...catalog.summaryExample, linearSamplePeak: Number.NaN },
    { ...catalog.summaryExample, integratedLufs: Number.NEGATIVE_INFINITY },
    { ...catalog.summaryExample, truePeakDbtp: Number.POSITIVE_INFINITY },
    { ...catalog.summaryExample, loudnessRangeLu: -1 },
    { ...catalog.summaryExample, frameCount: 28_800_001 },
    { ...catalog.summaryExample, actualBinCount: 4097 },
    { ...catalog.summaryExample, channels: 1 },
    { ...catalog.summaryExample, rawPath: "/private" },
  ]) {
    expect(audioAnalysisSummarySchema.safeParse(value).success).toBe(false);
  }
});
it("detects catalog addition corruption without rewriting predecessors", () => {
  const changed = structuredClone(mcp);
  changed.toolDefinitions.audio_analyze_mix.annotations.readOnlyHint = true;
  expect(() => removeAudioAnalysisMcpAdditions(changed)).toThrow();
  const changedWorker = structuredClone(worker);
  const analysisRequest = changedWorker.requests.at(-1);
  if (!analysisRequest) {
    throw new Error("Missing analysis worker fixture");
  }
  analysisRequest.request = {
    ...catalog.input,
    waveformBins: 5,
  };
  expect(() => removeAudioAnalysisWorkerAdditions(changedWorker)).toThrow();
  const changedRaw = JSON.stringify({ ...delivery, version: 99 });
  expect(() => restoreAudioAnalysisRaw(changedRaw)).toThrow();
});
