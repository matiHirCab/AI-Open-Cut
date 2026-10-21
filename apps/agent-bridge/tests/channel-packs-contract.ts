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
import { z } from "zod/v4";
import contract from "../../../contracts/channel-style-packs-v1.json";
import ownership from "../../../contracts/contract-ownership-v1.json";
import headless from "../../../contracts/headless-protocol-v1.json";
import {
  CHANNEL_CLI_BYTES,
  CHANNEL_PACKAGE_BYTES,
  channelPackageSchema,
  channelPackSchema,
  channelRequestSchema,
} from "../src/channel-packs/contract";
import { restoreChannelPackRaw } from "./fixtures/durable-main47-compat";

export const assertChannelContractConsumer = (source: string): void => {
  const parsed = createSourceFile(
    "contracts.test.ts",
    source,
    ScriptTarget.Latest,
    true
  );
  const aliases = new Set<string>();
  for (const statement of parsed.statements) {
    if (
      !(
        isImportDeclaration(statement) &&
        isStringLiteral(statement.moduleSpecifier)
      ) ||
      statement.moduleSpecifier.text !== "./channel-packs-contract"
    ) {
      continue;
    }
    const bindings = statement.importClause?.namedBindings;
    if (bindings && isNamedImports(bindings)) {
      for (const element of bindings.elements) {
        if (
          (element.propertyName ?? element.name).text ===
          "verifyChannelPackContract"
        ) {
          aliases.add(element.name.text);
        }
      }
    }
  }
  let found = false;
  const visit = (node: Node) => {
    if (
      isCallExpression(node) &&
      isIdentifier(node.expression) &&
      node.expression.text === "it"
    ) {
      const [, callback] = node.arguments;
      if (callback && isIdentifier(callback) && aliases.has(callback.text)) {
        found = true;
      }
    }
    forEachChild(node, visit);
  };
  visit(parsed);
  if (!found) {
    throw new Error(
      "Required channel-pack contract consumer is absent or masked"
    );
  }
};

export const verifyChannelPackContract = () => {
  expect(contract.version).toBe(1);
  expect(contract.limits.packBytes).toBe(CHANNEL_PACKAGE_BYTES);
  expect(contract.limits.cliBytes).toBe(CHANNEL_CLI_BYTES);
  expect(ownership.categories.channelStylePacks).toEqual(contract.ownership);
  expect(channelPackageSchema.parse(contract.package)).toEqual(
    contract.package
  );
  expect(
    channelPackSchema.parse({
      id: "00000000-0000-4000-8000-000000000211",
      package: contract.package,
      version: 1,
    }).package
  ).toEqual(contract.package);
  const operations = channelRequestSchema.options
    .map((schema) => {
      const metadata = z.toJSONSchema(schema);
      const operation = metadata.properties?.operation;
      return operation && typeof operation === "object"
        ? operation.const
        : undefined;
    })
    .sort();
  expect(operations).toEqual([...contract.operations].sort());
  for (const operation of contract.operations) {
    expect(headless.operations.filter((name) => name === operation)).toEqual([
      operation,
    ]);
  }
  for (const request of contract.requests) {
    expect(channelRequestSchema.parse(request)).toEqual(request);
    expect(
      channelRequestSchema.safeParse({ ...request, executable: true }).success
    ).toBe(false);
  }
  assertChannelContractConsumer(
    readFileSync(new URL("./contracts.test.ts", import.meta.url), "utf8")
  );
  for (const name of ["ownership", "headless"] as const) {
    const catalog =
      name === "ownership" ? "contract-ownership-v1" : "headless-protocol-v1";
    const raw = readFileSync(
      new URL(
        `./fixtures/channel-packs-predecessor-${name}.raw`,
        import.meta.url
      ),
      "utf8"
    );
    const current = readFileSync(
      new URL(`../../../contracts/${catalog}.json`, import.meta.url),
      "utf8"
    );
    expect(createHash("sha256").update(raw).digest("hex")).toBe(
      name === "ownership"
        ? contract.predecessorOwnershipSha256
        : contract.predecessorHeadlessSha256
    );
    expect(restoreChannelPackRaw(current, catalog)).toBe(raw);
    expect(restoreChannelPackRaw(raw, catalog)).toBe(raw);
    expect(
      restoreChannelPackRaw(JSON.stringify(JSON.parse(current)), catalog)
    ).toBe(JSON.stringify(JSON.parse(raw)));
  }
};
