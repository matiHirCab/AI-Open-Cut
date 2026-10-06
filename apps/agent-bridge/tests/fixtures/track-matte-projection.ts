import additions from "./track-matte-additions.json";

type JsonRecord = Record<string, unknown>;
export const TRACK_MATTE_PREDECESSOR_PINS: Record<string, string> = {
  "animation-channels-v1":
    "c4758e1b0d3417a590fc9c15a2b4f2b3dd56df2924a9afb9abce6c85f42e21a5",
  "animation-presets-v1":
    "257af56dde84a19b6b59b12e68673c7487bbd96131e1ccadf36900f37030df81",
  "extended-visual-animation-v1":
    "cc101cfc95359306f4208f645c2144bd0e29b06eae262f92ae8adcb9613eb857",
  "inherited-animation-timing-v1":
    "d5b7e1f79c1107a85221a8b016881699bbd8257e3ed81504f3f74e7e87eecc5f",
  "initial-motion-preset-pack-v1":
    "791fe0e2c97a9f607d7d69f93a7c198249d672e90653a0ae9e0cdd114f5b876a",
  "mask-models-v1":
    "ec58d0c4437020fa4df539c9390c5854ca5409850f748917de1a6bfcf75757ef",
  "mask-rendering-v1":
    "dafc860180d583b00f2976275630b15a29dc83bedd0b2f7d08eedb3887826a5d",
  "motion-blur-sampling-v1":
    "0435ce749017c161eb7ee7fa75fe7844ce66f24b25354acd59eeef83d7fe8137",
};
export const MATTE_REFERENCE_JSON = {
  additionalProperties: false,
  properties: {
    channel: { enum: ["alpha", "luma"], type: "string" },
    sourceId: { maxLength: 128, minLength: 1, type: "string" },
  },
  required: ["channel", "sourceId"],
  type: "object",
};
const record = (value: unknown): JsonRecord => {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Approved matte projection requires an object");
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
    throw new Error(`Incorrect approved matte addition: ${name}`);
  }
};
const parentAt = (source: JsonRecord, path: string[]) => {
  let parent = source;
  for (const key of path.slice(0, -1)) {
    parent = record(parent[key]);
  }
  const key = path.at(-1);
  if (!(key && Object.hasOwn(parent, key))) {
    throw new Error(`Missing approved matte addition: ${path.join(".")}`);
  }
  return { key, parent };
};
export const projectTrackMatteCatalogPredecessor = (
  name: string,
  source: unknown
): JsonRecord => {
  if (!Object.hasOwn(TRACK_MATTE_PREDECESSOR_PINS, name)) {
    throw new Error("Unapproved matte predecessor catalog");
  }
  const projected = record(structuredClone(source));
  exact(projected.projectSchemaVersion, 34, "current projectSchemaVersion");
  projected.projectSchemaVersion = 33;
  return projected;
};
export const projectTrackMatteMcpPredecessor = (
  source: unknown
): JsonRecord => {
  const input = record(source);
  const definitions = record(input.$defs);
  exact(
    definitions.TrackMatteReference,
    MATTE_REFERENCE_JSON,
    "TrackMatteReference definition"
  );
  const capabilities = input.capabilityIdentifiers;
  if (!Array.isArray(capabilities)) {
    throw new Error("Malformed capability list");
  }
  for (const capability of ["matte_models_v1", "track_mattes_v1"]) {
    exact(
      capabilities.filter((entry) => entry === capability).length,
      1,
      `${capability} multiplicity`
    );
  }
  exact(additions.entries.length, 58, "field whitelist cardinality");
  for (const entry of additions.entries) {
    const { key, parent } = parentAt(input, entry.path);
    let expected: unknown;
    if (key === "matte") {
      if (entry.applicability === "ineligible-stored-visual") {
        expected = { not: {} };
      } else if (entry.applicability === "shared-update-item") {
        expected = {
          anyOf: [{ $ref: "#/$defs/TrackMatteReference" }, { type: "null" }],
        };
      } else {
        expected = { $ref: "#/$defs/TrackMatteReference" };
      }
    } else {
      expected =
        entry.applicability === "ineligible-stored-visual"
          ? { const: false, type: "boolean" }
          : { type: "boolean" };
    }
    exact(parent[key], expected, entry.path.join("."));
  }
  for (const path of additions.literalPaths) {
    const { key, parent } = parentAt(input, path);
    exact(parent[key], 34, path.join("."));
  }
  // Reject malformed additions before copying the complete compact catalog.
  // Only a fully checked input may be projected; neither path mutates input.
  const projected = record(structuredClone(input));
  projected.capabilityIdentifiers = capabilities.filter(
    (entry) => entry !== "matte_models_v1" && entry !== "track_mattes_v1"
  );
  for (const entry of additions.entries) {
    const { key, parent } = parentAt(projected, entry.path);
    Reflect.deleteProperty(parent, key);
  }
  Reflect.deleteProperty(record(projected.$defs), "TrackMatteReference");
  for (const path of additions.literalPaths) {
    const { key, parent } = parentAt(projected, path);
    parent[key] = 33;
  }
  return projected;
};
