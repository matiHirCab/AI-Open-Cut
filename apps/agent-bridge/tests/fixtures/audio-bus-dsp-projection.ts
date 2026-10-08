import { createHash } from "node:crypto";
import catalog from "../../../../contracts/audio-bus-dsp-v1.json";
import additions from "./audio-bus-dsp-mcp-additions.json";
import owners from "./audio-bus-dsp-ownership-addition.json";
import {
  removeAudioBusDuckingHeadlessAdditions,
  removeAudioBusDuckingMcpAdditions,
  removeAudioBusDuckingOwnershipAddition,
  restoreAudioBusDuckingRawHeader,
} from "./audio-bus-ducking-projection";
import { orderedEffectDigest } from "./ordered-effect-projection";

const at = (source: unknown, path: string[]): Record<string, unknown> => {
  let value = source;
  for (const key of path) {
    if (!value || typeof value !== "object") {
      throw new Error("Malformed DSP projection");
    }
    value = (value as Record<string, unknown>)[key];
  }
  if (!value || typeof value !== "object") {
    throw new Error("Malformed DSP projection");
  }
  return value as Record<string, unknown>;
};
const removeAddition = (
  result: unknown,
  change: (typeof additions.additions)[number]
) => {
  if (change.kind === "array") {
    const values = at(result, change.path) as unknown as unknown[];
    if (!Array.isArray(values)) {
      throw new Error("Malformed DSP array");
    }
    const indexes = values.flatMap((v, i) =>
      orderedEffectDigest(v) === orderedEffectDigest(change.value) ? [i] : []
    );
    if (indexes.length !== 1) {
      throw new Error("Incorrect DSP addition");
    }
    values.splice(indexes[0] ?? -1, 1);
    return;
  }
  const parent = at(result, change.path.slice(0, -1));
  const key = change.path.at(-1) ?? "";
  if (orderedEffectDigest(parent[key]) !== orderedEffectDigest(change.value)) {
    throw new Error("Incorrect DSP property");
  }
  if (change.kind === "replacement" && "before" in change) {
    parent[key] = structuredClone(change.before);
  } else {
    Reflect.deleteProperty(parent, key);
  }
};
export const removeAudioBusDspMcpAdditions = <T>(source: T): T => {
  const result = removeAudioBusDuckingMcpAdditions(source);
  const capabilities = at(result, []).capabilityIdentifiers;
  if (
    !(Array.isArray(capabilities) && capabilities.includes("audio_bus_dsp_v1"))
  ) {
    return result;
  }
  for (const change of [...additions.additions].reverse()) {
    removeAddition(result, change);
  }
  return result;
};
export const removeAudioBusDspHeadlessAdditions = <T>(source: T): T => {
  const result = removeAudioBusDuckingHeadlessAdditions(source);
  const requests = at(result, ["requests"]);
  if (!Object.hasOwn(requests, "audioBusSetDsp")) {
    return result;
  }
  const expected = {
    edit: catalog.input,
    expectedRevision: 0,
    operation: "edit",
    projectId: "project-1",
  };
  if (
    orderedEffectDigest(requests.audioBusSetDsp) !==
    orderedEffectDigest(expected)
  ) {
    throw new Error("Incorrect DSP headless addition");
  }
  Reflect.deleteProperty(requests, "audioBusSetDsp");
  const caps = at(result, ["status"]).renderingCapabilities;
  if (
    !Array.isArray(caps) ||
    caps.filter((v) => v === "audio_bus_dsp_v1").length !== 1
  ) {
    throw new Error("Incorrect DSP capability");
  }
  caps.splice(caps.indexOf("audio_bus_dsp_v1"), 1);
  return result;
};
export const removeAudioBusDspOwnershipAddition = <T>(source: T): T => {
  const result = removeAudioBusDuckingOwnershipAddition(source);
  const categories = at(result, ["categories"]);
  if (!Object.hasOwn(categories, "audioBusDsp")) {
    return result;
  }
  if (
    orderedEffectDigest(categories.audioBusDsp) !==
    orderedEffectDigest(owners.category)
  ) {
    throw new Error("Incorrect DSP ownership");
  }
  Reflect.deleteProperty(categories, "audioBusDsp");
  return result;
};
export const restoreAudioBusDspRawHeader = (source: string): string => {
  const restored = restoreAudioBusDuckingRawHeader(source);
  if (!restored.includes('"projectSchemaVersion": 42')) {
    return restored;
  }
  if (restored.match(/"projectSchemaVersion": 42/g)?.length !== 1) {
    throw new Error("Incorrect DSP header");
  }
  return restored.replace(
    '"projectSchemaVersion": 42',
    '"projectSchemaVersion": 41'
  );
};
export const audioBusDspDigest = (value: unknown): string =>
  createHash("sha256").update(JSON.stringify(value)).digest("hex");
