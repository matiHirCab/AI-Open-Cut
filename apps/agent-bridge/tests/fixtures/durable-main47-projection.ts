import {
  removeDurableIntegration,
  restoreDurableIntegrationRaw,
} from "./artwork-durable-projection";
import {
  removeArtworkIntegration,
  restoreArtworkIntegrationRaw,
} from "./artwork-integration-projection";

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

export const normalizeToMain47 = <T>(source: T, copy = true): T => {
  const operations = at(source, ["operations"]);
  const hasArtwork =
    at(source, ["projectSchemaVersion"]) === 48 ||
    at(source, ["categories", "artworkPreparation"]) !== undefined ||
    at(source, [
      "$defs",
      "ProjectPropertiesAssetsItemsProperties",
      "artworkDerivation",
    ]) !== undefined ||
    at(source, [
      "toolDefinitions",
      "editor_get_status",
      "outputSchema",
      "properties",
      "projectSchemaVersion",
      "const",
    ]) === 48 ||
    (Array.isArray(operations) &&
      operations.some(
        (value) => value === "prepare_artwork" || value === "commit_artwork"
      ));
  return hasArtwork
    ? removeDurableIntegration(removeArtworkIntegration(source, copy), false)
    : removeDurableIntegration(source, copy);
};

export const restoreMain47Raw = (input: string): string =>
  restoreDurableIntegrationRaw(restoreArtworkIntegrationRaw(input));
