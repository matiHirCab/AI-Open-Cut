import { createHash } from "node:crypto";
import additions from "./known-text-mcp-additions.json";
import pin from "./known-text-mcp-predecessor-pin.json";
import { orderedEffectDigest } from "./ordered-effect-projection";

const at = (source: unknown, path: string[]): Record<string, unknown> => {
  let value = source;
  for (const key of path) {
    if (!(value && typeof value === "object")) {
      throw new Error("Malformed known-text projection path");
    }
    value = (value as Record<string, unknown>)[key];
  }
  if (!(value && typeof value === "object")) {
    throw new Error("Malformed known-text projection object");
  }
  return value as Record<string, unknown>;
};
// Remove only exact approved additions, keeping unrelated drift observable to every older proof.
export const removeKnownTextMcpAdditions = (source: unknown) => {
  const result = structuredClone(source);
  for (const { path, value } of additions.additions) {
    const parent = at(result, path.slice(0, -1));
    const key = path.at(-1) ?? "";
    if (orderedEffectDigest(parent[key]) !== orderedEffectDigest(value)) {
      throw new Error("Incorrect approved known-text MCP addition");
    }
    Reflect.deleteProperty(parent, key);
  }
  const required = at(result, additions.requiredPath) as unknown as string[];
  if (
    !Array.isArray(required) ||
    required.filter((value) => value === additions.requiredAddition).length !==
      1
  ) {
    throw new Error("Incorrect approved known-text MCP requirement");
  }
  required.splice(required.indexOf(additions.requiredAddition), 1);
  return result;
};
export const projectKnownTextMcpPredecessor = (source: unknown) => {
  const result = removeKnownTextMcpAdditions(source);
  if (
    createHash("sha256").update(JSON.stringify(result)).digest("hex") !==
    pin.semanticSha256
  ) {
    throw new Error("Unrelated known-text predecessor MCP drift");
  }
  return result;
};
