import { createHash } from "node:crypto";
import { orderedEffectDigest } from "./ordered-effect-projection";
import additions from "./semantic-sound-events-mcp-additions.json";
import ownership from "./semantic-sound-events-ownership-addition.json";
import pin from "./semantic-sound-events-predecessor-pins.json";

const at = (source: unknown, path: string[]): Record<string, unknown> => {
  let value = source;
  for (const key of path) {
    if (!value || typeof value !== "object") {
      throw new Error("Malformed sound-definition projection path");
    }
    value = (value as Record<string, unknown>)[key];
  }
  if (!value || typeof value !== "object") {
    throw new Error("Malformed sound-definition projection object");
  }
  return value as Record<string, unknown>;
};

export const restoreSoundEventCatalogMarker = (source: unknown) => {
  const value = at(source, []);
  if (value.projectSchemaVersion !== 40) {
    throw new Error("Incorrect approved sound-definition catalog marker");
  }
  return { ...structuredClone(value), projectSchemaVersion: 39 };
};

export const removeSoundEventMcpAdditions = (source: unknown) => {
  const result = structuredClone(source);
  for (const change of additions.additions) {
    if (change.kind === "array") {
      const array = at(result, change.path) as unknown as unknown[];
      if (!Array.isArray(array)) {
        throw new Error("Malformed sound-definition projection array");
      }
      const matches = array.flatMap((value, index) =>
        orderedEffectDigest(value) === orderedEffectDigest(change.value)
          ? [index]
          : []
      );
      if (matches.length !== 1) {
        throw new Error("Incorrect approved sound-definition array addition");
      }
      array.splice(matches[0] ?? -1, 1);
    } else {
      const object = at(result, change.path.slice(0, -1));
      const key = change.path.at(-1) ?? "";
      if (
        orderedEffectDigest(object[key]) !== orderedEffectDigest(change.value)
      ) {
        throw new Error("Incorrect approved sound-definition schema addition");
      }
      if (change.kind === "replacement") {
        object[key] = structuredClone(change.before);
      } else {
        Reflect.deleteProperty(object, key);
      }
    }
  }
  return result;
};

export const projectSoundEventMcpPredecessor = (source: unknown) => {
  const previous = removeSoundEventMcpAdditions(source);
  if (
    createHash("sha256").update(JSON.stringify(previous)).digest("hex") !==
    pin.mcpSemanticSha256
  ) {
    throw new Error("Unrelated sound-definition predecessor MCP drift");
  }
  return previous;
};

export const removeSoundEventHeadlessAdditions = <T>(source: T): T => {
  const result = structuredClone(source);
  const object = at(result, []);
  const requests = at(object, ["requests"]);
  const expected = {
    soundEventRegister: {
      edit: {
        busId: "sfx",
        defaultGainDb: -3,
        event: "impact",
        operation: "sound_event_register",
        variantAssetIds: ["asset-1"],
        variantSeed: 42,
      },
      expectedRevision: 0,
      operation: "edit",
      projectId: "project-1",
    },
  };
  for (const [key, value] of Object.entries(expected)) {
    if (orderedEffectDigest(requests[key]) !== orderedEffectDigest(value)) {
      throw new Error("Incorrect approved sound-definition headless request");
    }
    Reflect.deleteProperty(requests, key);
  }
  const capabilities = at(object, ["status"]).editorCapabilities;
  if (
    !Array.isArray(capabilities) ||
    capabilities.filter((v) => v === "semantic_sound_event_definitions_v1")
      .length !== 1
  ) {
    throw new Error("Incorrect approved sound-definition capability");
  }
  capabilities.splice(
    capabilities.indexOf("semantic_sound_event_definitions_v1"),
    1
  );
  return result;
};

export const removeSoundEventOwnershipAddition = <T>(source: T): T => {
  const result = structuredClone(source);
  const categories = at(result, ["categories"]);
  if (
    orderedEffectDigest(categories.semanticSoundEventDefinitions) !==
    orderedEffectDigest(ownership.category)
  ) {
    throw new Error("Incorrect approved sound-definition ownership addition");
  }
  Reflect.deleteProperty(categories, "semanticSoundEventDefinitions");
  return result;
};
