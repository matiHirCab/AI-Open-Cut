import { deepStrictEqual } from "node:assert/strict";
import contract from "../../../../contracts/short-candidate-planning-cli-v1.json";
import { restoreNarrationBlocksRaw } from "./narration-blocks-projection";

export const assertShortPlanningOwnership = (source: unknown): void => {
  if (
    !source ||
    typeof source !== "object" ||
    !("categories" in source) ||
    !source.categories ||
    typeof source.categories !== "object" ||
    !("sourceGroundedShortPlanning" in source.categories)
  ) {
    throw new Error("Missing short planning ownership addition");
  }
  try {
    deepStrictEqual(
      source.categories.sourceGroundedShortPlanning,
      contract.ownership
    );
  } catch (cause) {
    throw new Error("Incorrect short planning ownership addition", { cause });
  }
};

/** Invert only the independently authored new category, preserving every other byte/value. */
export const removeShortPlanningOwnership = <T>(source: T, copy = true): T => {
  const result = copy ? structuredClone(source) : source;
  if (!result || typeof result !== "object" || !("categories" in result)) {
    return result;
  }
  const { categories } = result;
  if (
    !categories ||
    typeof categories !== "object" ||
    !("sourceGroundedShortPlanning" in categories)
  ) {
    return result;
  }
  assertShortPlanningOwnership(result);
  Reflect.deleteProperty(categories, "sourceGroundedShortPlanning");
  return result;
};

export const restoreShortPlanningOwnershipRaw = (input: string): string => {
  const withoutBlocks = restoreNarrationBlocksRaw(input);
  if (withoutBlocks !== input) {
    return restoreShortPlanningOwnershipRaw(withoutBlocks);
  }
  const value: unknown = JSON.parse(input);
  const projected = removeShortPlanningOwnership(value);
  if (JSON.stringify(value) === JSON.stringify(projected)) {
    return input;
  }
  const entry = JSON.stringify(
    { sourceGroundedShortPlanning: contract.ownership },
    null,
    2
  )
    .split("\n")
    .slice(1, -1)
    .map((line) => `  ${line}`)
    .join("\n");
  const ownedSuffix = `,\n${entry}\n  }\n}\n`;
  const compactEntry = JSON.stringify({
    sourceGroundedShortPlanning: contract.ownership,
  }).slice(1, -1);
  const compactSuffix = `,${compactEntry}}}`;
  let restored: string;
  if (input.endsWith(ownedSuffix)) {
    restored = `${input.slice(0, -ownedSuffix.length)}\n  }\n}\n`;
  } else if (input.endsWith(compactSuffix)) {
    restored = `${input.slice(0, -compactSuffix.length)}}}`;
  } else {
    throw new Error("Unrelated short planning predecessor contract drift");
  }
  deepStrictEqual(JSON.parse(restored), projected);
  return restored;
};
