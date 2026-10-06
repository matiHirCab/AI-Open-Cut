import { createHash } from "node:crypto";

export const ORDERED_EFFECT_PREDECESSOR_SHA256 =
  "6912675bfa1e98c10420b0950c97321fe5d04c5af9395e836aa8e1ec4ac3a8a6";

export const sortedOrderedEffectJson = (value: unknown): unknown => {
  if (Array.isArray(value)) {
    return value.map(sortedOrderedEffectJson);
  }
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, child]) => [key, sortedOrderedEffectJson(child)])
    );
  }
  return value;
};

export const orderedEffectDigest = (value: unknown): string =>
  createHash("sha256")
    .update(JSON.stringify(sortedOrderedEffectJson(value)))
    .digest("hex");

export const projectOrderedEffectCatalogPredecessor = (
  source: unknown
): Record<string, unknown> => {
  if (!(source && typeof source === "object" && !Array.isArray(source))) {
    throw new Error("Ordered-effect predecessor requires a catalog object");
  }
  const input = source as Record<string, unknown>;
  if (
    !Object.hasOwn(input, "orderedEffectCases") ||
    orderedEffectDigest(input.orderedEffectCases) !==
      "8519348c8b58cb21a001138cc4cb9b7cd89b98c6a8876cd17e1c6c8fbc5a0a6c"
  ) {
    throw new Error("Missing or malformed approved orderedEffectCases");
  }
  const predecessor = structuredClone(input);
  Reflect.deleteProperty(predecessor, "orderedEffectCases");
  if (orderedEffectDigest(predecessor) !== ORDERED_EFFECT_PREDECESSOR_SHA256) {
    throw new Error(
      "Incorrect approved ordered-effect projection: Unrelated verified predecessor catalog drift"
    );
  }
  return predecessor;
};
