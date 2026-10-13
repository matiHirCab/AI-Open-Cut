import { createHash } from "node:crypto";
import addition from "./render-preparation-progress-ownership-addition.json";
import pins from "./render-preparation-progress-predecessor-pins.json";

const record = (value: unknown): Record<string, unknown> => {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Malformed preparation progress ownership");
  }
  return value as Record<string, unknown>;
};
const sorted = (value: unknown): unknown => {
  if (Array.isArray(value)) {
    return value.map(sorted);
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, child]) => [key, sorted(child)])
    );
  }
  return value;
};
export const assertRenderPreparationOwnership = (source: unknown): void => {
  const categories = record(record(source).categories);
  if (
    JSON.stringify(sorted(categories.renderPreparationProgress)) !==
    JSON.stringify(sorted(addition))
  ) {
    throw new Error(
      "Missing or changed preparation progress ownership addition"
    );
  }
};
export const removeRenderPreparationOwnershipAddition = <T>(source: T): T => {
  const value = record(source);
  if (
    !(
      value.categories &&
      Object.hasOwn(record(value.categories), "renderPreparationProgress")
    )
  ) {
    return source;
  }
  assertRenderPreparationOwnership(source);
  const result = structuredClone(value);
  Reflect.deleteProperty(
    record(result.categories),
    "renderPreparationProgress"
  );
  return result as T;
};
export const restoreRenderPreparationOwnershipRaw = (
  source: string
): string => {
  const value = JSON.parse(source);
  const projected = removeRenderPreparationOwnershipAddition(value);
  const raw = `${JSON.stringify(projected, null, 2)}\n`;
  if (
    createHash("sha256").update(raw).digest("hex") !==
    pins["contract-ownership-v1"]
  ) {
    throw new Error(
      "Unrelated preparation progress predecessor ownership drift"
    );
  }
  return raw;
};
