import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import {
  removeFfmpegSpawnHeadlessAddition,
  removeFfmpegSpawnMcpAddition,
  removeFfmpegSpawnOwnershipAddition,
} from "./ffmpeg-spawn-projection";
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
  let projected = removeNativeMotifOwnershipAddition(value);
  if (!Object.hasOwn(object(value.categories), "ffmpegSpawnDiagnostics")) {
    throw new Error("Missing merged ffmpegSpawnDiagnostics ownership");
  }
  projected = removeFfmpegSpawnOwnershipAddition(projected);
  const raw = `${JSON.stringify(projected, null, 2)}\n`;
  if (
    createHash("sha256").update(raw).digest("hex") !==
    pins["contract-ownership-v1"]
  ) {
    throw new Error("Unrelated native motif predecessor contract drift");
  }
  return raw;
};

// Invert the merged, strictly owned FF addition, then prove full ordered semantic
// equality to independently captured pre-addition raw bytes. Existing pins and
// bytes remain unchanged; unrelated catalog drift cannot be projected away.
export const restoreNativeMotifMergedCatalogRaw = (
  source: string,
  name: string
): string => {
  if (name !== "mcp-surface-v1" && name !== "headless-protocol-v1") {
    return source;
  }
  const parsed = JSON.parse(source);
  const projected =
    name === "mcp-surface-v1"
      ? removeFfmpegSpawnMcpAddition(parsed)
      : removeFfmpegSpawnHeadlessAddition(parsed);
  const raw = readFileSync(
    new URL(`./native-motif-pack-predecessor-${name}.raw`, import.meta.url),
    "utf8"
  );
  if (
    JSON.stringify(JSON.parse(raw)) !== JSON.stringify(projected) ||
    createHash("sha256").update(raw).digest("hex") !== pins[name]
  ) {
    throw new Error("Unrelated merged motif catalog drift");
  }
  return raw;
};
