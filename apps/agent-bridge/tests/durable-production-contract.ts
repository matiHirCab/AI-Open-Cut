import { deepStrictEqual } from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  createSourceFile,
  forEachChild,
  isCallExpression,
  isIdentifier,
  isImportDeclaration,
  isNamedImports,
  isStringLiteral,
  type Node,
  ScriptTarget,
} from "typescript";
import { expect } from "vitest";
import { z } from "zod/v4";
import liveOwnership from "../../../contracts/contract-ownership-v1.json";
import contract from "../../../contracts/durable-production-requests-v1.json";
import mcp from "../../../contracts/mcp-surface-v1.json";
import {
  DURABLE_CAPABILITY,
  DURABLE_LIMITS,
  parseAdmission,
  publicRecordSchema,
} from "../src/production-requests/contract";
import { registerProductionRequestTools } from "../src/server/production-requests";
import type { Server, ServerDependencies } from "../src/server/shared";
import {
  removeMain47Peers,
  restoreMain47PeersRaw,
} from "./fixtures/durable-main47-peers";
import {
  assertDurableProductionOwnership,
  removeDurableMcpAdditions,
  removeDurableOwnershipAddition,
  restoreDurableRaw,
} from "./fixtures/durable-production-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";
import {
  assertShortPlanningOwnership,
  removeShortPlanningOwnership,
  restoreShortPlanningOwnershipRaw,
} from "./fixtures/short-candidates-projection";
import { assertShortPlanningConsumer } from "./short-candidates-contract";

const digest = (value: string | Buffer) =>
  createHash("sha256").update(value).digest("hex");
const normalize = (value: unknown): unknown => {
  if (Array.isArray(value)) {
    return value.map(normalize);
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .filter(([key]) => key !== "description")
        .sort(([left], [right]) => left.localeCompare(right))
        .map(([key, item]) => [key, normalize(item)])
    );
  }
  return value;
};

/** Require the real direct Vitest consumer, not a comment or a skipped registration. */
export const assertDurableProductionConsumer = (source: string): void => {
  const file = createSourceFile(
    "contracts.test.ts",
    source,
    ScriptTarget.Latest,
    true
  );
  const aliases = new Set<string>();
  for (const statement of file.statements) {
    if (
      !(
        isImportDeclaration(statement) &&
        isStringLiteral(statement.moduleSpecifier)
      ) ||
      statement.moduleSpecifier.text !== "./durable-production-contract"
    ) {
      continue;
    }
    const bindings = statement.importClause?.namedBindings;
    if (bindings && isNamedImports(bindings)) {
      for (const element of bindings.elements) {
        if (
          (element.propertyName ?? element.name).text ===
          "verifyDurableProductionContract"
        ) {
          aliases.add(element.name.text);
        }
      }
    }
  }
  let registered = false;
  const visit = (node: Node) => {
    if (
      isCallExpression(node) &&
      isIdentifier(node.expression) &&
      node.expression.text === "it"
    ) {
      const [, consumer] = node.arguments;
      if (consumer && isIdentifier(consumer) && aliases.has(consumer.text)) {
        registered = true;
      }
    }
    forEachChild(node, visit);
  };
  visit(file);
  if (!registered) {
    throw new Error(
      "Required durable production contract consumer is absent or masked"
    );
  }
};

