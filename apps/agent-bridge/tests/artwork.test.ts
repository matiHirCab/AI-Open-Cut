import { expect, it } from "vitest";
import contract from "../../../contracts/artwork-preparation-v1.json";
import ownership from "../../../contracts/contract-ownership-v1.json";
import MCP from "../../../contracts/mcp-surface-v1.json";
import {
  artworkManifestSchema,
  artworkParametersSchema,
} from "../src/artwork/contract";
import {
  assertArtworkConsumer,
  verifyArtworkContract,
} from "./artwork-contract";
import { removeArtworkIntegration } from "./fixtures/artwork-integration-projection";
import { removeArtworkAdditions } from "./fixtures/artwork-projection";
import { removeShortPlanningOwnership } from "./fixtures/short-candidates-projection";

it(
  "verifies independent canonical artwork payloads and precise historical inverse",
  verifyArtworkContract
);

it("keeps omitted cleanup absent and rejects malformed strict controls", () => {
  const outline = { color: [0, 0, 0, 255], radius: 0 };
  expect(artworkParametersSchema.parse({ outline })).not.toHaveProperty(
    "cleanup"
  );
  for (const value of [
    { cleanup: null, outline },
    { outline: { ...outline, radius: 17 } },
    { outline: { ...outline, radius: 0.5 } },
    { outline: { ...outline, color: [0, 0, 256, 255] } },
    { outline: { ...outline, color: [0, 0, 0] } },
    { cleanup: { color: [255, 255, 255], tolerance: -1 }, outline },
    { cleanup: { color: [255, 255, 255], tolerance: 256 }, outline },
    { cleanup: { color: [255, 255, 255], extra: true, tolerance: 1 }, outline },
    { extra: true, outline },
  ]) {
    expect(artworkParametersSchema.safeParse(value).success).toBe(false);
  }
  for (const field of ["sourceHash", "resultHash"] as const) {
    expect(
      artworkManifestSchema.safeParse({ ...contract.manifest, [field]: "bad" })
        .success
    ).toBe(false);
  }
  expect(
    artworkManifestSchema.safeParse({ ...contract.manifest, extra: true })
      .success
  ).toBe(false);
});

it("retains unrelated matching data and independent planning additions in either inverse order", () => {
  const source = structuredClone(MCP);
  (
    source.toolDefinitions.editor_get_status.annotations as Record<
      string,
      unknown
    >
  ).artworkDerivation = { version: 47 };
  const previous = removeArtworkIntegration(source);
  expect(previous.toolDefinitions.editor_get_status.annotations).toMatchObject({
    artworkDerivation: { version: 47 },
  });
  expect(source.$defs.ProjectPropertiesAssetsItemsProperties).toHaveProperty(
    "artworkDerivation"
  );
  const categories = {
    categories: {
      artworkPreparation: contract.ownership,
      sourceGroundedShortPlanning: {
        canonical: "contracts/short-candidate-planning-cli-v1.json",
        consumers: [] as string[],
      },
    },
  };
  // Use the actual independently governed peer category, not an invented acceptance placeholder.
  categories.categories.sourceGroundedShortPlanning =
    ownership.categories.sourceGroundedShortPlanning;
  const left = removeShortPlanningOwnership(
    removeArtworkIntegration(categories)
  );
  const right = removeArtworkIntegration(
    removeShortPlanningOwnership(categories)
  );
  expect(left).toEqual(right);
  expect(left).toEqual({ categories: {} });
  expect(removeArtworkIntegration(categories).categories).toHaveProperty(
    "sourceGroundedShortPlanning"
  );
  expect(removeShortPlanningOwnership(categories).categories).toHaveProperty(
    "artworkPreparation"
  );
});

it("rejects missing changed or narrowed artwork schema additions without erasing unrelated drift", () => {
  for (const mutate of [
    (value: typeof MCP) =>
      Reflect.deleteProperty(
        value.$defs.ProjectPropertiesAssetsItemsProperties,
        "artworkDerivation"
      ),
    (value: typeof MCP) => {
      value.$defs.ProjectPropertiesAssetsItemsProperties.artworkDerivation.properties.sourceWidth.maximum = 511;
    },
    (value: typeof MCP) => {
      value.toolDefinitions.editor_get_status.outputSchema.properties.projectSchemaVersion.const = 49;
    },
  ]) {
    const value = structuredClone(MCP);
    mutate(value);
    expect(() => removeArtworkIntegration(value)).toThrow();
  }
});

it("requires live artwork parity and preserves peer operations", () => {
  const source =
    'import { verifyArtworkContract as verify } from "./artwork-contract";\nit("contract", verify);';
  expect(() => assertArtworkConsumer(source)).not.toThrow();
  for (const masked of [
    "",
    `/* ${source} */`,
    source.replace('it("contract"', 'it.skip("contract"'),
    source.replace('it("contract"', 'it.todo("contract"'),
  ]) {
    expect(() => assertArtworkConsumer(masked)).toThrow("absent or masked");
  }
  const catalog = {
    operations: ["commit_artwork", "peer_operation", "prepare_artwork"],
  };
  expect(
    removeArtworkAdditions(catalog, "headless-protocol-v1").operations
  ).toEqual(["peer_operation"]);
  expect(catalog.operations).toHaveLength(3);
});
