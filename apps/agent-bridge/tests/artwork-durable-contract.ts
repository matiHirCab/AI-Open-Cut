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
import pins from "./fixtures/artwork-durable-input-pins.json";
import { restoreDurableIntegrationRaw } from "./fixtures/artwork-durable-projection";
import { restoreArtworkIntegrationRaw } from "./fixtures/artwork-integration-projection";

const digest = (raw: string) => createHash("sha256").update(raw).digest("hex");
export const assertArtworkDurableConsumer = (source: string): void => {
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
      statement.moduleSpecifier.text !== "./artwork-durable-contract"
    ) {
      continue;
    }
    const bindings = statement.importClause?.namedBindings;
    if (bindings && isNamedImports(bindings)) {
      for (const element of bindings.elements) {
        if (
          (element.propertyName ?? element.name).text ===
          "verifyArtworkDurableContract"
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
      "Required artwork/durable integration consumer is absent or masked"
    );
  }
};

/** Freeze both independently fetched heads; retain every existing predecessor oracle. */
export const verifyArtworkDurableContract = () => {
  assertArtworkDurableConsumer(
    readFileSync(new URL("./contracts.test.ts", import.meta.url), "utf8")
  );
  for (const [label, restore] of [
    ["main", restoreArtworkIntegrationRaw],
    ["artwork", restoreDurableIntegrationRaw],
  ] as const) {
    for (const [path, expected] of Object.entries(pins.catalogs[label])) {
      const raw = readFileSync(
        new URL(`../../../${path}`, import.meta.url),
        "utf8"
      );
      for (const [input, key] of [
        [raw, "rawSha256"],
        [JSON.stringify(JSON.parse(raw)), "compactSha256"],
      ] as const) {
        const restored = restore(input);
        expect(digest(restored), `${label} ${path} ${key}`).toBe(expected[key]);
        expect(restore(restored)).toBe(restored);
      }
      if (
        path === "contracts/mcp-surface-v1.json" ||
        path === "contracts/contract-ownership-v1.json"
      ) {
        const captured = readFileSync(
          new URL(
            `./fixtures/artwork-durable-${label}-${path.slice(10, -5)}.raw`,
            import.meta.url
          ),
          "utf8"
        );
        expect(digest(captured)).toBe(expected.rawSha256);
        expect(restore(raw)).toBe(captured);
      }
    }
  }
  for (const name of [
    "mcp-surface-v1",
    "contract-ownership-v1",
    "headless-protocol-v1",
  ] as const) {
    const raw = readFileSync(
      new URL(`../../../contracts/${name}.json`, import.meta.url),
      "utf8"
    );
    for (const input of [raw, JSON.stringify(JSON.parse(raw))]) {
      expect(
        restoreArtworkIntegrationRaw(restoreDurableIntegrationRaw(input))
      ).toBe(restoreDurableIntegrationRaw(restoreArtworkIntegrationRaw(input)));
    }
  }
};