export const verifyDurableProductionContract = () => {
  const ownership = liveOwnership;
  verifyIndependentGovernanceCoexistence();
  assertDurableProductionOwnership(ownership);
  expect(contract.version).toBe(1);
  expect(contract.capability).toBe(DURABLE_CAPABILITY);
  expect(ownership.categories.durableProductionRequests).toEqual(
    contract.ownership
  );
  expect(DURABLE_LIMITS).toEqual({
    catalogBytes: 1_048_576,
    intentBytes: 4096,
    recordBytes: 8192,
    records: 64,
    retentionMs: 604_800_000,
  });
  for (const example of contract.validIntentNormalization) {
    expect(
      parseAdmission({ ...contract.input, intent: example.input }).intent
    ).toEqual(example.expected);
  }
  for (const snapshot of Object.values(contract.completePublicSnapshots)) {
    expect(publicRecordSchema.parse(snapshot)).toEqual(snapshot);
  }
  const expanded = expandMcpSurfaceCatalog(mcp);
  const registered = new Map<string, unknown>();
  const server = {
    registerTool(
      name: string,
      definition: {
        annotations: unknown;
        inputSchema: z.ZodType;
        outputSchema: z.ZodType;
      }
    ) {
      registered.set(name, {
        annotations: normalize(definition.annotations),
        inputSchema: normalize(
          z.toJSONSchema(definition.inputSchema, {
            io: "input",
            target: "draft-2020-12",
            unrepresentable: "throw",
          })
        ),
        outputSchema: normalize(
          z.toJSONSchema(definition.outputSchema, {
            io: "output",
            target: "draft-2020-12",
            unrepresentable: "throw",
          })
        ),
      });
    },
  } as unknown as Server;
  registerProductionRequestTools(server, {} as ServerDependencies);
  expect([...registered.keys()].sort()).toEqual([...contract.toolNames].sort());
  for (const [name, actual] of registered) {
    deepStrictEqual(actual, normalize(expanded.toolDefinitions[name]));
  }
  expect(() =>
    removeDurableMcpAdditions({
      ...mcp,
      capabilityIdentifiers: mcp.capabilityIdentifiers.filter(
        (value) => value !== contract.capability
      ),
    })
  ).toThrow();
  for (const name of contract.toolNames) {
    const missing = structuredClone(mcp);
    Reflect.deleteProperty(missing.toolDefinitions, name);
    expect(() => removeDurableMcpAdditions(missing)).toThrow();
    const narrowed = structuredClone(mcp);
    narrowed.tools = narrowed.tools.filter((tool) => tool !== name);
    expect(() => removeDurableMcpAdditions(narrowed)).toThrow();
  }
  const ownerDrift = structuredClone(ownership);
  ownerDrift.categories.durableProductionRequests.consumers.pop();
  expect(() => removeDurableOwnershipAddition(ownerDrift)).toThrow();
  const repository = resolve(import.meta.dirname, "../../..");
  for (const name of ["mcp-surface-v1", "contract-ownership-v1"] as const) {
    const raw = readFileSync(
      resolve(repository, `contracts/${name}.json`),
      "utf8"
    );
    const captured = readFileSync(
      resolve(
        import.meta.dirname,
        `fixtures/durable-production-predecessor-${name}.raw`
      )
    );
    expect(captured.length).toBe(contract.predecessors[name].bytes);
    expect(digest(captured)).toBe(contract.predecessors[name].sha256);
    const restore = (input: string) =>
      restoreDurableRaw(
        name === "contract-ownership-v1"
          ? restoreShortPlanningOwnershipRaw(restoreMain47PeersRaw(input))
          : restoreMain47PeersRaw(input)
      );
    expect(restore(raw)).toBe(captured.toString("utf8"));
    const altered = raw.replace('"version": 1', '"version": 2');
    expect(digest(restore(altered))).not.toBe(
      contract.predecessors[name].sha256
    );
  }
};

