import { createHash } from "node:crypto";
import {
  removeChannelPackAdditions,
  restoreChannelPackRaw,
} from "./channel-packs-projection";
import {
  removeNarrationBlockAdditions,
  restoreNarrationBlocksRaw,
} from "./narration-blocks-projection";
import additions from "./narration-profiles-additions.json";
import markerPins from "./narration-profiles-marker-pins.json";
import {
  removeShortPlanningOwnership,
  restoreShortPlanningOwnershipRaw,
} from "./short-candidates-projection";

type RecordValue = Record<string, unknown>;
type Path = (string | number)[];
interface Change {
  after?: unknown;
  before?: unknown;
  path: Path;
}
type Catalog = keyof typeof additions;
const equal = (a: unknown, b: unknown) =>
  JSON.stringify(a) === JSON.stringify(b);
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

// Invert only the independently authored NP1 catalog additions. Never remove
// matching user data elsewhere, change prior pins, or derive catalogs from Zod.
export const removeNarrationProfileAdditions = <T>(
  source: T,
  name: Catalog,
  copy = true
): T => {
  const base = copy ? structuredClone(source) : source;
  const result = removeShortPlanningOwnership(
    removeChannelPackAdditions(
      removeNarrationBlockAdditions(base, name, false),
      name,
      false
    ),
    false
  );
  const changes: Change[] = additions[name];
  let present: boolean;
  if (name === "mcp-surface-v1") {
    present =
      at(result, ["toolDefinitions", "narration_profile_apply"]) !==
        undefined ||
      at(result, [
        "toolDefinitions",
        "editor_get_status",
        "outputSchema",
        "properties",
        "projectSchemaVersion",
        "const",
      ]) === 46;
  } else if (name === "headless-protocol-v1") {
    present =
      (at(result, ["operations"]) as string[] | undefined)?.includes(
        "narration_profile_resolve"
      ) ?? false;
  } else {
    present = at(result, ["categories", "narrationProfiles"]) !== undefined;
  }
  if (!present) {
    return result;
  }
  for (const change of changes) {
    const actual = at(result, change.path);
    if (!equal(actual, change.after)) {
      throw new Error(
        `Incorrect narration profile addition: ${change.path.join(".")}`
      );
    }
    const parent = at(result, change.path.slice(0, -1)) as RecordValue;
    const key = change.path.at(-1);
    if (key === undefined || !parent) {
      throw new Error("Malformed narration profile projection path");
    }
    if (Object.hasOwn(change, "before")) {
      parent[key] = structuredClone(change.before);
    } else {
      Reflect.deleteProperty(parent, key);
    }
  }
  return result;
};

export const restoreNarrationCatalogMarker = <T>(source: T): T => {
  const result = structuredClone(source);
  if (at(result, ["projectSchemaVersion"]) === 47) {
    (result as RecordValue).projectSchemaVersion = 46;
  }
  if (at(result, ["projectSchemaVersion"]) === 46) {
    (result as RecordValue).projectSchemaVersion = 45;
  }
  return result;
};

export const restoreNarrationProfileRaw = (input: string): string => {
  const withoutBlocks = restoreNarrationBlocksRaw(input);
  if (withoutBlocks !== input) {
    return restoreNarrationProfileRaw(withoutBlocks);
  }
  const document: unknown = JSON.parse(input);
  const catalog =
    document && typeof document === "object" && "categories" in document
      ? "contract-ownership-v1"
      : "headless-protocol-v1";
  const withoutChannel = restoreChannelPackRaw(input, catalog);
  if (withoutChannel !== input) {
    return restoreNarrationProfileRaw(withoutChannel);
  }
  const previous = restoreShortPlanningOwnershipRaw(input);
  if (previous !== input) {
    return restoreNarrationProfileRaw(previous);
  }
  const source = JSON.parse(input);
  if (source.projectSchemaVersion === 46) {
    const raw = input.replace(
      '"projectSchemaVersion": 46',
      '"projectSchemaVersion": 45'
    );
    const digest = createHash("sha256").update(raw).digest("hex");
    if (
      !(
        Object.values(markerPins).includes(digest) &&
        equal(JSON.parse(raw), restoreNarrationCatalogMarker(source))
      )
    ) {
      throw new Error(
        "Unrelated narration profile predecessor contract drift (schema marker)"
      );
    }
    return raw;
  }
  let name: Catalog | undefined;
  if (source.toolDefinitions) {
    name = "mcp-surface-v1";
  } else if (source.operations) {
    name = "headless-protocol-v1";
  } else if (source.categories) {
    name = "contract-ownership-v1";
  }
  if (!name) {
    return input;
  }
  const projected = removeNarrationProfileAdditions(source, name);
  if (equal(projected, source)) {
    return input;
  }
  // Preserve unrelated fields and raw headless numeric lexemes so established
  // owner-specific validators still diagnose their own tampering. The NP1
  // tests independently require the exact complete predecessor SHA256.
  let raw = `${JSON.stringify(projected, null, 2)}\n`;
  if (name === "headless-protocol-v1") {
    raw = input;
    const [change] = additions["headless-protocol-v1"];
    if (!change) {
      throw new Error("Missing narration profile operation rollback");
    }
    for (const operation of change.after.filter(
      (value) => !change.before.includes(value)
    )) {
      raw = raw
        .replace(`    ${JSON.stringify(operation)},\r\n`, "")
        .replace(`    ${JSON.stringify(operation)},\n`, "");
    }
  }
  if (!equal(JSON.parse(raw), projected)) {
    throw new Error(`Unrelated ${name} predecessor contract drift`);
  }
  return raw;
};
