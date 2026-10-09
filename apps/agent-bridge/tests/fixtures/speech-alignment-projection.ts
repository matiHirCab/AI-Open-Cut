import { createHash } from "node:crypto";
import { restoreAudioBusCatalogMarker } from "./audio-buses-projection";
import { removeKnownTextMcpAdditions } from "./known-text-projection";
import { orderedEffectDigest } from "./ordered-effect-projection";
import addition from "./speech-alignment-addition.json";
import pins from "./speech-alignment-predecessor-pins.json";

type RecordValue = Record<string, unknown>;
const object = (value: unknown): RecordValue => {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Incorrect approved speech alignment object");
  }
  return value as RecordValue;
};
const digest = (value: unknown) =>
  createHash("sha256").update(JSON.stringify(value)).digest("hex");
export const restoreSpeechAlignmentCatalogMarker = (
  source: unknown
): RecordValue => {
  const raw = object(source);
  const value =
    raw.projectSchemaVersion === 39 ||
    raw.projectSchemaVersion === 40 ||
    raw.projectSchemaVersion === 41 ||
    raw.projectSchemaVersion === 42 ||
    raw.projectSchemaVersion === 43 ||
    raw.projectSchemaVersion === 44
      ? restoreAudioBusCatalogMarker(raw)
      : raw;
  if (value.projectSchemaVersion !== 38) {
    throw new Error(
      "Incorrect approved speech alignment marker: Unrelated verified predecessor"
    );
  }
  return { ...structuredClone(value), projectSchemaVersion: 37 };
};
export const projectSpeechAlignmentCatalogPredecessor = (
  name: string,
  source: unknown
): RecordValue => {
  const previous = restoreSpeechAlignmentCatalogMarker(source);
  const pin = (pins.catalogs as Record<string, { semanticSha256: string }>)[
    name
  ];
  if (!pin || digest(previous) !== pin.semanticSha256) {
    throw new Error("Unrelated verified predecessor catalog drift");
  }
  return previous;
};
const at = (source: RecordValue, path: string[]): RecordValue => {
  let current: unknown = source;
  for (const key of path) {
    if (Array.isArray(current)) {
      current = current[Number(key)];
    } else {
      current = object(current)[key];
    }
  }
  return object(current);
};
export const removeSpeechAlignmentMcpAdditions = (
  source: unknown
): RecordValue => {
  const previous = object(removeKnownTextMcpAdditions(source));
  for (const path of pins.mcpVersionPaths) {
    const parent = at(previous, path.slice(0, -1));
    const key = path.at(-1);
    if (!key || parent[key] !== 38) {
      throw new Error("Incorrect approved speech alignment MCP literal");
    }
    parent[key] = 37;
  }
  const properties = at(previous, pins.mcpAlignmentParent);
  if (
    orderedEffectDigest(properties.alignment) !== orderedEffectDigest(addition)
  ) {
    throw new Error("Incorrect approved speech alignment MCP field");
  }
  Reflect.deleteProperty(properties, "alignment");
  return previous;
};
export const projectSpeechAlignmentMcpPredecessor = (
  source: unknown
): RecordValue => {
  const previous = removeSpeechAlignmentMcpAdditions(source);
  if (
    digest(previous) !== pins.catalogs["mcp-surface-v1.json"].semanticSha256
  ) {
    throw new Error("Unrelated verified predecessor MCP drift");
  }
  return previous;
};
