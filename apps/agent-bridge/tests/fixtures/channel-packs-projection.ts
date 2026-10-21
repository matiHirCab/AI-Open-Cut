import { deepStrictEqual } from "node:assert/strict";
import contract from "../../../../contracts/channel-style-packs-v1.json";
import { restoreArtworkIntegrationRaw } from "./artwork-integration-projection";
import { restoreNarrationBlocksRaw } from "./narration-blocks-projection";

const equal = (left: unknown, right: unknown): void =>
  deepStrictEqual(left, right);
type Catalog = "contract-ownership-v1" | "headless-protocol-v1";

export const removeChannelPackAdditions = <T>(
  source: T,
  name: string,
  copy = true
): T => {
  if (!source || typeof source !== "object") {
    return source;
  }
  if (name === "contract-ownership-v1") {
    if (
      !("categories" in source && source.categories) ||
      typeof source.categories !== "object" ||
      !Object.hasOwn(source.categories, "channelStylePacks")
    ) {
      return source;
    }
    equal(
      (source.categories as Record<string, unknown>).channelStylePacks,
      contract.ownership
    );
    const result = copy ? structuredClone(source) : source;
    Reflect.deleteProperty(
      (result as { categories: object }).categories,
      "channelStylePacks"
    );
    return result;
  }
  if (
    name !== "headless-protocol-v1" ||
    !("operations" in source) ||
    !Array.isArray(source.operations) ||
    !source.operations.some((operation) =>
      contract.operations.includes(operation)
    )
  ) {
    return source;
  }
  for (const operation of contract.operations) {
    equal(
      source.operations.filter((value) => value === operation),
      [operation]
    );
  }
  const present = source.operations.filter((operation) =>
    contract.operations.includes(operation)
  );
  equal(present, [...contract.operations].sort());
  const result = copy ? structuredClone(source) : source;
  (result as { operations: unknown[] }).operations = source.operations.filter(
    (operation) => !contract.operations.includes(operation)
  );
  return result;
};

const removeOnce = (input: string, fragment: string): string | undefined => {
  const index = input.indexOf(fragment);
  if (index < 0) {
    return undefined;
  }
  if (input.indexOf(fragment, index + fragment.length) >= 0) {
    throw new Error("Repeated channel predecessor fragment");
  }
  return input.slice(0, index) + input.slice(index + fragment.length);
};

/** Remove precisely CP1's reviewed bytes; never serialize unrelated metadata. */
export const restoreChannelPackRaw = (input: string, name: Catalog): string => {
  const withoutArtwork = restoreArtworkIntegrationRaw(input);
  if (withoutArtwork !== input) {
    return restoreChannelPackRaw(withoutArtwork, name);
  }
  const withoutBlocks = restoreNarrationBlocksRaw(input);
  if (withoutBlocks !== input) {
    return restoreChannelPackRaw(withoutBlocks, name);
  }
  const source: unknown = JSON.parse(input);
  const projected = removeChannelPackAdditions(source, name);
  if (projected === source) {
    return input;
  }
  let restored: string | undefined;
  if (name === "contract-ownership-v1") {
    const pretty = JSON.stringify(
      { channelStylePacks: contract.ownership },
      null,
      2
    )
      .split("\n")
      .slice(1, -1)
      .map((line) => `  ${line}`)
      .join("\n");
    const compact = JSON.stringify({
      channelStylePacks: contract.ownership,
    }).slice(1, -1);
    restored =
      removeOnce(input, `${pretty},\n`) ??
      removeOnce(input, `,\n${pretty}`) ??
      removeOnce(input, `${compact},`) ??
      removeOnce(input, `,${compact}`);
  } else {
    const sorted = [...contract.operations].sort();
    const pretty = sorted.map((operation) => `    "${operation}",\n`).join("");
    const compact = `${sorted.map((operation) => JSON.stringify(operation)).join(",")},`;
    restored = removeOnce(input, pretty) ?? removeOnce(input, compact);
  }
  if (restored === undefined) {
    throw new Error("Unrelated channel predecessor contract drift");
  }
  equal(JSON.parse(restored), projected);
  return restored;
};
