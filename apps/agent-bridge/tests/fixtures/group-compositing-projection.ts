import additions from "./group-compositing-additions.json";
import pins from "./group-compositing-predecessor-pins.json";
import { orderedEffectDigest } from "./ordered-effect-projection";
import {
  removeSpeechAlignmentMcpAdditions,
  restoreSpeechAlignmentCatalogMarker,
} from "./speech-alignment-projection";

type JsonRecord = Record<string, unknown>;
const object = (value: unknown): JsonRecord => {
  if (!(value && typeof value === "object" && !Array.isArray(value))) {
    throw new Error("Incorrect approved group-compositing object");
  }
  return value as JsonRecord;
};
export const GROUP_PREDECESSOR_PINS: Record<string, string> = pins.catalogs;
export const restoreGroupCatalogMarker = (source: unknown): JsonRecord => {
  const input = restoreSpeechAlignmentCatalogMarker(source);
  if (input.projectSchemaVersion !== 37) {
    throw new Error(
      "Incorrect approved group-compositing marker: Unrelated verified predecessor"
    );
  }
  return { ...structuredClone(input), projectSchemaVersion: 36 };
};
export const projectGroupCatalogPredecessor = (
  name: string,
  source: unknown
): JsonRecord => {
  if (!Object.hasOwn(GROUP_PREDECESSOR_PINS, name)) {
    throw new Error("Unapproved group-compositing predecessor catalog");
  }
  const previous = restoreGroupCatalogMarker(source);
  if (orderedEffectDigest(previous) !== GROUP_PREDECESSOR_PINS[name]) {
    throw new Error("Unrelated verified predecessor catalog drift");
  }
  return previous;
};
const parentAt = (source: JsonRecord, path: string[]) => {
  let parent = source;
  for (const key of path.slice(0, -1)) {
    parent = object(parent[key]);
  }
  const key = path.at(-1);
  if (!(key && Object.hasOwn(parent, key))) {
    throw new Error("Missing approved group-compositing field");
  }
  return { key, parent };
};
const literalPaths = [
  [
    "toolDefinitions",
    "editor_get_status",
    "outputSchema",
    "properties",
    "projectSchemaVersion",
    "const",
  ],
  [
    "$defs",
    "ProjectGetStateOutputPropertiesProjectProperties",
    "schemaVersion",
    "const",
  ],
];
const identifiers = ["group_compositing_models_v1", "group_compositing_v1"];
export const removeGroupMcpAdditions = (source: unknown): JsonRecord => {
  const input = removeSpeechAlignmentMcpAdditions(source);
  const capabilities = input.capabilityIdentifiers;
  if (
    !Array.isArray(capabilities) ||
    identifiers.some((id) => capabilities.filter((v) => v === id).length !== 1)
  ) {
    throw new Error(
      "Incorrect approved group-compositing capability multiplicity"
    );
  }
  const variants = object(object(input.$defs).ExtendedVisualEffect).oneOf;
  if (
    !Array.isArray(variants) ||
    variants.length !== 7 ||
    variants
      .slice(5)
      .some(
        (v, i) =>
          orderedEffectDigest(v) !== orderedEffectDigest(additions.effects[i])
      )
  ) {
    throw new Error("Incorrect approved group-compositing effect variants");
  }
  for (const path of literalPaths) {
    const { parent, key } = parentAt(input, path);
    if (parent[key] !== 37) {
      throw new Error("Incorrect approved group-compositing literal");
    }
  }
  for (const pointer of additions.paths) {
    const { parent, key } = parentAt(input, pointer.split("/").slice(1));
    const expected =
      pointer.includes("UpdateItemProperties") ||
      pointer.includes("/timeline_update_item/")
        ? { anyOf: [additions.clip, { type: "null" }] }
        : additions.clip;
    if (orderedEffectDigest(parent[key]) !== orderedEffectDigest(expected)) {
      throw new Error("Incorrect approved group-compositing clip field");
    }
  }
  const previous = object(structuredClone(input));
  previous.capabilityIdentifiers = capabilities.filter(
    (v) => !identifiers.includes(v)
  );
  object(object(previous.$defs).ExtendedVisualEffect).oneOf = structuredClone(
    variants.slice(0, 5)
  );
  for (const path of literalPaths) {
    const { parent, key } = parentAt(previous, path);
    parent[key] = 36;
  }
  for (const pointer of additions.paths) {
    const { parent, key } = parentAt(previous, pointer.split("/").slice(1));
    Reflect.deleteProperty(parent, key);
  }
  return previous;
};

export const projectGroupMcpPredecessor = (source: unknown): JsonRecord => {
  const previous = removeGroupMcpAdditions(source);
  // Independently frozen actual verified55 compact source detects all unrelated
  // fields/descriptions/annotations, while full expansion digests remain mandatory.
  if (orderedEffectDigest(previous) !== pins.compactMcpDigest) {
    throw new Error("Unrelated verified predecessor MCP drift");
  }
  return previous;
};
