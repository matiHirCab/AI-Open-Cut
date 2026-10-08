import { createHash } from "node:crypto";
import { removeAudioBusMcpAdditions } from "./audio-buses-projection";
import { orderedEffectDigest } from "./ordered-effect-projection";
import additions from "./speech-markers-mcp-additions.json";
import pin from "./speech-markers-predecessor-pins.json";

const at = (source: unknown, path: string[]): Record<string, unknown> => {
  let value = source;
  for (const key of path) {
    if (!value || typeof value !== "object") {
      throw new Error("Malformed speech-marker projection path");
    }
    value = (value as Record<string, unknown>)[key];
  }
  if (!value || typeof value !== "object") {
    throw new Error("Malformed speech-marker projection object");
  }
  return value as Record<string, unknown>;
};
export const removeSpeechMarkerMcpAdditions = (source: unknown) => {
  const result =
    source &&
    typeof source === "object" &&
    "toolDefinitions" in source &&
    source.toolDefinitions &&
    typeof source.toolDefinitions === "object" &&
    "audio_bus_set_route" in source.toolDefinitions
      ? removeAudioBusMcpAdditions(source)
      : structuredClone(source);
  for (const { kind, path, value } of additions.additions) {
    if (kind === "property") {
      const parent = at(result, path.slice(0, -1));
      const key = path.at(-1) ?? "";
      if (orderedEffectDigest(parent[key]) !== orderedEffectDigest(value)) {
        throw new Error("Incorrect approved speech-marker MCP addition");
      }
      Reflect.deleteProperty(parent, key);
    } else {
      const array = at(result, path) as unknown as unknown[];
      if (!Array.isArray(array)) {
        throw new Error("Incorrect approved speech-marker array");
      }
      const matches = array.flatMap((entry, index) =>
        orderedEffectDigest(entry) === orderedEffectDigest(value) ? [index] : []
      );
      if (matches.length !== 1) {
        throw new Error("Incorrect approved speech-marker array addition");
      }
      array.splice(matches[0] ?? -1, 1);
    }
  }
  return result;
};
export const projectSpeechMarkerMcpPredecessor = (source: unknown) => {
  const previous = removeSpeechMarkerMcpAdditions(source);
  if (
    createHash("sha256").update(JSON.stringify(previous)).digest("hex") !==
    pin.mcpSemanticSha256
  ) {
    throw new Error("Unrelated speech-marker predecessor MCP drift");
  }
  return previous;
};
