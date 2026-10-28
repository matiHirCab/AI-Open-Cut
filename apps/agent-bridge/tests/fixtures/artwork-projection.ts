import { deepStrictEqual } from "node:assert/strict";
import additions from "./artwork-additions.json";

type RecordValue = Record<string | number, unknown>;
type Path = (string | number)[];
type Catalog = keyof typeof additions;
interface Change {
  after: unknown;
  before?: unknown;
  path: Path;
}
const at = (source: unknown, path: Path): unknown => {
  let value = source;
  for (const key of path) {
    if (!value || typeof value !== "object") {
      return undefined;
    }
    value = (value as RecordValue)[key];
  }
  return value;
};
const same = (a: unknown, b: unknown): boolean => {
  try {
    deepStrictEqual(a, b);
    return true;
  } catch {
    return false;
  }
};

/** Invert only the independently authored AF1 changes, preserving peer additions. */
export const removeArtworkAdditions = <T>(
  source: T,
  name: Catalog,
  copy = true
): T => {
  const result = copy ? structuredClone(source) : source;
  const changes: Change[] = additions[name];
  let present: boolean | undefined;
  if (name === "mcp-surface-v1") {
    present = changes.some(
      (change) =>
        at(result, change.path) !== undefined &&
        (change.before === undefined ||
          same(at(result, change.path), change.after))
    );
  } else if (name === "headless-protocol-v1") {
    present = (at(result, ["operations"]) as unknown[] | undefined)?.some(
      (value) => value === "commit_artwork" || value === "prepare_artwork"
    );
  } else {
    present = at(result, ["categories", "artworkPreparation"]) !== undefined;
  }
  if (!present) {
    return result;
  }
  if (name === "headless-protocol-v1") {
    const operations = at(result, ["operations"]) as unknown[];
    for (const operation of ["commit_artwork", "prepare_artwork"]) {
      if (operations.filter((value) => value === operation).length !== 1) {
        throw new Error("Incorrect artwork operation multiplicity");
      }
      operations.splice(operations.indexOf(operation), 1);
    }
    return result;
  }
  for (const change of changes) {
    if (!same(at(result, change.path), change.after)) {
      throw new Error(`Incorrect artwork addition: ${change.path.join(".")}`);
    }
    const parent = at(result, change.path.slice(0, -1)) as RecordValue;
    const key = change.path.at(-1);
    if (key === undefined || !parent) {
      throw new Error("Malformed artwork projection path");
    }
    if (Object.hasOwn(change, "before")) {
      parent[key] = structuredClone(change.before);
    } else {
      Reflect.deleteProperty(parent, key);
    }
  }
  return result;
};

export const restoreArtworkCatalogMarker = <T>(source: T): T => {
  const result = structuredClone(source);
  if (at(result, ["projectSchemaVersion"]) === 47) {
    (result as RecordValue).projectSchemaVersion = 46;
  }
  return result;
};

export const restoreArtworkRaw = (input: string): string => {
  const value = JSON.parse(input);
  let projected: unknown;
  let name: Catalog | undefined;
  if (value.projectSchemaVersion === 47) {
    projected = restoreArtworkCatalogMarker(value);
  } else if (value.toolDefinitions) {
    name = "mcp-surface-v1";
  } else if (value.operations) {
    name = "headless-protocol-v1";
  } else if (value.categories) {
    name = "contract-ownership-v1";
  }
  projected = name
    ? removeArtworkAdditions(value, name)
    : restoreArtworkCatalogMarker(value);
  if (same(value, projected)) {
    return input;
  }
  let raw: string;
  if (name === "headless-protocol-v1" && input.includes("\n")) {
    raw = input;
    for (const operation of ["commit_artwork", "prepare_artwork"]) {
      raw = raw
        .replace(`    "${operation}",\r\n`, "")
        .replace(`    "${operation}",\n`, "");
    }
  } else if (name) {
    raw =
      input === JSON.stringify(value)
        ? JSON.stringify(projected)
        : `${JSON.stringify(projected, null, 2)}\n`;
  } else {
    raw = input.replace(
      '"projectSchemaVersion": 47',
      '"projectSchemaVersion": 46'
    );
    if (raw === input) {
      raw = input.replace(
        '"projectSchemaVersion":47',
        '"projectSchemaVersion":46'
      );
    }
  }
  deepStrictEqual(JSON.parse(raw), projected);
  return raw;
};
