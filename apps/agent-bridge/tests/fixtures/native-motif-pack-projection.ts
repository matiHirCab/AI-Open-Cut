import { createHash } from "node:crypto";
import addition from "./native-motif-pack-ownership-addition.json";
import pins from "./native-motif-pack-predecessor-pins.json";

const object = (value: unknown): Record<string, unknown> => {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Malformed native motif ownership");
  }
  return value as Record<string, unknown>;
};

export const assertCurrentNativeMotifOwnership = (source: unknown): void => {
  const categories = object(object(source).categories);
  if (
    !Object.hasOwn(categories, "nativeMotifPack") ||
    JSON.stringify(categories.nativeMotifPack) !== JSON.stringify(addition)
  ) {
    throw new Error("Missing or incomplete current native motif ownership");
  }
};

// Remove exactly this reviewed category. Historical registries remain identity;
// partial additions and unrelated drift are never repaired by projection.
export const removeNativeMotifOwnershipAddition = <T>(source: T): T => {
  const result = structuredClone(source);
  const categories = object(object(result).categories);
  if (Object.hasOwn(categories, "nativeMotifPack")) {
    if (
      JSON.stringify(categories.nativeMotifPack) !== JSON.stringify(addition)
    ) {
      throw new Error("Unapproved native motif ownership drift");
    }
    Reflect.deleteProperty(categories, "nativeMotifPack");
  }
  return result;
};

export const restoreNativeMotifRaw = (source: string): string => {
  const value = object(JSON.parse(source));
  if (
    !(
      value.categories &&
      Object.hasOwn(object(value.categories), "nativeMotifPack")
    )
  ) {
    return source;
  }
  const raw = `${JSON.stringify(removeNativeMotifOwnershipAddition(value), null, 2)}\n`;
  if (
    createHash("sha256").update(raw).digest("hex") !==
    pins["contract-ownership-v1"]
  ) {
    throw new Error("Unrelated native motif predecessor contract drift");
  }
  return raw;
};
