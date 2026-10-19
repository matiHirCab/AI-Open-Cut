import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
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
import ownership from "../../../contracts/contract-ownership-v1.json";
import contract from "../../../contracts/short-candidate-planning-cli-v1.json";
import {
  PLANNING_LIMITS,
  PLANNING_WARNINGS,
  PlanningError,
  shortPlanningFailureSchema,
  shortPlanningSourceSchema,
  shortPlanningSuccessSchema,
} from "../src/short-candidates/contract";
import { planShortCandidates } from "../src/short-candidates/planner";
import {
  assertShortPlanningOwnership,
  removeShortPlanningOwnership,
  restoreShortPlanningOwnershipRaw,
} from "./fixtures/short-candidates-projection";

/** Require the real direct Vitest consumer, not a comment or a skipped registration. */
export const assertShortPlanningConsumer = (source: string): void => {
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
      statement.moduleSpecifier.text !== "./short-candidates-contract"
    ) {
      continue;
    }
    const bindings = statement.importClause?.namedBindings;
    if (bindings && isNamedImports(bindings)) {
      for (const element of bindings.elements) {
        if (
          (element.propertyName ?? element.name).text ===
          "verifyShortPlanningContract"
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
      "Required short planning contract consumer is absent or masked"
    );
  }
};

export const verifyShortPlanningContract = () => {
  assertShortPlanningOwnership(ownership);
  assertShortPlanningConsumer(
    readFileSync(new URL("./contracts.test.ts", import.meta.url), "utf8")
  );
  const source = shortPlanningSourceSchema.parse(contract.source);
  expect(contract.version).toBe(1);
  expect(contract.limits).toEqual(PLANNING_LIMITS);
  expect(contract.warnings).toEqual(PLANNING_WARNINGS);
  expect(ownership.categories.sourceGroundedShortPlanning).toEqual(
    contract.ownership
  );
  for (const example of contract.valid) {
    expect(planShortCandidates(example.input, source), example.name).toEqual(
      example.expected
    );
    expect(shortPlanningSuccessSchema.parse(example.expected)).toEqual(
      example.expected
    );
    expect(
      shortPlanningSuccessSchema.safeParse({
        ...example.expected,
        invented: true,
      }).success
    ).toBe(false);
  }
  const [original] = contract.valid;
  if (!original) {
    throw new Error("Missing independent planning fixture");
  }
  for (const example of contract.invalid) {
    const input: unknown = structuredClone(original.input);
    let parent = input as Record<string | number, unknown>;
    for (const part of example.path.slice(0, -1)) {
      parent = parent[part] as Record<string | number, unknown>;
    }
    const leaf = example.path.at(-1);
    if (leaf === undefined) {
      throw new Error("Missing invalid fixture path");
    }
    Object.defineProperty(parent, leaf, {
      configurable: true,
      enumerable: true,
      value: structuredClone(example.value),
      writable: true,
    });
    try {
      planShortCandidates(input, source);
      throw new Error(`Accepted invalid canonical fixture: ${example.name}`);
    } catch (error) {
      expect(error, example.name).toBeInstanceOf(PlanningError);
      expect((error as PlanningError).kind, example.name).toBe(example.kind);
    }
  }
  for (const error of contract.errors) {
    expect(shortPlanningFailureSchema.parse(error)).toEqual(error);
    expect(
      shortPlanningFailureSchema.safeParse({ ...error, secretPath: "/private" })
        .success
    ).toBe(false);
  }
  const current = readFileSync(
    new URL("../../../contracts/contract-ownership-v1.json", import.meta.url),
    "utf8"
  );
  const raw = readFileSync(
    new URL(
      "./fixtures/short-candidates-predecessor-ownership.raw",
      import.meta.url
    ),
    "utf8"
  );
  expect(createHash("sha256").update(raw).digest("hex")).toBe(
    contract.predecessorOwnershipSha256
  );
  expect(restoreShortPlanningOwnershipRaw(current)).toBe(raw);
  expect(restoreShortPlanningOwnershipRaw(raw)).toBe(raw);
  expect(restoreShortPlanningOwnershipRaw(JSON.stringify(ownership))).toBe(
    JSON.stringify(JSON.parse(raw))
  );
  const compactDrift = structuredClone(ownership);
  compactDrift.categories.artifactDelivery.canonical = "foreign.json";
  const restoredDrift = restoreShortPlanningOwnershipRaw(
    JSON.stringify(compactDrift)
  );
  expect(JSON.parse(restoredDrift).categories.artifactDelivery.canonical).toBe(
    "foreign.json"
  );
  expect(restoredDrift).not.toBe(JSON.stringify(JSON.parse(raw)));
  const changed = structuredClone(ownership);
  changed.categories.sourceGroundedShortPlanning.consumers.pop();
  expect(() => removeShortPlanningOwnership(changed)).toThrow(
    "Incorrect short planning ownership addition"
  );
  const missing = structuredClone(ownership);
  Reflect.deleteProperty(missing.categories, "sourceGroundedShortPlanning");
  expect(() => assertShortPlanningOwnership(missing)).toThrow(
    "Missing short planning ownership addition"
  );
  const unrelated = structuredClone(ownership);
  unrelated.reviewer = "unrelated drift";
  expect(removeShortPlanningOwnership(unrelated).reviewer).toBe(
    "unrelated drift"
  );
  expect(
    restoreShortPlanningOwnershipRaw(`${JSON.stringify(unrelated, null, 2)}\n`)
  ).not.toBe(raw);
  expect(ownership.categories.sourceGroundedShortPlanning).toEqual(
    contract.ownership
  );
};
