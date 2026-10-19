import { deepStrictEqual } from "node:assert/strict";
import contract from "../../../../contracts/durable-production-requests-v1.json";

const record = (value: unknown): Record<string, unknown> => {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Malformed durable contract projection");
  }
  return value as Record<string, unknown>;
};
const equal = (actual: unknown, expected: unknown) => {
  try {
    deepStrictEqual(actual, expected);
  } catch (cause) {
    throw new Error("Incorrect durable contract addition", { cause });
  }
};
const removeValue = (value: unknown, expected: string) => {
  if (
    !Array.isArray(value) ||
    value.filter((item) => item === expected).length !== 1
  ) {
    throw new Error("Incorrect durable contract array addition");
  }
  value.splice(value.indexOf(expected), 1);
};

/** Invert only the exact new surface, preserving all historical drift for its owners. */
export const removeDurableMcpAdditions = <T>(input: T, copy = true): T => {
  const result = copy ? structuredClone(input) : input;
  const source = record(result);
  const tools = record(source.toolDefinitions ?? {});
  const definitions = record(source.$defs ?? {});
  const addition = contract.mcpAdditions;
  const present =
    Object.keys(addition.tools).some((name) => Object.hasOwn(tools, name)) ||
    Object.keys(addition.definitions).some((name) =>
      Object.hasOwn(definitions, name)
    ) ||
    (Array.isArray(source.capabilityIdentifiers) &&
      source.capabilityIdentifiers.includes(addition.capability));
  if (!present) {
    return result;
  }
  removeValue(source.capabilityIdentifiers, addition.capability);
  for (const [name, expected] of Object.entries(addition.tools)) {
    equal(tools[name], expected);
    removeValue(source.tools, name);
    Reflect.deleteProperty(tools, name);
  }
  for (const [name, expected] of Object.entries(addition.definitions)) {
    equal(definitions[name], expected);
    Reflect.deleteProperty(definitions, name);
  }
  return result;
};

export const assertDurableProductionOwnership = (input: unknown): void => {
  const categories = record(record(input).categories);
  if (!Object.hasOwn(categories, "durableProductionRequests")) {
    throw new Error("Missing durable ownership addition");
  }
  equal(categories.durableProductionRequests, contract.ownership);
};

export const removeDurableOwnershipAddition = <T>(input: T, copy = true): T => {
  const result = copy ? structuredClone(input) : input;
  const source = record(result);
  const categories = record(source.categories ?? {});
  if (!Object.hasOwn(categories, "durableProductionRequests")) {
    return result;
  }
  assertDurableProductionOwnership(source);
  Reflect.deleteProperty(categories, "durableProductionRequests");
  return result;
};

const once = (input: string, fragment: string, replacement: string) => {
  if (input.split(fragment).length !== 2) {
    throw new Error("Incorrect durable raw addition representation");
  }
  return input.replace(fragment, replacement);
};
const entries = (value: unknown) =>
  JSON.stringify(value, null, 2)
    .split("\n")
    .slice(1, -1)
    .map((line) => `  ${line}`)
    .join("\n");

/** Literal inverse: retain unrelated bytes/numeric lexemes and historical diagnostics. */
export const restoreDurableRaw = (input: string): string => {
  const source = record(JSON.parse(input));
  let restored = input;
  let projected: unknown;
  const pretty = input.includes("\n");
  if (
    Object.hasOwn(record(source.categories ?? {}), "durableProductionRequests")
  ) {
    projected = removeDurableOwnershipAddition(source);
    const addition = { durableProductionRequests: contract.ownership };
    restored = pretty
      ? once(input, `,\n${entries(addition)}`, "")
      : once(input, `,${JSON.stringify(addition).slice(1, -1)}`, "");
  } else if (
    Array.isArray(source.capabilityIdentifiers) &&
    source.capabilityIdentifiers.includes(contract.capability)
  ) {
    projected = removeDurableMcpAdditions(source);
    restored = pretty
      ? once(restored, `,\n    "${contract.capability}"`, "")
      : once(restored, `,"${contract.capability}"`, "");
    for (const name of Object.keys(contract.mcpAdditions.tools)) {
      restored = pretty
        ? once(restored, `    "${name}",\n`, "")
        : once(restored, `"${name}",`, "");
    }
    restored = pretty
      ? once(
          restored,
          `,\n${entries(contract.mcpAdditions.tools)}\n  },\n  "tools"`,
          '\n  },\n  "tools"'
        )
      : once(
          restored,
          `,${JSON.stringify(contract.mcpAdditions.tools).slice(1, -1)}},"tools"`,
          '},"tools"'
        );
    restored = pretty
      ? once(
          restored,
          `,\n${entries(contract.mcpAdditions.definitions)}\n  }`,
          "\n  }"
        )
      : once(
          restored,
          `,${JSON.stringify(contract.mcpAdditions.definitions).slice(1, -1)}}`,
          "}"
        );
  } else {
    return input;
  }
  equal(JSON.parse(restored), projected);
  return restored;
};
