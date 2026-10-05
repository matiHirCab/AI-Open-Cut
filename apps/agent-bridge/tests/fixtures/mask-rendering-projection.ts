// Approved issue51 exact storage changes; matching fields elsewhere are untouched.
type JsonRecord = Record<string, unknown>;
export const MASK_PROPERTIES = [
  "mask.path_points",
  "mask.paint_color",
  "mask.gradient_stops",
  "mask.feather_px",
  "mask.expansion_px",
  "mask.transform.position_x",
  "mask.transform.position_y",
  "mask.transform.scale_x",
  "mask.transform.scale_y",
  "mask.transform.anchor_x",
  "mask.transform.anchor_y",
  "mask.transform.rotation_deg",
  "mask.transform.skew_x_deg",
  "mask.transform.skew_y_deg",
  "mask.transform.opacity",
] as const;
const MASK_PROPERTY_BOUNDS: Record<string, [string, string, number, number]> = {
  "mask.expansion_px": ["scalar", "minimum", -128, 128],
  "mask.feather_px": ["scalar", "minimum", 0, 128],
  "mask.gradient_stops": ["gradient_stops", "minimum", 0, 1],
  "mask.paint_color": ["rgba", "minimum", 0, 1],
  "mask.path_points": ["path_points", "minimum", -1_000_000, 1_000_000],
  "mask.transform.anchor_x": ["scalar", "minimum", 0, 1],
  "mask.transform.anchor_y": ["scalar", "minimum", 0, 1],
  "mask.transform.opacity": ["scalar", "minimum", 0, 1],
  "mask.transform.position_x": ["scalar", "minimum", -1_000_000, 1_000_000],
  "mask.transform.position_y": ["scalar", "minimum", -1_000_000, 1_000_000],
  "mask.transform.rotation_deg": ["scalar", "minimum", -36_000, 36_000],
  "mask.transform.scale_x": ["scalar", "minimumExclusive", 0, 100],
  "mask.transform.scale_y": ["scalar", "minimumExclusive", 0, 100],
  "mask.transform.skew_x_deg": ["scalar", "minimum", -80, 80],
  "mask.transform.skew_y_deg": ["scalar", "minimum", -80, 80],
};
const normalized = (value: unknown): unknown => {
  if (Array.isArray(value)) {
    return value.map(normalized);
  }
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, entry]) => [key, normalized(entry)])
    );
  }
  return value;
};
const assertExact = (value: unknown, expected: unknown, name: string) => {
  if (
    JSON.stringify(normalized(value)) !== JSON.stringify(normalized(expected))
  ) {
    throw new Error(`Incorrect approved addition: ${name}`);
  }
};
export const MASK_RENDERING_PREDECESSOR_PINS: Record<string, string> = {
  "animation-channels-v1":
    "be551a8a32a265b29e305afbb86f21abd13544b571146da6814709547cd1c590",
  "animation-presets-v1":
    "73c4024a0dabf1a8780a2315d0b3f82b37d3376ad04c3cc06ca0382df20fd86f",
  "extended-visual-animation-v1":
    "015ffd5d1950e1039014455d0514994afcbe95af6b48013c5a52e290e9a90321",
  "inherited-animation-timing-v1":
    "2bd375715f3e1dd29daf94f001acd0f01a49a0e61f29b2ad4b9c4f256d11f7b6",
  "initial-motion-preset-pack-v1":
    "bacb44d097dc18a96b0e07f50741da6993bfdef6da24ce769cb8827b617b1ff2",
  "mask-models-v1":
    "cbb2943e382c0a4c542f66c640e68d8fb5a15a5f589ab2bf3778570b30a3e972",
  "motion-blur-sampling-v1":
    "6f109c62b37cf3d29ae18f42619304dd07038621a4183904f2557b8290fac9ca",
};
const record = (value: unknown): JsonRecord => {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Approved projection requires an object");
  }
  return value as JsonRecord;
};
const at = (source: JsonRecord, path: string[]): JsonRecord => {
  let node = source;
  for (const key of path) {
    node = record(node[key]);
  }
  return node;
};
const without = (source: JsonRecord, keys: readonly string[]): JsonRecord => {
  for (const key of keys) {
    if (!Object.hasOwn(source, key)) {
      throw new Error(`Missing approved addition: ${key}`);
    }
  }
  return Object.fromEntries(
    Object.entries(source).filter(([key]) => !keys.includes(key))
  );
};
export const projectMaskRenderingCatalogPredecessor = (
  name: string,
  source: unknown
): JsonRecord => {
  if (!Object.hasOwn(MASK_RENDERING_PREDECESSOR_PINS, name)) {
    throw new Error("Unapproved catalog");
  }
  const projected = record(structuredClone(source));
  if (projected.projectSchemaVersion !== 33) {
    throw new Error("Current marker must be 33");
  }
  projected.projectSchemaVersion = 32;
  if (name === "animation-channels-v1") {
    const active = record(projected.active);
    for (const [
      property,
      [valueType, minimumKey, minimum, maximum],
    ] of Object.entries(MASK_PROPERTY_BOUNDS)) {
      assertExact(
        active[property],
        {
          activation: "active",
          target: "mask_scoped",
          valueType,
          [minimumKey]: minimum,
          maximum,
        },
        property
      );
    }
    assertExact(
      projected.maskTarget,
      {
        activationSchemaVersion: 33,
        applicabilityContract: "contracts/mask-rendering-v1.json",
        idScope: "owning_item",
        kind: "mask",
        maxIdBytes: 128,
        scope: ["root", "component:<definition-id>"],
      },
      "maskTarget"
    );
    projected.active = without(record(projected.active), MASK_PROPERTIES);
    return without(projected, ["maskTarget"]);
  }
  if (name === "mask-models-v1") {
    if (
      projected.status !== "authoring_with_active_rendering_contract" ||
      projected.rendererStage !== "contracts/mask-rendering-v1.json"
    ) {
      throw new Error("Incorrect active model annotation");
    }
    projected.status = "authoring_metadata_only";
    projected.rendererStage =
      "identity_until_separately_approved_mask_rendering";
    const semantics = record(projected.semantics);
    if (
      semantics.order !==
      "geometric_fill, signed_expansion, original_coordinate_paint_channel, feather, mask_affine_opacity, inversion, ordered_combination"
    ) {
      throw new Error("Incorrect approved mask order");
    }
    if (semantics.timing !== "static_and_mask_targeted_animation") {
      throw new Error("Incorrect approved mask timing");
    }
    semantics.timing = "static_metadata_no_animation_targets";
    semantics.order =
      "path_fill_paint, signed_expansion, feather, mask_affine_opacity, inversion, ordered_combination";
  }
  return projected;
};
export const projectMaskRenderingMcpPredecessor = (
  source: unknown
): JsonRecord => {
  const projected = record(structuredClone(source));
  const capabilities = projected.capabilityIdentifiers;
  if (!Array.isArray(capabilities)) {
    throw new Error("Invalid capability list");
  }
  for (const name of ["mask_animation_v1", "mask_rendering_v1"]) {
    if (capabilities.filter((value) => value === name).length !== 1) {
      throw new Error(`Incorrect capability multiplicity: ${name}`);
    }
  }
  projected.capabilityIdentifiers = capabilities.filter(
    (value) => value !== "mask_animation_v1" && value !== "mask_rendering_v1"
  );
  const properties = at(projected, [
    "$defs",
    "ComponentInstancePropertiesAnimationChannelsItemsProperties",
  ]);
  const property = record(properties.property);
  const names = property.enum;
  if (!Array.isArray(names)) {
    throw new Error("Invalid property enum");
  }
  for (const name of MASK_PROPERTIES) {
    if (names.filter((value) => value === name).length !== 1) {
      throw new Error(`Missing or duplicated property: ${name}`);
    }
  }
  property.enum = names.filter(
    (value) =>
      !MASK_PROPERTIES.includes(value as (typeof MASK_PROPERTIES)[number])
  );
  const kind = at(properties, ["target", "properties", "kind"]);
  if (
    !Array.isArray(kind.enum) ||
    kind.enum.filter((value) => value === "mask").length !== 1
  ) {
    throw new Error("Incorrect mask target multiplicity");
  }
  kind.enum = kind.enum.filter((value) => value !== "mask");
  for (const path of [
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
  ]) {
    const schema = at(projected, path);
    if (schema.const !== 33) {
      throw new Error("Incorrect current MCP version");
    }
    schema.const = 32;
  }
  return projected;
};
