import { createHash } from "node:crypto";
import additions from "./audio-buses-mcp-additions.json";
import ownership from "./audio-buses-ownership-addition.json";
import pin from "./audio-buses-predecessor-pins.json";
import { orderedEffectDigest } from "./ordered-effect-projection";

const at = (source: unknown, path: string[]): Record<string, unknown> => {
  let value = source;
  for (const key of path) {
    if (!value || typeof value !== "object") {
      throw new Error("Malformed audio-bus projection path");
    }
    value = (value as Record<string, unknown>)[key];
  }
  if (!value || typeof value !== "object") {
    throw new Error("Malformed audio-bus projection object");
  }
  return value as Record<string, unknown>;
};

export const restoreAudioBusCatalogMarker = (source: unknown) => {
  const value = at(source, []);
  if (value.projectSchemaVersion !== 39) {
    throw new Error("Incorrect approved audio-bus catalog marker");
  }
  return { ...structuredClone(value), projectSchemaVersion: 38 };
};

export const removeAudioBusMcpAdditions = (source: unknown) => {
  const result = structuredClone(source);
  for (const change of additions.additions) {
    if (change.kind === "array") {
      const array = at(result, change.path) as unknown as unknown[];
      if (!Array.isArray(array)) {
        throw new Error("Malformed audio-bus projection array");
      }
      const matches = array.flatMap((value, index) =>
        orderedEffectDigest(value) === orderedEffectDigest(change.value)
          ? [index]
          : []
      );
      if (matches.length !== 1) {
        throw new Error("Incorrect approved audio-bus array addition");
      }
      array.splice(matches[0] ?? -1, 1);
    } else {
      const object = at(result, change.path.slice(0, -1));
      const key = change.path.at(-1) ?? "";
      if (
        orderedEffectDigest(object[key]) !== orderedEffectDigest(change.value)
      ) {
        throw new Error("Incorrect approved audio-bus schema addition");
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

export const projectAudioBusMcpPredecessor = (source: unknown) => {
  const previous = removeAudioBusMcpAdditions(source);
  if (
    createHash("sha256").update(JSON.stringify(previous)).digest("hex") !==
    pin.mcpSemanticSha256
  ) {
    throw new Error("Unrelated audio-bus predecessor MCP drift");
  }
  return previous;
};

export const removeAudioBusHeadlessAdditions = <T>(source: T): T => {
  const result = structuredClone(source);
  const object = at(result, []);
  const requests = at(object, ["requests"]);
  const expected = {
    audioBusSetRoute: {
      edit: {
        busId: "music",
        operation: "audio_bus_set_route",
        outputBusId: "sfx",
      },
      expectedRevision: 0,
      operation: "edit",
      projectId: "project-1",
    },
    audioTrackRoute: {
      edit: {
        busId: "music",
        operation: "audio_track_route",
        scope: "root",
        trackId: "track-1",
      },
      expectedRevision: 1,
      operation: "edit",
      projectId: "project-1",
    },
  };
  for (const [key, value] of Object.entries(expected)) {
    if (orderedEffectDigest(requests[key]) !== orderedEffectDigest(value)) {
      throw new Error("Incorrect approved audio-bus headless request");
    }
    Reflect.deleteProperty(requests, key);
  }
  const capabilities = at(object, ["status"]).editorCapabilities;
  if (
    !Array.isArray(capabilities) ||
    capabilities.filter((v) => v === "project_audio_buses_v1").length !== 1
  ) {
    throw new Error("Incorrect approved audio-bus capability");
  }
  capabilities.splice(capabilities.indexOf("project_audio_buses_v1"), 1);
  return result;
};

export const removeAudioBusOwnershipAddition = <T>(source: T): T => {
  const result = structuredClone(source);
  const categories = at(result, ["categories"]);
  if (
    orderedEffectDigest(categories.audioBuses) !==
    orderedEffectDigest(ownership.category)
  ) {
    throw new Error("Incorrect approved audio-bus ownership addition");
  }
  Reflect.deleteProperty(categories, "audioBuses");
  return result;
};
