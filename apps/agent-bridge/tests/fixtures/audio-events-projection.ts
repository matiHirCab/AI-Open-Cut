import { createHash } from "node:crypto";
import {
  removeAudioBusDspHeadlessAdditions,
  removeAudioBusDspMcpAdditions,
  removeAudioBusDspOwnershipAddition,
} from "./audio-bus-dsp-projection";
import additions from "./audio-events-mcp-additions.json";
import ownership from "./audio-events-ownership-addition.json";
import pins from "./audio-events-predecessor-pins.json";
import { orderedEffectDigest } from "./ordered-effect-projection";

const digest = (value: unknown) =>
  createHash("sha256").update(JSON.stringify(value)).digest("hex");
const at = (source: unknown, path: string[]): Record<string, unknown> => {
  let value = source;
  for (const key of path) {
    if (!value || typeof value !== "object") {
      throw new Error("Malformed audio-event projection");
    }
    value = (value as Record<string, unknown>)[key];
  }
  if (!value || typeof value !== "object") {
    throw new Error("Malformed audio-event projection");
  }
  return value as Record<string, unknown>;
};
export const restoreAudioEventCatalogMarker = (source: unknown) => {
  const value = structuredClone(at(source, []));
  if (value.projectSchemaVersion === 43) {
    value.projectSchemaVersion = 42;
  }
  if (value.projectSchemaVersion === 42) {
    value.projectSchemaVersion = 41;
  }
  if (value.projectSchemaVersion !== 41) {
    throw new Error("Incorrect audio-event schema transition");
  }
  value.projectSchemaVersion = 40;
  return value;
};
export const removeAudioEventMcpAdditions = <T>(source: T): T => {
  const result = removeAudioBusDspMcpAdditions(source);
  for (const change of [...additions.additions].reverse()) {
    if (change.kind === "array") {
      const array = at(result, change.path) as unknown as unknown[];
      if (!Array.isArray(array)) {
        throw new Error("Malformed audio-event array");
      }
      const matches = array.flatMap((value, index) =>
        digest(value) === digest(change.value) ? [index] : []
      );
      if (matches.length !== 1) {
        throw new Error("Incorrect audio-event array addition");
      }
      array.splice(matches[0] ?? -1, 1);
    } else {
      const object = at(result, change.path.slice(0, -1));
      const key = change.path.at(-1) ?? "";
      if (digest(object[key]) !== digest(change.value)) {
        throw new Error("Incorrect audio-event property addition");
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
export const removeAudioEventHeadlessAdditions = <T>(source: T): T => {
  const result = removeAudioBusDspHeadlessAdditions(source);
  const root = at(result, []);
  const requests = at(root, ["requests"]);
  const expected = {
    edit: {
      at: { type: "milliseconds", valueMs: 250 },
      durationMs: 300,
      event: "impact",
      gainDb: -3,
      operation: "timeline_add_audio_event",
      scope: "root",
      trackId: "audio",
      variantSeed: 1,
    },
    expectedRevision: 0,
    operation: "edit",
    projectId: "project-1",
  };
  if (
    orderedEffectDigest(requests.timelineAddAudioEvent) !==
    orderedEffectDigest(expected)
  ) {
    throw new Error("Incorrect audio-event headless request");
  }
  Reflect.deleteProperty(requests, "timelineAddAudioEvent");
  const caps = at(root, ["status"]).editorCapabilities;
  if (
    !Array.isArray(caps) ||
    caps.filter((v) => v === "timeline_audio_events_v1").length !== 1
  ) {
    throw new Error("Incorrect audio-event capability");
  }
  caps.splice(caps.indexOf("timeline_audio_events_v1"), 1);
  return result;
};
export const removeAudioEventOwnershipAddition = <T>(source: T): T => {
  const result = removeAudioBusDspOwnershipAddition(source);
  const categories = at(result, ["categories"]);
  if (digest(categories.timelineAudioEvents) !== digest(ownership.category)) {
    throw new Error("Incorrect audio-event ownership");
  }
  Reflect.deleteProperty(categories, "timelineAudioEvents");
  return result;
};

export const projectAudioEventMcpPredecessor = <T>(source: T): T => {
  const result = removeAudioEventMcpAdditions(source);
  if (digest(result) !== pins.mcpSemanticSha256) {
    throw new Error("Unrelated audio-event predecessor MCP drift");
  }
  return result;
};
