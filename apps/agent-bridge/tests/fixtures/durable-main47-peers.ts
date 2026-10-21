import {
  removeChannelPackAdditions,
  restoreChannelPackRaw,
} from "./channel-packs-projection";
import {
  removeNarrationBlockAdditions,
  restoreNarrationBlocksRaw,
} from "./narration-blocks-projection";

// Remove only main's independently approved additions, preserving durable peers.
export const removeMain47Peers = <T>(source: T, name: string, copy = true): T =>
  removeChannelPackAdditions(
    removeNarrationBlockAdditions(source, name, copy),
    name,
    false
  );
export const restoreMain47PeersRaw = (input: string): string => {
  const raw = restoreNarrationBlocksRaw(input);
  const value = JSON.parse(raw);
  if (value?.categories) {
    return restoreChannelPackRaw(raw, "contract-ownership-v1");
  }
  if (value?.operations) {
    return restoreChannelPackRaw(raw, "headless-protocol-v1");
  }
  return raw;
};

import { deepStrictEqual } from "node:assert/strict";
import durableContract from "../../../../contracts/durable-production-requests-v1.json";
import pins from "./durable-main47-pins.json";
export const restoreDurable228Raw = (input: string): string => {
  const raw = restoreMain47PeersRaw(input);
  const source = JSON.parse(raw);
  const actual = source?.categories?.durableProductionRequests;
  if (
    actual === undefined ||
    JSON.stringify(actual) === JSON.stringify(pins.originalDurableOwnership)
  ) {
    return raw;
  }
  deepStrictEqual(actual, durableContract.ownership);
  for (const space of [2, undefined]) {
    const fragment = (value: unknown) =>
      space === 2
        ? JSON.stringify({ durableProductionRequests: value }, null, 2)
            .split("\n")
            .slice(1, -1)
            .map((line) => `  ${line}`)
            .join("\n")
        : JSON.stringify({ durableProductionRequests: value }).slice(1, -1);
    const old = fragment(durableContract.ownership);
    if (raw.split(old).length === 2) {
      const restored = raw.replace(
        old,
        fragment(pins.originalDurableOwnership)
      );
      const expected = structuredClone(source);
      expected.categories.durableProductionRequests =
        pins.originalDurableOwnership;
      deepStrictEqual(JSON.parse(restored), expected);
      return restored;
    }
  }
  throw new Error("Missing exact durable integration ownership fragment");
};
