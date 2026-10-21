import { deepStrictEqual } from "node:assert/strict";
import { createHash } from "node:crypto";
import contract from "../../../../contracts/narration-blocks-v1.json";
import additions from "./narration-blocks-mcp-additions.json";
import compactPins from "./narration-blocks-predecessor-compact-pins.json";
import pins from "./narration-blocks-predecessor-pins.json";

type RecordValue = Record<string, unknown>;
const at = (source: unknown, path: readonly string[]): unknown => {
  let value = source;
  for (const key of path) {
    if (!value || typeof value !== "object") {
      return undefined;
    }
    value = (value as RecordValue)[key];
  }
  return value;
};
const removeOnce = (raw: string, fragment: string): string => {
  const position = raw.indexOf(fragment);
  if (position < 0 || raw.indexOf(fragment, position + fragment.length) >= 0) {
    throw new Error("Narration block predecessor fragment missing or repeated");
  }
  return raw.slice(0, position) + raw.slice(position + fragment.length);
};
const projectOwnership = (result: RecordValue) => {
  const categories = at(result, ["categories"]) as RecordValue | undefined;
  if (!(categories && Object.hasOwn(categories, "narrationBlocks"))) {
    return;
  }
  deepStrictEqual(categories.narrationBlocks, contract.ownership);
  Reflect.deleteProperty(categories, "narrationBlocks");
};
const projectOperations = (result: RecordValue) => {
  const { operations } = result;
  if (
    !(
      Array.isArray(operations) &&
      operations.some((value) => contract.operations.includes(value))
    )
  ) {
    return;
  }
  for (const operation of contract.operations) {
    deepStrictEqual(
      operations.filter((value) => value === operation),
      [operation]
    );
  }
  deepStrictEqual(
    operations.filter((value) => contract.operations.includes(value)),
    contract.operations
  );
  result.operations = operations.filter(
    (value) => !contract.operations.includes(value)
  );
};
const projectMcp = (result: RecordValue) => {
  const properties = at(result, additions.projectStatePath) as
    | RecordValue
    | undefined;
  if (!(properties && Object.hasOwn(properties, "narrationManifests"))) {
    if (at(result, additions.versionPaths[0] ?? []) === 47) {
      throw new Error(
        "Schema47 MCP is missing required narration manifest metadata"
      );
    }
    return;
  }
  deepStrictEqual(properties.narrationManifests, additions.manifestArray);
  for (const path of additions.versionPaths) {
    deepStrictEqual(at(result, path), 47);
    const parent = at(result, path.slice(0, -1)) as RecordValue;
    const key = path.at(-1);
    if (!key) {
      throw new Error("Malformed version path");
    }
    parent[key] = 46;
  }
  const container = at(result, [
    "$defs",
    "ProjectGetStateOutputPropertiesProject",
  ]) as RecordValue;
  const { required } = container;
  if (!Array.isArray(required)) {
    throw new Error("Missing project required fields");
  }
  deepStrictEqual(
    required.filter((value) => value === "narrationManifests"),
    ["narrationManifests"]
  );
  container.required = required.filter(
    (value) => value !== "narrationManifests"
  );
  Reflect.deleteProperty(properties, "narrationManifests");
};
export const removeNarrationBlockAdditions = <T>(
  source: T,
  name: string,
  copy = true
): T => {
  if (!source || typeof source !== "object") {
    return source;
  }
  const result = copy ? structuredClone(source) : source;
  const object = result as RecordValue;
  switch (name) {
    case "contract-ownership-v1":
      projectOwnership(object);
      break;
    case "headless-protocol-v1":
      projectOperations(object);
      break;
    case "mcp-surface-v1":
      projectMcp(object);
      break;
    default:
      if (object.projectSchemaVersion === 47) {
        object.projectSchemaVersion = 46;
      }
  }
  return result;
};
const rawMcp = (input: string, compact: boolean) => {
  let restored = removeOnce(
    input,
    compact
      ? `"narrationManifests":${JSON.stringify(additions.manifestArray)},`
      : additions.rawPropertyFragment
  );
  restored = removeOnce(
    restored,
    compact ? '"narrationManifests",' : additions.rawRequiredFragment
  );
  if ((restored.match(/"const": ?47/g) ?? []).length !== 2) {
    throw new Error("Unexpected narration MCP schema marker drift");
  }
  return restored.replaceAll(
    compact ? '"const":47' : '"const": 47',
    compact ? '"const":46' : '"const": 46'
  );
};
const rawOwnership = (input: string, compact: boolean) => {
  const fragment = JSON.stringify(
    { narrationBlocks: contract.ownership },
    null,
    2
  )
    .split("\n")
    .slice(1, -1)
    .map((line) => `  ${line}`)
    .join("\n");
  return removeOnce(
    input,
    compact
      ? `${JSON.stringify({ narrationBlocks: contract.ownership }).slice(1, -1)},`
      : `${fragment},\n`
  );
};
const rawOperations = (input: string, compact: boolean) => {
  let restored = input;
  for (const operation of contract.operations) {
    restored = removeOnce(
      restored,
      compact ? `${JSON.stringify(operation)},` : `    "${operation}",\n`
    );
  }
  return restored;
};
const rawMarker = (input: string, compact: boolean) => {
  const marker = compact
    ? '"projectSchemaVersion":47'
    : '"projectSchemaVersion": 47';
  if (input.indexOf(marker) !== input.lastIndexOf(marker)) {
    throw new Error("Repeated narration schema marker");
  }
  const restored = input.replace(
    marker,
    compact ? '"projectSchemaVersion":46' : '"projectSchemaVersion": 46'
  );
  const digest = createHash("sha256").update(restored).digest("hex");
  if (!Object.values(compact ? compactPins : pins).includes(digest)) {
    throw new Error(
      "Unrelated narration schema marker predecessor contract drift"
    );
  }
  return restored;
};
// Exact owned-fragment inverse; dedicated tests pin complete predecessor hashes.
// Unrelated data stays present so historical parity tests can detect its drift.
export const restoreNarrationBlocksRaw = (input: string): string => {
  const source = JSON.parse(input);
  const compact = !input.includes("\n");
  let name: string;
  let restore: (raw: string, isCompact: boolean) => string;
  if (source.toolDefinitions) {
    name = "mcp-surface-v1";
    restore = rawMcp;
  } else if (source.categories?.narrationBlocks) {
    name = "contract-ownership-v1";
    restore = rawOwnership;
  } else if (
    Array.isArray(source.operations) &&
    source.operations.some((value: string) =>
      contract.operations.includes(value)
    )
  ) {
    name = "headless-protocol-v1";
    restore = rawOperations;
  } else if (source.projectSchemaVersion === 47) {
    return rawMarker(input, compact);
  } else {
    return input;
  }
  const projected = removeNarrationBlockAdditions(source, name);
  if (JSON.stringify(projected) === JSON.stringify(source)) {
    return input;
  }
  const restored = restore(input, compact);
  deepStrictEqual(JSON.parse(restored), projected);
  return restored;
};
