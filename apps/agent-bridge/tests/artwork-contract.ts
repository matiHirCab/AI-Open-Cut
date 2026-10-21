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
import contract from "../../../contracts/artwork-preparation-v1.json";
import ownership from "../../../contracts/contract-ownership-v1.json";
import HEADLESS from "../../../contracts/headless-protocol-v1.json";
import MCP from "../../../contracts/mcp-surface-v1.json";
import {
  artworkCliInputSchema,
  artworkCliOutputSchema,
} from "../src/artwork/cli";
import {
  ARTWORK_LIMITS,
  artworkDerivationSchema,
  artworkManifestSchema,
  artworkParametersSchema,
} from "../src/artwork/contract";
import type { HeadlessRequest } from "../src/headless-contract";
import additions from "./fixtures/artwork-additions.json";
import { restoreArtworkBranchRaw } from "./fixtures/artwork-integration-history";
import pins from "./fixtures/artwork-predecessor-pins.json";
import {
  removeArtworkAdditions,
  restoreArtworkRaw,
} from "./fixtures/artwork-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";

const digest = (value: string) =>
  createHash("sha256").update(value).digest("hex");

/** Require a live direct registration in the unchanged standalone contract gate. */
export const assertArtworkConsumer = (source: string): void => {
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
      statement.moduleSpecifier.text !== "./artwork-contract"
    ) {
      continue;
    }
    const bindings = statement.importClause?.namedBindings;
    if (bindings && isNamedImports(bindings)) {
      for (const element of bindings.elements) {
        if (
          (element.propertyName ?? element.name).text ===
          "verifyArtworkContract"
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
    throw new Error("Required artwork contract consumer is absent or masked");
  }
};

/** Required from the unchanged standalone contracts.test.ts gate. */
export const verifyArtworkContract = () => {
  assertArtworkConsumer(
    readFileSync(new URL("./contracts.test.ts", import.meta.url), "utf8")
  );
  expect(contract.cli.version).toBe(1);
  expect(
    artworkCliInputSchema.options.map((option) => option.shape.operation.value)
  ).toEqual(contract.cli.operations);
  expect(contract.cli.validRequests.map(({ operation }) => operation)).toEqual(
    contract.cli.operations
  );
  expect(contract.cli.validRequests[0]).toEqual({
    expectedRevision: contract.manifest.expectedRevision,
    operation: "preview",
    outputDirectory: contract.cli.responses.preview.directory,
    parameters: contract.parameters,
    projectId: contract.manifest.projectId,
    sourceAssetId: contract.manifest.sourceAssetId,
  });
  expect(contract.cli.responses.preview.manifest).toEqual(contract.manifest);
  for (const request of contract.cli.validRequests) {
    expect(artworkCliInputSchema.parse(request)).toEqual(request);
  }
  for (const { name, request } of contract.cli.invalidRequests) {
    expect(artworkCliInputSchema.safeParse(request).success, name).toBe(false);
  }
  for (const response of Object.values(contract.cli.responses)) {
    expect(artworkCliOutputSchema.parse(response)).toEqual(response);
    expect(
      artworkCliOutputSchema.safeParse({ ...response, unapproved: true })
        .success
    ).toBe(false);
    expect(
      artworkCliOutputSchema.safeParse({ ...response, version: 2 }).success
    ).toBe(false);
  }
  expect(contract.version).toBe(1);
  expect(contract.projectSchemaVersion).toBe(48);
  expect(contract.limits).toEqual(ARTWORK_LIMITS);
  const parameters = artworkParametersSchema.parse(contract.parameters);
  const manifest = artworkManifestSchema.parse(contract.manifest);
  expect(artworkDerivationSchema.parse(contract.derivation)).toEqual(
    contract.derivation
  );
  const requests = [
    {
      expectedRevision: manifest.expectedRevision,
      operation: "prepare_artwork",
      parameters,
      projectId: manifest.projectId,
      sourceAssetId: manifest.sourceAssetId,
    },
    { manifest, operation: "commit_artwork" },
  ] satisfies HeadlessRequest[];
  expect(requests.map(({ operation }) => operation).sort()).toEqual(
    contract.operations
  );
  for (const operation of contract.operations) {
    expect(
      HEADLESS.operations.filter((value) => value === operation)
    ).toHaveLength(1);
  }
  expect(ownership.categories.artworkPreparation).toEqual(contract.ownership);
  const schema = z.toJSONSchema(artworkDerivationSchema, { io: "output" });
  const { $schema: _dialect, ...structural } = schema;
  expect(structural).toEqual(additions["mcp-surface-v1"][2]?.after);
  expect(
    MCP.$defs.ProjectPropertiesAssetsItemsProperties.artworkDerivation
  ).toEqual(structural);
  const historical = JSON.parse(restoreArtworkBranchRaw(JSON.stringify(MCP)));
  const predecessor = removeArtworkAdditions(historical, "mcp-surface-v1");
  expect(predecessor.tools).toEqual(MCP.tools);
  expect(Object.keys(predecessor.toolDefinitions)).toEqual(
    Object.keys(MCP.toolDefinitions)
  );
  for (const name of [
    "mcp-surface-v1",
    "headless-protocol-v1",
    "contract-ownership-v1",
  ] as const) {
    const raw = readFileSync(
      new URL(`../../../contracts/${name}.json`, import.meta.url),
      "utf8"
    );
    expect(digest(restoreArtworkRaw(restoreArtworkBranchRaw(raw))), name).toBe(
      pins[name]
    );
  }
  for (const [name, expected] of Object.entries(pins.markers)) {
    const raw = readFileSync(
      new URL(`../../../contracts/${name}`, import.meta.url),
      "utf8"
    );
    expect(JSON.parse(raw).projectSchemaVersion).toBe(
      contract.projectSchemaVersion
    );
    expect(digest(restoreArtworkRaw(restoreArtworkBranchRaw(raw))), name).toBe(
      expected
    );
  }
  expect(digest(JSON.stringify(expandMcpSurfaceCatalog(historical)))).toBe(
    pins.expandedMcpSha256
  );
  expect(digest(JSON.stringify(expandMcpSurfaceCatalog(predecessor)))).toBe(
    pins.predecessorExpandedMcpSha256
  );
};
