import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import catalog from "../../../../contracts/master-normalization-v1.json";
import headless from "./master-normalization-headless-additions.json";
import additions from "./master-normalization-mcp-additions.json";
import ownership from "./master-normalization-ownership-addition.json";
import pins from "./master-normalization-predecessor-pins.json";
import { removeMotionWorkflowPrompt } from "./motion-workflow-projection";
import { orderedEffectDigest } from "./ordered-effect-projection";

const record = (value: unknown): Record<string, unknown> => {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Malformed normalization projection");
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
const owns = (value: unknown, key: string): boolean =>
  value !== null &&
  typeof value === "object" &&
  !Array.isArray(value) &&
  Object.hasOwn(value, key);
const removeArrayValue = (source: unknown, expected: unknown) => {
  if (!Array.isArray(source)) {
    throw new Error("Malformed normalization array");
  }
  const matches = source.flatMap((value, index) =>
    orderedEffectDigest(value) === orderedEffectDigest(expected) ? [index] : []
  );
  if (matches.length !== 1) {
    throw new Error("Incorrect normalization array addition");
  }
  source.splice(matches[0] ?? -1, 1);
};
const removeProperty = (
  source: Record<string, unknown>,
  key: string,
  expected: unknown
) => {
  if (orderedEffectDigest(source[key]) !== orderedEffectDigest(expected)) {
    throw new Error("Incorrect normalization property addition");
  }
  Reflect.deleteProperty(source, key);
};

export const removeMasterNormalizationMcpAdditions = <T>(source: T): T => {
  const result = removeMotionWorkflowPrompt(source);
  const tools = record(at(result, ["toolDefinitions"]));
  const capabilities = at(result, ["capabilityIdentifiers"]);
  if (
    !Object.hasOwn(tools, catalog.tool) &&
    Array.isArray(capabilities) &&
    !capabilities.includes(catalog.capability)
  ) {
    return result;
  }
  for (const change of [...additions.additions].reverse()) {
    if (change.kind === "array") {
      removeArrayValue(at(result, change.path), change.value);
    } else {
      const parent = record(at(result, change.path.slice(0, -1)));
      const key = change.path.at(-1) ?? "";
      if (
        orderedEffectDigest(parent[key]) !== orderedEffectDigest(change.value)
      ) {
        throw new Error("Incorrect normalization property addition");
      }
      if (change.kind === "replacement" && "before" in change) {
        parent[key] = structuredClone(change.before);
      } else {
        Reflect.deleteProperty(parent, key);
      }
    }
  }
  return result;
};
export const removeMasterNormalizationHeadlessAdditions = <T>(source: T): T => {
  const result = structuredClone(source);
  const requests = record(at(result, ["requests"]));
  if (!Object.hasOwn(requests, "masterNormalizationSet")) {
    return result;
  }
  removeProperty(requests, "masterNormalizationSet", headless.request);
  removeArrayValue(
    at(result, ["status", "renderingCapabilities"]),
    catalog.capability
  );
  return result;
};
export const removeMasterNormalizationOwnershipAddition = <T>(source: T): T => {
  const result = structuredClone(source);
  const categories = record(at(result, ["categories"]));
  if (Object.hasOwn(categories, "masterNormalization")) {
    removeProperty(categories, "masterNormalization", ownership.category);
  }
  return result;
};
export const restoreMasterNormalizationCatalogMarker = (source: unknown) => {
  const result = structuredClone(record(source));
  if (result.projectSchemaVersion === catalog.projectSchemaVersion) {
    result.projectSchemaVersion = 43;
  }
  return result;
};

const capturedRaw = (name: string): string => {
  const path = `contracts/${name}.json` as keyof typeof pins.rawSha256;
  const expected = pins.rawSha256[path];
  if (!expected) {
    throw new Error("Uncaptured normalization predecessor");
  }
  const raw = readFileSync(
    resolve(
      import.meta.dirname,
      `master-normalization-${name}-predecessor.raw`
    ),
    "utf8"
  );
  if (createHash("sha256").update(raw).digest("hex") !== expected) {
    throw new Error("Normalization predecessor raw bytes changed");
  }
  return raw;
};

// The independently authored addition appends the one optional root property.
// Restore only that insertion order after the existing schema normalizer sorts
// keys; retain every actual value, extra key and older object's original order.
export const alignMasterNormalizationRuntimeObjectOrder = <T>(source: T): T => {
  const align = (value: unknown): unknown => {
    if (Array.isArray(value)) {
      return value.map(align);
    }
    if (!value || typeof value !== "object") {
      return value;
    }
    const entries = Object.entries(value).map(
      ([key, child]) => [key, align(child)] as const
    );
    const result = Object.fromEntries(entries);
    if (
      owns(result, "masterNormalization") &&
      owns(result, "soundDefinitions") &&
      owns(result, "audioBuses") &&
      owns(result, "schemaVersion")
    ) {
      return Object.fromEntries([
        ...entries.filter(([key]) => key !== "masterNormalization"),
        ["masterNormalization", result.masterNormalization],
      ]);
    }
    return result;
  };
  return align(source) as T;
};

// Restore only the ten explicitly captured changed catalogs. Semantic rollback
// must agree before any captured bytes may replace formatting or key order.
export const restoreMasterNormalizationRaw = (source: string): string => {
  const value = record(JSON.parse(source));
  let name: string | undefined;
  let projected: unknown;
  if (owns(value.toolDefinitions, catalog.tool)) {
    name = "mcp-surface-v1";
    projected = removeMasterNormalizationMcpAdditions(value);
  } else if (owns(value.categories, "masterNormalization")) {
    name = "contract-ownership-v1";
    projected = removeMasterNormalizationOwnershipAddition(value);
  } else if (owns(value.requests, "masterNormalizationSet")) {
    name = "headless-protocol-v1";
    projected = removeMasterNormalizationHeadlessAdditions(value);
  } else if (value.projectSchemaVersion === catalog.projectSchemaVersion) {
    projected = restoreMasterNormalizationCatalogMarker(value);
    for (const candidate of pins.markerCatalogs) {
      const raw = capturedRaw(candidate);
      if (
        orderedEffectDigest(JSON.parse(raw)) === orderedEffectDigest(projected)
      ) {
        if (source.match(/"projectSchemaVersion": 44/g)?.length !== 1) {
          throw new Error("Incorrect normalization marker representation");
        }
        const restored = source.replace(
          '"projectSchemaVersion": 44',
          '"projectSchemaVersion": 43'
        );
        if (restored !== raw) {
          throw new Error("Normalization reporting raw rollback differs");
        }
        return raw;
      }
    }
    throw new Error("Unrelated normalization reporting catalog drift");
  } else {
    return source;
  }
  const raw = capturedRaw(name);
  if (orderedEffectDigest(projected) !== orderedEffectDigest(JSON.parse(raw))) {
    throw new Error("Normalization complete predecessor rollback differs");
  }
  return raw;
};
export const masterNormalizationDigest = (value: unknown): string =>
  createHash("sha256").update(JSON.stringify(value)).digest("hex");
