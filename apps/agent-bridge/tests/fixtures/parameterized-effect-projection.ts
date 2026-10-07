import {
  removeGroupMcpAdditions,
  restoreGroupCatalogMarker,
} from "./group-compositing-projection";
import { orderedEffectDigest } from "./ordered-effect-projection";
import addition from "./parameterized-effect-addition.json";
import pins from "./parameterized-effect-predecessor-pins.json";

type RecordValue = Record<string, unknown>;
const object = (value: unknown): RecordValue => {
  if (!(value && typeof value === "object" && !Array.isArray(value))) {
    throw new Error("Incorrect approved parameterized-effect object");
  }
  return value as RecordValue;
};
export const PARAMETERIZED_PREDECESSOR_PINS: Record<string, string> = pins;
// This marker-only step composes historical negative controls. The strict
// public predecessor proof below separately pins every complete current catalog.
export const restoreSchema36ParameterizedCatalogMarker = (
  source: unknown
): RecordValue => {
  const input = object(source);
  if (input.projectSchemaVersion !== 36) {
    throw new Error(
      "Incorrect approved parameterized-effect schema marker: Unrelated verified predecessor"
    );
  }
  return { ...structuredClone(input), projectSchemaVersion: 35 };
};
export const projectSchema36ParameterizedCatalogPredecessor = (
  name: string,
  source: unknown
): RecordValue => {
  if (!Object.hasOwn(PARAMETERIZED_PREDECESSOR_PINS, name)) {
    throw new Error("Unapproved parameterized-effect predecessor catalog");
  }
  const projected = restoreSchema36ParameterizedCatalogMarker(source);
  if (orderedEffectDigest(projected) !== PARAMETERIZED_PREDECESSOR_PINS[name]) {
    throw new Error("Unrelated verified predecessor catalog drift");
  }
  return projected;
};
const paths = [
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
const parentAt = (source: RecordValue, path: string[]) => {
  let parent = source;
  for (const key of path.slice(0, -1)) {
    parent = object(parent[key]);
  }
  const key = path.at(-1);
  if (!(key && Object.hasOwn(parent, key))) {
    throw new Error("Missing approved parameterized-effect literal");
  }
  return { key, parent };
};
export const projectSchema36ParameterizedMcpPredecessor = (
  source: unknown
): RecordValue => {
  const input = object(source);
  const capabilities = input.capabilityIdentifiers;
  const identifiers = [
    "parameterized_effect_models_v1",
    "parameterized_effects_v1",
  ];
  if (
    !Array.isArray(capabilities) ||
    identifiers.some(
      (id) => capabilities.filter((value) => value === id).length !== 1
    )
  ) {
    throw new Error(
      "Incorrect approved parameterized-effect capability multiplicity"
    );
  }
  const definition = object(object(input.$defs).ExtendedVisualEffect);
  const variants = definition.oneOf;
  if (
    !Array.isArray(variants) ||
    variants.length !== 5 ||
    orderedEffectDigest(variants[4]) !== orderedEffectDigest(addition)
  ) {
    throw new Error("Incorrect approved parameterized-effect variant");
  }
  for (const path of paths) {
    const { parent, key } = parentAt(input, path);
    if (parent[key] !== 36) {
      throw new Error("Incorrect approved parameterized-effect literal");
    }
  }
  const projected = object(structuredClone(input));
  projected.capabilityIdentifiers = capabilities.filter(
    (value) => !identifiers.includes(value)
  );
  object(object(projected.$defs).ExtendedVisualEffect).oneOf = structuredClone(
    variants.slice(0, 4)
  );
  for (const path of paths) {
    const { parent, key } = parentAt(projected, path);
    parent[key] = 35;
  }
  return projected;
};

// Current37 must first pass the issue56 exact additions/marker proof. Historical
// issue55 logic and pins remain unchanged and separately callable on verified36.
export const restoreParameterizedCatalogMarker = (
  source: unknown
): RecordValue =>
  restoreSchema36ParameterizedCatalogMarker(restoreGroupCatalogMarker(source));
export const projectParameterizedCatalogPredecessor = (
  name: string,
  source: unknown
): RecordValue =>
  projectSchema36ParameterizedCatalogPredecessor(
    name,
    restoreGroupCatalogMarker(source)
  );
export const projectParameterizedMcpPredecessor = (
  source: unknown
): RecordValue =>
  projectSchema36ParameterizedMcpPredecessor(removeGroupMcpAdditions(source));
