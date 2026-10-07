import additions from "./blend-mode-additions.json";
import pins from "./blend-mode-predecessor-pins.json";
import { projectOrderedEffectCatalogPredecessor } from "./ordered-effect-projection";
import {
  projectParameterizedMcpPredecessor,
  restoreParameterizedCatalogMarker,
} from "./parameterized-effect-projection";

type JsonRecord = Record<string, unknown>;
export const BLEND_PREDECESSOR_PINS: Record<string, string> = pins;
export const BLEND_VALUES = [
  "normal",
  "multiply",
  "screen",
  "overlay",
  "add",
  "darken",
  "lighten",
];
const record = (value: unknown): JsonRecord => {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Approved blend projection requires an object");
  }
  return value as JsonRecord;
};
const normalized = (value: unknown): unknown => {
  if (Array.isArray(value)) {
    return value.map(normalized);
  }
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, child]) => [key, normalized(child)])
    );
  }
  return value;
};
const exact = (value: unknown, expected: unknown, name: string) => {
  if (
    JSON.stringify(normalized(value)) !== JSON.stringify(normalized(expected))
  ) {
    throw new Error(`Incorrect approved blend addition: ${name}`);
  }
};
const parentAt = (source: JsonRecord, path: string[]) => {
  let parent = source;
  for (const key of path.slice(0, -1)) {
    parent = record(parent[key]);
  }
  const key = path.at(-1);
  if (!(key && Object.hasOwn(parent, key))) {
    throw new Error(`Missing approved blend addition: ${path.join(".")}`);
  }
  return { key, parent };
};
export const projectBlendCatalogPredecessor = (
  name: string,
  source: unknown
): JsonRecord => {
  if (!Object.hasOwn(BLEND_PREDECESSOR_PINS, name)) {
    throw new Error("Unapproved blend predecessor catalog");
  }
  const previous = restoreParameterizedCatalogMarker(source);
  const input = record(
    name === "extended-visual-animation-v1"
      ? projectOrderedEffectCatalogPredecessor(previous)
      : previous
  );
  exact(input.projectSchemaVersion, 35, "projectSchemaVersion");
  return { ...structuredClone(input), projectSchemaVersion: 34 };
};
const validateBlendNodes = (input: JsonRecord): void => {
  for (const entry of additions.entries) {
    const { key, parent } = parentAt(input, entry.path);
    exact(
      parent[key],
      entry.applicability === "ineligible-stored-normal"
        ? { const: "normal", type: "string" }
        : { enum: BLEND_VALUES, type: "string" },
      entry.path.join(".")
    );
  }
};
export const projectBlendMcpPredecessor = (source: unknown): JsonRecord => {
  // These nodes are unchanged by the successor projections. Reject malformed
  // controls before copying the entire catalog; valid input still executes every
  // predecessor proof, including the schema35 check below and full digest test.
  validateBlendNodes(record(source));
  return projectSchema35BlendMcpPredecessor(
    projectParameterizedMcpPredecessor(source)
  );
};
export const projectSchema35BlendMcpPredecessor = (
  source: unknown
): JsonRecord => {
  const input = record(source);
  const capabilities = input.capabilityIdentifiers;
  if (
    !Array.isArray(capabilities) ||
    capabilities.filter((value) => value === "blend_models_v1").length !== 1 ||
    capabilities.filter((value) => value === "blend_modes_v1").length !== 1
  ) {
    throw new Error("Incorrect blend capability multiplicity");
  }
  validateBlendNodes(input);
  for (const path of additions.literalPaths) {
    const { key, parent } = parentAt(input, path);
    exact(parent[key], 35, path.join("."));
  }
  const projected = record(structuredClone(input));
  projected.capabilityIdentifiers = capabilities.filter(
    (value) => value !== "blend_models_v1" && value !== "blend_modes_v1"
  );
  for (const entry of additions.entries) {
    const { key, parent } = parentAt(projected, entry.path);
    Reflect.deleteProperty(parent, key);
  }
  for (const path of additions.literalPaths) {
    const { key, parent } = parentAt(projected, path);
    parent[key] = 34;
  }
  return projected;
};
