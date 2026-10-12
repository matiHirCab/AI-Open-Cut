import { createHash } from "node:crypto";
import catalog from "../../../../contracts/solid-graphic-color-slots-v1.json";
import { restoreFfmpegSpawnGraphicRaw } from "./ffmpeg-spawn-graphic-projection";
import changes from "./solid-graphic-color-slots-mcp-additions.json";
import category from "./solid-graphic-color-slots-ownership-addition.json";
import pins from "./solid-graphic-color-slots-predecessor-pins.json";

const record = (value: unknown): Record<string, unknown> => {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Malformed graphic color projection");
  }
  return value as Record<string, unknown>;
};
const at = (source: unknown, path: string[]): unknown => {
  let value = source;
  for (const key of path) {
    if (!value || typeof value !== "object") {
      return undefined;
    }
    value = (value as Record<string, unknown>)[key];
  }
  return value;
};
const equal = (a: unknown, b: unknown) =>
  JSON.stringify(a) === JSON.stringify(b);

export const restoreSolidGraphicCatalogMarker = <T>(source: T): T => {
  const result = structuredClone(source);
  const value = record(result);
  if (value.projectSchemaVersion === catalog.projectSchemaVersion) {
    value.projectSchemaVersion = 44;
  }
  return result;
};

export const removeSolidGraphicMcpAdditions = <T>(source: T): T => {
  const result = structuredClone(source);
  if (!changes.some(({ path, after }) => equal(at(result, path), after))) {
    return result;
  }
  for (const { path, before, after } of changes) {
    if (path.length === 1 && path[0] === "capabilityIdentifiers") {
      const values = at(result, path);
      if (
        !Array.isArray(values) ||
        values.filter((v) => v === catalog.capability).length !== 1
      ) {
        throw new Error("Incorrect graphic color capability multiplicity");
      }
      values.splice(values.indexOf(catalog.capability), 1);
      continue;
    }
    if (!equal(at(result, path), after)) {
      throw new Error("Incorrect graphic color MCP addition");
    }
    const parent = record(at(result, path.slice(0, -1)));
    const key = path.at(-1);
    if (!key) {
      throw new Error("Empty graphic color projection path");
    }
    parent[key] = structuredClone(before);
  }
  return result;
};
export const removeSolidGraphicHeadlessAddition = <T>(source: T): T => {
  const result = structuredClone(source);
  const capabilities = at(result, ["status", "editorCapabilities"]);
  if (Array.isArray(capabilities)) {
    const matches = capabilities.filter((v) => v === catalog.capability);
    if (matches.length > 1) {
      throw new Error("Duplicate graphic color capability");
    }
    const index = capabilities.indexOf(catalog.capability);
    if (index >= 0) {
      capabilities.splice(index, 1);
    }
  }
  return result;
};
export const removeSolidGraphicOwnershipAddition = <T>(source: T): T => {
  const result = structuredClone(source);
  const categories = record(record(result).categories);
  if (Object.hasOwn(categories, "solidGraphicColorSlots")) {
    if (!equal(categories.solidGraphicColorSlots, category)) {
      throw new Error("Incorrect graphic color ownership addition");
    }
    Reflect.deleteProperty(categories, "solidGraphicColorSlots");
  }
  return result;
};
export const restoreSolidGraphicRaw = (source: string): string => {
  const value = record(JSON.parse(source));
  if (
    at(value, ["$defs", "FfmpegSpawnDiagnostic"]) ||
    at(value, ["categories", "ffmpegSpawnDiagnostics"]) ||
    value.ffmpegSpawnError
  ) {
    return restoreFfmpegSpawnGraphicRaw(source);
  }
  let path: keyof typeof pins.rawSha256;
  let projected: unknown;
  if (value.toolDefinitions) {
    path = "contracts/mcp-surface-v1.json";
    projected = removeSolidGraphicMcpAdditions(value);
  } else if (at(value, ["status", "editorCapabilities"])) {
    path = "contracts/headless-protocol-v1.json";
    projected = removeSolidGraphicHeadlessAddition(value);
  } else if (value.categories) {
    path = "contracts/contract-ownership-v1.json";
    projected = removeSolidGraphicOwnershipAddition(value);
  } else if (value.capability === "typed_template_slots") {
    path = "contracts/template-slots-v1.json";
    projected = structuredClone(value);
    const { properties } = record(projected);
    if (!Array.isArray(properties)) {
      throw new Error("Malformed graphic slot properties");
    }
    record(projected).properties = properties.filter(
      (v) => !catalog.properties.includes(v)
    );
  } else if (value.projectSchemaVersion === catalog.projectSchemaVersion) {
    // Undo exactly the owned current marker while retaining every authored raw byte.
    const raw = source.replace(
      '"projectSchemaVersion": 45',
      '"projectSchemaVersion": 44'
    );
    const digest = createHash("sha256").update(raw).digest("hex");
    const marker = pins.markerCatalogs.find(
      (name) => pins.rawSha256[name as keyof typeof pins.rawSha256] === digest
    );
    if (
      !(
        marker &&
        equal(JSON.parse(raw), restoreSolidGraphicCatalogMarker(value))
      )
    ) {
      throw new Error("Graphic color predecessor contract drift");
    }
    return raw;
  } else {
    return source;
  }
  if (equal(projected, value)) {
    return source;
  }
  // The headless predecessor intentionally retains authored .0 numeric
  // lexemes. Remove only the exact appended capability and preserve its raw
  // bytes; require semantic equality before using the captured raw digest.
  const raw =
    path === "contracts/headless-protocol-v1.json"
      ? source.replace(',\n      "solid_graphic_color_slots_v1"', "")
      : `${JSON.stringify(projected, null, 2)}\n`;
  if (!equal(JSON.parse(raw), projected)) {
    throw new Error("Graphic color predecessor contract drift");
  }
  if (createHash("sha256").update(raw).digest("hex") !== pins.rawSha256[path]) {
    throw new Error("Graphic color predecessor contract drift");
  }
  return raw;
};