/** Independent frozen predecessors, checked before this executable integration. */
const ownershipVersionPattern = /"version":\s*1/;
const verifyIndependentGovernanceCoexistence = () => {
  const ownership = removeMain47Peers(liveOwnership, "contract-ownership-v1");
  const raw = readFileSync(
    new URL(
      "./fixtures/short-candidates-predecessor-ownership.raw",
      import.meta.url
    ),
    "utf8"
  );
  expect(digest(raw)).toBe(
    "5febc40dd7b61404226d263058fe628b27dd1e72297f0d3f8aead30c4bb4f4fe"
  );
  const compact = JSON.stringify(JSON.parse(raw));
  expect(Buffer.byteLength(compact)).toBe(72_618);
  expect(digest(compact)).toBe(
    "83bc962dbf3bd509adc10eba9bbb628b36be9a7d652ac582f09a2855e0b8fb5d"
  );
  const current = readFileSync(
    new URL("../../../contracts/contract-ownership-v1.json", import.meta.url),
    "utf8"
  );
  const mcpRaw = readFileSync(
    new URL("../../../contracts/mcp-surface-v1.json", import.meta.url),
    "utf8"
  );
  for (const noOp of [
    mcpRaw,
    " null ",
    " 1.0 ",
    "true",
    ' "unaltered" ',
    "[]",
    '{"version":1.0}',
    '{"categories":null}',
    '{"categories":2}',
    '{"categories":[]}',
  ]) {
    expect(restoreShortPlanningOwnershipRaw(noOp)).toBe(noOp);
  }
  const inverses = [
    (input: string) =>
      restoreDurableRaw(restoreShortPlanningOwnershipRaw(input)),
    (input: string) =>
      restoreShortPlanningOwnershipRaw(restoreDurableRaw(input)),
  ];
  for (const [input, expected] of [
    [restoreMain47PeersRaw(current), raw],
    [JSON.stringify(ownership), compact],
  ]) {
    if (input === undefined || expected === undefined) {
      throw new Error("Missing frozen integration oracle");
    }
    for (const inverse of inverses) {
      expect(inverse(input)).toBe(expected);
      expect(inverse(expected)).toBe(expected);

      expect(
        inverse(input.replace(ownershipVersionPattern, '"version":1.0'))
      ).toBe(expected.replace(ownershipVersionPattern, '"version":1.0'));
      const foreign = '"reviewer":9007199254740993,';
      expect(inverse(input.replace("{", `{${foreign}`))).toBe(
        expected.replace("{", `{${foreign}`)
      );
      const drift = input.replace(
        '"contracts/artifact-delivery-v2.json"',
        '"foreign.json"'
      );
      expect(drift).not.toBe(input);
      const restored = inverse(drift);
      expect(restored).toBe(
        expected.replace(
          '"contracts/artifact-delivery-v2.json"',
          '"foreign.json"'
        )
      );
      expect(digest(restored)).not.toBe(digest(expected));
    }
    expect(JSON.parse(restoreDurableRaw(input))).toEqual(
      removeDurableOwnershipAddition(ownership)
    );
    expect(JSON.parse(restoreShortPlanningOwnershipRaw(input))).toEqual(
      removeShortPlanningOwnership(ownership)
    );
  }
  const reordered = structuredClone(ownership);
  const durable = reordered.categories.durableProductionRequests;
  Reflect.deleteProperty(reordered.categories, "durableProductionRequests");
  const short = reordered.categories.sourceGroundedShortPlanning;
  Reflect.deleteProperty(reordered.categories, "sourceGroundedShortPlanning");
  Object.assign(reordered.categories, {
    durableProductionRequests: durable,
    sourceGroundedShortPlanning: short,
  });
  for (const input of [
    `${JSON.stringify(reordered, null, 2)}\n`,
    JSON.stringify(reordered),
  ]) {
    for (const inverse of inverses) {
      expect(inverse(input)).toBe(input.includes("\n") ? raw : compact);
    }
  }
  for (const [category, assertOwn, removeOwn] of [
    [
      "durableProductionRequests",
      assertDurableProductionOwnership,
      removeDurableOwnershipAddition,
    ],
    [
      "sourceGroundedShortPlanning",
      assertShortPlanningOwnership,
      removeShortPlanningOwnership,
    ],
  ] as const) {
    const missing = structuredClone(ownership);
    Reflect.deleteProperty(missing.categories, category);
    expect(() => assertOwn(missing)).toThrow();
    const malformed = structuredClone(ownership);
    Reflect.set(malformed.categories, category, null);
    expect(() => removeOwn(malformed)).toThrow();
    for (const inverse of inverses) {
      expect(() => inverse(JSON.stringify(malformed))).toThrow();
    }
    const narrowed = structuredClone(ownership);
    narrowed.categories[category].consumers.pop();
    expect(() => removeOwn(narrowed)).toThrow();
    for (const inverse of inverses) {
      expect(() => inverse(JSON.stringify(narrowed))).toThrow();
    }
  }
  const source = readFileSync(
    new URL("./contracts.test.ts", import.meta.url),
    "utf8"
  );
  for (const [callback, assertOwn, assertPeer] of [
    [
      "verifyDurableProductionContract",
      assertDurableProductionConsumer,
      assertShortPlanningConsumer,
    ],
    [
      "verifyShortPlanningContract",
      assertShortPlanningConsumer,
      assertDurableProductionConsumer,
    ],
  ] as const) {
    assertOwn(source);
    const registration = new RegExp(
      `it\\(\\s*"[^"]+",\\s*${callback}\\s*\\)`,
      "g"
    );
    const removed = source.replace(registration, "");
    expect(removed).not.toBe(source);
    expect(() => assertOwn(removed)).toThrow();
    assertPeer(removed);
    const skipped = source.replace(registration, (match) =>
      match.replace("it(", "it.skip(")
    );
    expect(() => assertOwn(skipped)).toThrow();
    assertPeer(skipped);
    const commented = source.replace(registration, (match) => `/* ${match} */`);
    expect(() => assertOwn(commented)).toThrow();
    assertPeer(commented);
  }
};
