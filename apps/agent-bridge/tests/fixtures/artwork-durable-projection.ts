import { deepStrictEqual } from "node:assert/strict";
import durableContract from "../../../../contracts/durable-production-requests-v1.json";
import shortContract from "../../../../contracts/short-candidate-planning-cli-v1.json";
import pins from "./durable-main47-pins.json";
import {
  removeDurableMcpAdditions,
  removeDurableOwnershipAddition,
  restoreDurableRaw,
} from "./durable-production-projection";

export const removeDurableIntegration = <T>(source: T, copy = true): T => {
  if (!source || typeof source !== "object") {
    return source;
  }
  const value = source as Record<string, unknown>;
  const categories = value.categories as Record<string, unknown> | undefined;
  const hasDurable =
    Object.hasOwn(categories ?? {}, "durableProductionRequests") ||
    (Array.isArray(value.capabilityIdentifiers) &&
      value.capabilityIdentifiers.includes(durableContract.capability));
  if (
    JSON.stringify(value.ownership) === JSON.stringify(shortContract.ownership)
  ) {
    const normalized = copy ? structuredClone(source) : source;
    (normalized as Record<string, unknown>).ownership = structuredClone(
      pins.mainShortOwnership
    );
    return normalized;
  }
  const hasShort = Object.hasOwn(
    categories ?? {},
    "sourceGroundedShortPlanning"
  );
  if (!(hasDurable || hasShort || value.toolDefinitions)) {
    return source;
  }
  let result = removeDurableMcpAdditions(source, copy);
  result = removeDurableOwnershipAddition(result, result === source && copy);
  if (!result || typeof result !== "object" || !("categories" in result)) {
    return result;
  }
  const projectedCategories = result.categories as Record<
    string,
    unknown
  > | null;
  if (
    !(
      projectedCategories &&
      Object.hasOwn(projectedCategories, "sourceGroundedShortPlanning")
    )
  ) {
    return result;
  }
  const actual = projectedCategories.sourceGroundedShortPlanning;
  if (JSON.stringify(actual) === JSON.stringify(pins.mainShortOwnership)) {
    return result;
  }
  deepStrictEqual(actual, shortContract.ownership);
  if (result === source && copy) {
    result = structuredClone(result);
  }
  const normalized = result as Record<string, unknown>;
  const normalizedCategories = normalized.categories as Record<string, unknown>;
  normalizedCategories.sourceGroundedShortPlanning = structuredClone(
    pins.mainShortOwnership
  );
  return result;
};

export const restoreDurableIntegrationRaw = (input: string): string => {
  const raw = restoreDurableRaw(input);
  const source = JSON.parse(raw);
  const projected = removeDurableIntegration(source);
  if (JSON.stringify(source) === JSON.stringify(projected)) {
    return raw;
  }
  // Replace exactly the reviewed ownership bytes, never unrelated numeric lexemes.
  const key =
    source.ownership?.canonical === shortContract.ownership.canonical
      ? "ownership"
      : "sourceGroundedShortPlanning";
  for (const space of [2, undefined]) {
    const fragment = (value: unknown) =>
      space === 2
        ? JSON.stringify({ [key]: value }, null, 2)
            .split("\n")
            .slice(1, -1)
            .map((line) => `${key === "ownership" ? "" : "  "}${line}`)
            .join("\n")
        : JSON.stringify({ [key]: value }).slice(1, -1);
    const old = fragment(shortContract.ownership);
    const position = raw.indexOf(old);
    if (position < 0) {
      continue;
    }
    if (raw.indexOf(old, position + old.length) >= 0) {
      throw new Error("Repeated source planning ownership fragment");
    }
    const restored =
      raw.slice(0, position) +
      fragment(pins.mainShortOwnership) +
      raw.slice(position + old.length);
    deepStrictEqual(JSON.parse(restored), projected);
    return restored;
  }
  throw new Error("Missing exact source planning ownership fragment");
};
