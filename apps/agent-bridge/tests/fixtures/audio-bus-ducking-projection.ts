import { createHash } from "node:crypto";
import catalog from "../../../../contracts/audio-bus-ducking-v1.json";
import additions from "./audio-bus-ducking-mcp-additions.json";
import owners from "./audio-bus-ducking-ownership-addition.json";
import { orderedEffectDigest } from "./ordered-effect-projection";

const at = (source: unknown, path: string[]): Record<string, unknown> => {
  let value = source;
  for (const key of path) {
    if (!value || typeof value !== "object") {
      throw new Error("Malformed ducking projection");
    }
    value = (value as Record<string, unknown>)[key];
  }
  if (!value || typeof value !== "object") {
    throw new Error("Malformed ducking projection");
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
      throw new Error("Malformed ducking array");
    }
    const indexes = values.flatMap((v, i) =>
      orderedEffectDigest(v) === orderedEffectDigest(change.value) ? [i] : []
    );
    if (indexes.length !== 1) {
      throw new Error("Incorrect ducking addition");
    }
    values.splice(indexes[0] ?? -1, 1);
    return;
  }
  const parent = at(result, change.path.slice(0, -1));
  const key = change.path.at(-1) ?? "";
  if (orderedEffectDigest(parent[key]) !== orderedEffectDigest(change.value)) {
    throw new Error("Incorrect ducking property");
  }
  if (change.kind === "replacement" && "before" in change) {
    parent[key] = structuredClone(change.before);
  } else {
    Reflect.deleteProperty(parent, key);
  }
};
export const removeAudioBusDuckingMcpAdditions = <T>(source: T): T => {
  const result = structuredClone(source);
  const capabilities = at(result, []).capabilityIdentifiers;
  if (
    !(
      Array.isArray(capabilities) &&
      capabilities.includes("audio_bus_ducking_v1")
    )
  ) {
    return result;
  }
  for (const change of [...additions.additions].reverse()) {
    removeAddition(result, change);
  }
  return result;
};
export const removeAudioBusDuckingHeadlessAdditions = <T>(source: T): T => {
  const result = structuredClone(source);
  const requests = at(result, ["requests"]);
  if (!Object.hasOwn(requests, "audioBusSetDucking")) {
    return result;
  }
  const expected = {
    edit: catalog.input,
    expectedRevision: 0,
    operation: "edit",
    projectId: "project-1",
  };
  if (
    orderedEffectDigest(requests.audioBusSetDucking) !==
    orderedEffectDigest(expected)
  ) {
    throw new Error("Incorrect ducking headless addition");
  }
  Reflect.deleteProperty(requests, "audioBusSetDucking");
  const caps = at(result, ["status"]).renderingCapabilities;
  if (
    !Array.isArray(caps) ||
    caps.filter((v) => v === "audio_bus_ducking_v1").length !== 1
  ) {
    throw new Error("Incorrect ducking capability");
  }
  caps.splice(caps.indexOf("audio_bus_ducking_v1"), 1);
  return result;
};
export const removeAudioBusDuckingOwnershipAddition = <T>(source: T): T => {
  const result = structuredClone(source);
  const categories = at(result, ["categories"]);
  if (!Object.hasOwn(categories, "audioBusDucking")) {
    return result;
  }
  if (
    orderedEffectDigest(categories.audioBusDucking) !==
    orderedEffectDigest(owners.category)
  ) {
    throw new Error("Incorrect ducking ownership");
  }
  Reflect.deleteProperty(categories, "audioBusDucking");
  return result;
};
export const restoreAudioBusDuckingRawHeader = (source: string): string => {
  if (!source.includes('"projectSchemaVersion": 43')) {
    return source;
  }
  if (source.match(/"projectSchemaVersion": 43/g)?.length !== 1) {
    throw new Error("Incorrect ducking header");
  }
  return source.replace(
    '"projectSchemaVersion": 43',
    '"projectSchemaVersion": 42'
  );
};
export const audioBusDuckingDigest = (value: unknown): string =>
  createHash("sha256").update(JSON.stringify(value)).digest("hex");
