import { deepStrictEqual } from "node:assert/strict";
import contract from "../../../../contracts/artwork-preparation-v1.json";
import historical from "./artwork-additions.json";
import { removeArtworkAdditions } from "./artwork-projection";

type Document = Record<string, unknown>;
const versionPaths = [
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
] as const;
const parentAt = (value: Document, path: readonly string[]): Document => {
  let parent = value;
  for (const key of path.slice(0, -1)) {
    const candidate: unknown = parent[key];
    if (
      !candidate ||
      typeof candidate !== "object" ||
      Array.isArray(candidate)
    ) {
      throw new Error("Missing integration schema marker parent");
    }
    parent = candidate as Document;
  }
  return parent;
};

/** Only AF1's integrated schema48 additions; retain main's exact schema47. */
export const removeArtworkIntegration = <T>(source: T, copy = true): T => {
  if (!source || typeof source !== "object") {
    return source;
  }
  let result = (copy ? structuredClone(source) : source) as Document;
  if (result.projectSchemaVersion === 48) {
    result.projectSchemaVersion = 47;
  } else if (result.toolDefinitions) {
    const properties = (result.$defs as Document | undefined)
      ?.ProjectPropertiesAssetsItemsProperties as Document | undefined;
    if (!(properties && Object.hasOwn(properties, "artworkDerivation"))) {
      if (versionPaths.some((path) => parentAt(result, path).const === 48)) {
        throw new Error(
          "Incorrect artwork addition: missing artworkDerivation"
        );
      }
      return result as T;
    }
    // The original AF1 inverse stays unchanged at its historical47 ->46 boundary.
    for (const path of versionPaths) {
      const parent = parentAt(result, path);
      deepStrictEqual(parent.const, 48);
      parent.const = 47;
    }
    result = removeArtworkAdditions(result, "mcp-surface-v1", false);
    for (const path of versionPaths) {
      const parent = parentAt(result, path);
      deepStrictEqual(parent.const, 46);
      parent.const = 47;
    }
  } else if (result.operations) {
    result = removeArtworkAdditions(result, "headless-protocol-v1", false);
  } else if (result.categories) {
    const categories = result.categories as Document;
    if (Object.hasOwn(categories, "artworkPreparation")) {
      deepStrictEqual(categories.artworkPreparation, contract.ownership);
      categories.artworkPreparation = structuredClone(
        historical["contract-ownership-v1"][0]?.after
      );
    }
    result = removeArtworkAdditions(result, "contract-ownership-v1", false);
  }
  return result as T;
};

export const restoreArtworkIntegrationRaw = (input: string): string => {
  const source = JSON.parse(input) as Document;
  const result = removeArtworkIntegration(source);
  if (JSON.stringify(source) === JSON.stringify(result)) {
    return input;
  }
  let raw = input;
  const replaceOnce = (fragment: string, replacement = "") => {
    if (raw.split(fragment).length !== 2) {
      throw new Error("Missing or repeated integration fragment");
    }
    raw = raw.replace(fragment, replacement);
  };
  const removeProperty = (key: string, value: unknown, indent: number) => {
    const pretty = JSON.stringify({ [key]: value }, null, 2)
      .split("\n")
      .slice(1, -1)
      .map((line) => `${" ".repeat(indent - 2)}${line}`)
      .join("\n");
    const compact = JSON.stringify({ [key]: value }).slice(1, -1);
    for (const fragment of [
      `${pretty},\n`,
      `,\n${pretty}`,
      `${compact},`,
      `,${compact}`,
    ]) {
      if (raw.includes(fragment)) {
        replaceOnce(fragment);
        return;
      }
    }
    throw new Error(
      "Incorrect artwork addition: missing exact property fragment"
    );
  };
  if (source.projectSchemaVersion === 48) {
    const marker = input.includes("\n")
      ? '"projectSchemaVersion": 48'
      : '"projectSchemaVersion":48';
    replaceOnce(marker, marker.replace("48", "47"));
  } else if (source.operations) {
    for (const operation of ["commit_artwork", "prepare_artwork"]) {
      const quoted = `"${operation}",`;
      replaceOnce(raw.includes("\n") ? `    ${quoted}\n` : quoted);
    }
  } else if (source.categories) {
    removeProperty(
      "artworkPreparation",
      (source.categories as Document).artworkPreparation,
      4
    );
  } else if (source.toolDefinitions) {
    removeProperty(
      "artworkDerivation",
      (
        (source.$defs as Document)
          .ProjectPropertiesAssetsItemsProperties as Document
      ).artworkDerivation,
      6
    );
    const markers = raw.match(/"const": ?48(?=[,}\s])/g) ?? [];
    if (markers.length !== 2) {
      throw new Error("Missing or repeated integration MCP schema marker");
    }
    raw = raw.replace(/"const": ?48(?=[,}\s])/g, (marker) =>
      marker.replace("48", "47")
    );
  }
  deepStrictEqual(JSON.parse(raw), result);
  return raw;
};
