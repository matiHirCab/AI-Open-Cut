import { deepStrictEqual } from "node:assert/strict";
import shortContract from "../../../../contracts/short-candidate-planning-cli-v1.json";
import {
  removeChannelPackAdditions as removeChannel,
  restoreChannelPackRaw as restoreChannel,
} from "./channel-packs-projection";
import pins from "./durable-main47-pins.json";
import {
  normalizeToMain47,
  restoreMain47Raw,
} from "./durable-main47-projection";
import {
  removeNarrationBlockAdditions as removeNarration,
  restoreNarrationBlocksRaw as restoreNarration,
} from "./narration-blocks-projection";
import { restoreShortPlanningOwnershipRaw as restoreCurrentShort } from "./short-candidates-projection";
export const removeNarrationBlockAdditions = <T>(
  source: T,
  name: string,
  copy = true
): T => removeNarration(normalizeToMain47(source, copy), name, false);
export const restoreNarrationBlocksRaw = (input: string): string =>
  restoreNarration(restoreMain47Raw(input));
export const removeChannelPackAdditions = <T>(
  source: T,
  name: string,
  copy = true
): T => removeChannel(normalizeToMain47(source, copy), name, false);
export const restoreChannelPackRaw = (
  input: string,
  name: "contract-ownership-v1" | "headless-protocol-v1"
): string => restoreChannel(restoreMain47Raw(input), name);

// Main's frozen source-planning category predates exactly two reviewed durable consumers.
export const restoreShortPlanningOwnershipRaw = (input: string): string => {
  const source = JSON.parse(input);
  const actual = source?.categories?.sourceGroundedShortPlanning;
  if (
    actual === undefined ||
    JSON.stringify(actual) === JSON.stringify(shortContract.ownership)
  ) {
    return restoreCurrentShort(input);
  }
  deepStrictEqual(actual, pins.mainShortOwnership);
  for (const space of [2, undefined]) {
    const fragment = (value: unknown) =>
      space === 2
        ? JSON.stringify({ sourceGroundedShortPlanning: value }, null, 2)
            .split("\n")
            .slice(1, -1)
            .map((line) => `  ${line}`)
            .join("\n")
        : JSON.stringify({ sourceGroundedShortPlanning: value }).slice(1, -1);
    const old = fragment(pins.mainShortOwnership);
    if (input.split(old).length === 2) {
      return restoreCurrentShort(
        input.replace(old, fragment(shortContract.ownership))
      );
    }
  }
  throw new Error("Missing exact main source planning ownership fragment");
};
