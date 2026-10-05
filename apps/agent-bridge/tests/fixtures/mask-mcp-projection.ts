// Approved issue #50 catalog storage locations; never search recursively for matching keys.
// Expansion after this exact projection must reproduce the verified #49 catalog digest.
type JsonRecord = Record<string, unknown>;
const MASK_PROPERTY_LOCATIONS: string[][] = [
  ["toolDefinitions", "timeline_update_item", "inputSchema", "properties"],
  [
    "$defs",
    "ComponentCreatePropertiesTracksItemsPropertiesItemsItemsOneOfCaptionProperties",
  ],
  [
    "$defs",
    "ComponentCreatePropertiesTracksItemsPropertiesItemsItemsOneOfTextProperties",
  ],
  ["$defs", "InputPropertiesOperationsItemsOneOfUpdateItemProperties"],
  [
    "$defs",
    "InputPropertiesTracksItemsPropertiesItemsItemsOneOfCaptionProperties",
  ],
  [
    "$defs",
    "InputPropertiesTracksItemsPropertiesItemsItemsOneOfComponentInstanceProperties",
  ],
  [
    "$defs",
    "InputPropertiesTracksItemsPropertiesItemsItemsOneOfGridProperties",
  ],
  [
    "$defs",
    "InputPropertiesTracksItemsPropertiesItemsItemsOneOfGroupProperties",
  ],
  [
    "$defs",
    "InputPropertiesTracksItemsPropertiesItemsItemsOneOfMediaProperties",
  ],
  [
    "$defs",
    "InputPropertiesTracksItemsPropertiesItemsItemsOneOfRectangleProperties",
  ],
  [
    "$defs",
    "InputPropertiesTracksItemsPropertiesItemsItemsOneOfShapeProperties",
  ],
  [
    "$defs",
    "InputPropertiesTracksItemsPropertiesItemsItemsOneOfSolidColorProperties",
  ],
  ["$defs", "InputPropertiesTracksItemsPropertiesItemsItemsOneOfSvgProperties"],
  [
    "$defs",
    "InputPropertiesTracksItemsPropertiesItemsItemsOneOfTextProperties",
  ],
  [
    "$defs",
    "InputPropertiesTracksItemsPropertiesItemsItemsOneOfTransitionProperties",
  ],
  [
    "$defs",
    "ItemsPropertiesTracksItemsPropertiesItemsItemsOneOfComponentInstanceProperties",
  ],
  ["$defs", "OutputPropertiesOperationsItemsOneOfUpdateItemProperties"],
  [
    "$defs",
    "ProjectPropertiesTracksItemsPropertiesItemsItemsOneOfCaptionProperties",
  ],
  [
    "$defs",
    "ProjectPropertiesTracksItemsPropertiesItemsItemsOneOfComponentInstanceProperties",
  ],
  [
    "$defs",
    "ProjectPropertiesTracksItemsPropertiesItemsItemsOneOfGridProperties",
  ],
  [
    "$defs",
    "ProjectPropertiesTracksItemsPropertiesItemsItemsOneOfGroupProperties",
  ],
  [
    "$defs",
    "ProjectPropertiesTracksItemsPropertiesItemsItemsOneOfMediaProperties",
  ],
  [
    "$defs",
    "ProjectPropertiesTracksItemsPropertiesItemsItemsOneOfRectangleProperties",
  ],
  [
    "$defs",
    "ProjectPropertiesTracksItemsPropertiesItemsItemsOneOfShapeProperties",
  ],
  [
    "$defs",
    "ProjectPropertiesTracksItemsPropertiesItemsItemsOneOfSolidColorProperties",
  ],
  [
    "$defs",
    "ProjectPropertiesTracksItemsPropertiesItemsItemsOneOfSvgProperties",
  ],
  [
    "$defs",
    "ProjectPropertiesTracksItemsPropertiesItemsItemsOneOfTextProperties",
  ],
  [
    "$defs",
    "ProjectPropertiesTracksItemsPropertiesItemsItemsOneOfTransitionProperties",
  ],
  ["$defs", "TracksItemsPropertiesItemsItemsOneOfRepeaterProperties"],
];
const SCHEMA_VERSION_LOCATIONS: string[][] = [
  [
    "toolDefinitions",
    "editor_get_status",
    "outputSchema",
    "properties",
    "projectSchemaVersion",
  ],
  [
    "$defs",
    "ProjectGetStateOutputPropertiesProjectProperties",
    "schemaVersion",
  ],
];
const at = (source: JsonRecord, path: string[]): JsonRecord => {
  let node: unknown = source;
  for (const key of path) {
    if (node === null || typeof node !== "object" || Array.isArray(node)) {
      throw new Error(`Invalid approved MCP location: ${path.join("/")}`);
    }
    node = (node as JsonRecord)[key];
  }
  if (node === null || typeof node !== "object" || Array.isArray(node)) {
    throw new Error(`Missing approved MCP location: ${path.join("/")}`);
  }
  return node as JsonRecord;
};
export const projectMaskMcpPredecessor = (source: unknown): JsonRecord => {
  const projected = structuredClone(source) as JsonRecord;
  for (const path of MASK_PROPERTY_LOCATIONS) {
    const properties = at(projected, path);
    if (!Object.hasOwn(properties, "masks")) {
      throw new Error(`Missing approved masks field: ${path.join("/")}`);
    }
    // biome-ignore lint/performance/noDelete: predecessor projection must remove the key completely.
    delete properties.masks;
  }
  for (const path of SCHEMA_VERSION_LOCATIONS) {
    const version = at(projected, path);
    if (version.const !== 32) {
      throw new Error(`Incorrect schema version at ${path.join("/")}`);
    }
    version.const = 31;
  }
  const definitions = at(projected, ["$defs"]);
  if (!Object.hasOwn(definitions, "MaskModelRecord")) {
    throw new Error("Missing approved mask definition");
  }
  // biome-ignore lint/performance/noDelete: predecessor projection must remove the definition completely.
  delete definitions.MaskModelRecord;
  const capabilities = projected.capabilityIdentifiers;
  if (
    !Array.isArray(capabilities) ||
    capabilities.filter((value) => value === "mask_models_v1").length !== 1
  ) {
    throw new Error("Incorrect mask capability multiplicity");
  }
  projected.capabilityIdentifiers = capabilities.filter(
    (value) => value !== "mask_models_v1"
  );
  return projected;
};
