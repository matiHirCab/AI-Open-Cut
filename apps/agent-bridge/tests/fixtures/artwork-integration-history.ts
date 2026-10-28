import { deepStrictEqual } from "node:assert/strict";
import artwork from "../../../../contracts/artwork-preparation-v1.json";
import channel from "../../../../contracts/channel-style-packs-v1.json";
import narration from "../../../../contracts/narration-blocks-v1.json";
import historical from "./artwork-additions.json";
import { restoreDurableIntegrationRaw } from "./artwork-durable-projection";
import additions from "./narration-blocks-mcp-additions.json";

type Document = Record<string, unknown>;
const removeNarrationMcp = (result: Document): void => {
  const defs = result.$defs as Record<string, Document | undefined>;
  const properties = defs.ProjectGetStateOutputPropertiesProjectProperties;
  if (!properties) {
    throw new Error("Missing integrated project schema");
  }
  deepStrictEqual(properties.narrationManifests, additions.manifestArray);
  Reflect.deleteProperty(properties, "narrationManifests");
  const container = defs.ProjectGetStateOutputPropertiesProject;
  if (!(container && Array.isArray(container.required))) {
    throw new Error("Missing narration required fields");
  }
  deepStrictEqual(
    container.required.filter((key) => key === "narrationManifests"),
    ["narrationManifests"]
  );
  container.required = container.required.filter(
    (key) => key !== "narrationManifests"
  );
  for (const path of additions.versionPaths) {
    let parent = result;
    for (const key of path.slice(0, -1)) {
      parent = parent[key] as Document;
    }
    deepStrictEqual(parent.const, 48);
    parent.const = 47;
  }
};

/** Reconstruct the independently pinned AF1-only branch, retaining its artwork. */
export const restoreArtworkBranchRaw = (current: string): string => {
  const input = restoreDurableIntegrationRaw(current);
  const result = JSON.parse(input) as Document;
  let raw = input;
  if (result.projectSchemaVersion === 48) {
    result.projectSchemaVersion = 47;
    const marker = input.includes("\n")
      ? '"projectSchemaVersion": 48'
      : '"projectSchemaVersion":48';
    if (input.split(marker).length !== 2) {
      throw new Error("Missing or repeated integration marker");
    }
    raw = input.replace(marker, marker.replace("48", "47"));
    deepStrictEqual(JSON.parse(raw), result);
    return raw;
  }
  if (result.toolDefinitions) {
    removeNarrationMcp(result);
  } else if (Array.isArray(result.operations)) {
    for (const operation of [...narration.operations, ...channel.operations]) {
      deepStrictEqual(
        result.operations.filter((value) => value === operation),
        [operation]
      );
      result.operations.splice(result.operations.indexOf(operation), 1);
      const fragment = input.includes("\n")
        ? `    "${operation}",\n`
        : `"${operation}",`;
      if (raw.split(fragment).length !== 2) {
        throw new Error("Missing or repeated peer operation fragment");
      }
      raw = raw.replace(fragment, "");
    }
  } else if (result.categories) {
    const categories = result.categories as Document;
    deepStrictEqual(categories.narrationBlocks, narration.ownership);
    deepStrictEqual(categories.channelStylePacks, channel.ownership);
    Reflect.deleteProperty(categories, "narrationBlocks");
    Reflect.deleteProperty(categories, "channelStylePacks");
    deepStrictEqual(categories.artworkPreparation, artwork.ownership);
    categories.artworkPreparation = structuredClone(
      historical["contract-ownership-v1"][0]?.after
    );
  }
  if (result.operations && !result.projectSchemaVersion) {
    deepStrictEqual(JSON.parse(raw), result);
    return raw;
  }
  return input.includes("\n")
    ? `${JSON.stringify(result, null, 2)}\n`
    : JSON.stringify(result);
};
