import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import catalog from "../../../../contracts/audio-analysis-v1.json";
import lifetime from "./audio-analysis-lifetime-additions.json";
import additions from "./audio-analysis-mcp-additions.json";
import owners from "./audio-analysis-ownership-addition.json";
import pins from "./audio-analysis-predecessor-pins.json";
import { orderedEffectDigest } from "./ordered-effect-projection";

const record = (value: unknown): Record<string, unknown> => {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Malformed analysis projection");
  }
  return value as Record<string, unknown>;
};
const at = (source: unknown, path: string[]): unknown => {
  let value = source;
  for (const key of path) {
    value = record(value)[key];
  }
  return value;
};
const removeArrayValue = (source: unknown, expected: unknown) => {
  if (!Array.isArray(source)) {
    throw new Error("Malformed analysis array");
  }
  const indexes = source.flatMap((value, index) =>
    orderedEffectDigest(value) === orderedEffectDigest(expected) ? [index] : []
  );
  if (indexes.length !== 1) {
    throw new Error("Incorrect analysis array addition");
  }
  source.splice(indexes[0] ?? -1, 1);
};
const removeProperty = (
  source: Record<string, unknown>,
  key: string,
  expected: unknown
) => {
  if (orderedEffectDigest(source[key]) !== orderedEffectDigest(expected)) {
    throw new Error("Incorrect analysis property addition");
  }
  Reflect.deleteProperty(source, key);
};

export const removeAudioAnalysisMcpAdditions = <T>(source: T): T => {
  const result = structuredClone(source);
  const capabilities = record(result).capabilityIdentifiers;
  if (
    !(Array.isArray(capabilities) && capabilities.includes(catalog.capability))
  ) {
    return result;
  }
  for (const change of [...additions.additions].reverse()) {
    if (change.kind === "array") {
      removeArrayValue(at(result, change.path), change.value);
    } else {
      removeProperty(
        record(at(result, change.path.slice(0, -1))),
        change.path.at(-1) ?? "",
        change.value
      );
    }
  }
  return result;
};
export const removeAudioAnalysisHeadlessAdditions = <T>(source: T): T => {
  const result = structuredClone(source);
  const requests = record(at(result, ["requests"]));
  if (!Object.hasOwn(requests, "analyzeAudio")) {
    return result;
  }
  removeProperty(requests, "analyzeAudio", catalog.input);
  removeArrayValue(at(result, ["operations"]), catalog.operation);
  removeArrayValue(
    at(result, ["status", "renderingCapabilities"]),
    catalog.capability
  );
  return result;
};
export const removeAudioAnalysisOwnershipAddition = <T>(source: T): T => {
  const result = structuredClone(source);
  const categories = record(at(result, ["categories"]));
  if (Object.hasOwn(categories, "audioAnalysis")) {
    removeProperty(categories, "audioAnalysis", owners.category);
  }
  return result;
};
export const removeAudioAnalysisWorkerAdditions = <T>(source: T): T => {
  const result = structuredClone(source);
  const operations = at(result, ["operations"]);
  if (Array.isArray(operations) && operations.includes(catalog.operation)) {
    removeArrayValue(operations, catalog.operation);
    removeArrayValue(at(result, ["requests"]), lifetime.workerRequest);
    removeArrayValue(at(result, ["events"]), lifetime.workerEvent);
  }
  return result;
};
export const removeAudioAnalysisDeliveryAddition = <T>(source: T): T => {
  const result = structuredClone(source);
  const fixtures = at(result, ["fixtures"]);
  if (
    Array.isArray(fixtures) &&
    fixtures.some((value) => record(value).kind === catalog.jobKind)
  ) {
    removeArrayValue(fixtures, lifetime.artifactFixture);
  }
  return result;
};

// Only independently captured changed catalogs have a byte-restoration path.
// First require exact semantic rollback; a changed or missing addition fails.
export const restoreAudioAnalysisRaw = (source: string): string => {
  const parsed: unknown = JSON.parse(source);
  const value = record(parsed);
  let name: string;
  let projected: unknown;
  if (Array.isArray(value.tools) && value.tools.includes(catalog.tool)) {
    name = "mcp-surface-v1";
    projected = removeAudioAnalysisMcpAdditions(parsed);
  } else if (record(value.categories ?? {}).audioAnalysis) {
    name = "contract-ownership-v1";
    projected = removeAudioAnalysisOwnershipAddition(parsed);
  } else if (
    Array.isArray(value.operations) &&
    value.operations.includes(catalog.operation)
  ) {
    if (Array.isArray(value.requests)) {
      name = "render-worker-v1";
      projected = removeAudioAnalysisWorkerAdditions(parsed);
    } else {
      name = "headless-protocol-v1";
      projected = removeAudioAnalysisHeadlessAdditions(parsed);
    }
  } else if (
    Array.isArray(value.fixtures) &&
    value.fixtures.some((fixture) => record(fixture).kind === catalog.jobKind)
  ) {
    name = "artifact-delivery-v2";
    projected = removeAudioAnalysisDeliveryAddition(parsed);
  } else {
    return source;
  }
  const raw = readFileSync(
    resolve(import.meta.dirname, `audio-analysis-${name}-predecessor.raw`),
    "utf8"
  );
  const path = `contracts/${name}.json` as keyof typeof pins.rawSha256;
  if (
    createHash("sha256").update(raw).digest("hex") !== pins.rawSha256[path] ||
    orderedEffectDigest(projected) !== orderedEffectDigest(JSON.parse(raw))
  ) {
    throw new Error("Analysis byte rollback differs from captured predecessor");
  }
  return raw;
};
export const audioAnalysisDigest = (value: unknown): string =>
  createHash("sha256").update(JSON.stringify(value)).digest("hex");

// Adapt only insertion order introduced by the manually authored analysis
// addition. Scalar values, arrays, absent/extra keys and older objects retain
// their exact runtime representation; no fixture values replace producer data.
export const alignAudioAnalysisRuntimeObjectOrder = <T>(source: T): T => {
  const summaryKeys = [
    "startMs",
    "endMs",
    "sampleRateHz",
    "channels",
    "frameCount",
    "actualBinCount",
    "linearSamplePeak",
    "samplePeakDbfs",
    "integratedLufs",
    "truePeakDbtp",
    "loudnessRangeLu",
    "thresholdLufs",
  ];
  const align = (value: unknown): unknown => {
    if (Array.isArray(value)) {
      return value.map(align);
    }
    if (value !== null && typeof value === "object") {
      const object = value as Record<string, unknown>;
      let keys = Object.keys(object);
      if (
        keys.length === summaryKeys.length &&
        summaryKeys.every((key) => Object.hasOwn(object, key))
      ) {
        keys = summaryKeys;
      } else if (Object.hasOwn(object, "audioAnalysis")) {
        keys = [
          ...keys.filter((key) => key !== "audioAnalysis"),
          "audioAnalysis",
        ];
      }
      return Object.fromEntries(keys.map((key) => [key, align(object[key])]));
    }
    return value;
  };
  return align(source) as T;
};
