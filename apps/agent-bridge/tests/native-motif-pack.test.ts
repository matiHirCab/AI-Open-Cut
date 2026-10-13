import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { Client } from "@modelcontextprotocol/client";
import { InMemoryTransport, McpServer } from "@modelcontextprotocol/server";
import { expect, it } from "vitest";
import owners from "../../../contracts/contract-ownership-v1.json";
import recipe from "../../../contracts/native-motif-pack-v1.json";
import { headlessEditSchema } from "../src/schemas";
import { registerWorkflowPrompts } from "../src/server/context";
import pins from "./fixtures/native-motif-pack-predecessor-pins.json";
import {
  assertCurrentNativeMotifOwnership,
  removeNativeMotifOwnershipAddition,
  restoreNativeMotifMergedCatalogRaw,
  restoreNativeMotifRaw,
} from "./fixtures/native-motif-pack-projection";
import { verifyMotifPixels } from "./native-motif-pack-native";

it("preserves predecessor bytes and rejects partial or unrelated ownership drift", () => {
  const raw = readFileSync(
    new URL("../../../contracts/contract-ownership-v1.json", import.meta.url),
    "utf8"
  );
  assertCurrentNativeMotifOwnership(owners);
  const restored = restoreNativeMotifRaw(raw);
  expect(() => assertCurrentNativeMotifOwnership(JSON.parse(restored))).toThrow(
    "Missing"
  );
  expect(createHash("sha256").update(restored).digest("hex")).toBe(
    pins["contract-ownership-v1"]
  );
  expect(restoreNativeMotifRaw(restored)).toBe(restored);
  expect(removeNativeMotifOwnershipAddition(JSON.parse(restored))).toEqual(
    JSON.parse(restored)
  );
  const partial = structuredClone(owners);
  partial.categories.nativeMotifPack.consumers.pop();
  expect(() => restoreNativeMotifRaw(JSON.stringify(partial))).toThrow(
    "ownership drift"
  );
  const unrelated = structuredClone(owners);
  unrelated.version = 99;
  expect(() => restoreNativeMotifRaw(JSON.stringify(unrelated))).toThrow(
    "predecessor contract drift"
  );
  expect(() => removeNativeMotifOwnershipAddition({ categories: [] })).toThrow(
    "Malformed"
  );
});

it("retains exact MCP/headless catalogs and original hero source bytes", () => {
  for (const [path, pin] of [
    ["../../../contracts/mcp-surface-v1.json", pins["mcp-surface-v1"]],
    [
      "../../../contracts/headless-protocol-v1.json",
      pins["headless-protocol-v1"],
    ],
    ["../src/recipes/native-hero-card-v1.json", pins["native-hero-card-v1"]],
  ] as const) {
    expect(
      createHash("sha256")
        .update(
          restoreNativeMotifMergedCatalogRaw(
            readFileSync(new URL(path, import.meta.url), "utf8"),
            path.split("/").at(-1)?.replace(".json", "") ?? ""
          )
        )
        .digest("hex")
    ).toBe(pin);
  }
});

it("provides two bounded native phases through one deterministic read-only prompt", async () => {
  const server = new McpServer({ name: "motif-guidance", version: "1" });
  registerWorkflowPrompts(server);
  const client = new Client({ name: "motif-reader", version: "1" });
  const [s, c] = InMemoryTransport.createLinkedPair();
  try {
    await server.connect(s);
    await client.connect(c);
    const args = {
      arguments: { projectId: "p", request: "five native motifs" },
      name: "create_motion_graphics",
    };
    const result = await client.getPrompt(args);
    expect(await client.getPrompt(args)).toEqual(result);
    expect(result.messages).toHaveLength(1);
    const content = result.messages[0]?.content;
    if (content?.type !== "text") {
      throw new Error("Missing motif recipe");
    }
    const block = content.text
      .split("native-motif-pack-v1 recipe JSON:\n")[1]
      ?.split("\nEND native-motif-pack-v1 recipe JSON")[0];
    expect(JSON.parse(block ?? "null")).toEqual(recipe);
    const hero = JSON.parse(
      content.text.split("native-hero-card-v1 recipe JSON:\n")[1] ?? "null"
    );
    expect(hero.recipeId).toBe("native-hero-card-v1");
    const values: Record<string, string> = { $overlay: "overlay" };
    for (const motif of recipe.motifs) {
      values[motif.componentIdPlaceholder] = motif.id;
    }
    const resolve = (v: unknown) =>
      JSON.parse(JSON.stringify(v), (_key, x) =>
        typeof x === "string" && Object.hasOwn(values, x) ? values[x] : x
      );
    for (const motif of recipe.motifs) {
      headlessEditSchema.parse(motif.seed);
      headlessEditSchema.parse(resolve(motif.populate));
      expect(motif.populate.tracks[0]?.items.map((item) => item.id)).toEqual(
        motif.roles
      );
      expect(motif.seed.tracks).toEqual([]);
      expect(motif.seed.slots).toEqual([]);
    }
    for (const entry of [
      ...recipe.galleryInstances,
      ...recipe.aroundHeroInstances,
    ]) {
      headlessEditSchema.parse(resolve(entry));
    }
    expect(recipe.motifs.map((m) => m.id)).toEqual([
      "dot_backdrop",
      "sticker_contour",
      "sun_rays",
      "fireball",
      "speed_lines",
    ]);
    expect(recipe.motifs[1]?.parameters.strokeWidth).toBe(12);
    const paper = recipe.motifs[1]?.populate.tracks[0]?.items[0];
    expect(paper && "stroke" in paper ? paper.stroke?.width : undefined).toBe(
      12
    );
    expect(recipe.motifs.map((m) => m.expandedVisualLeaves)).toEqual([
      1, 2, 2, 2, 1,
    ]);
    expect(recipe.motifs.reduce((n, m) => n + m.populate.slots.length, 0)).toBe(
      9
    );
    expect(recipe.provenance.rasterMotifAssets).toEqual([]);
    expect(recipe.provenance.externalArtwork).toEqual([]);
    expect(recipe.provenance.tracing).toBe(false);
  } finally {
    await client.close();
    await server.close();
  }
});

it("rejects blank, truncated and wrong-paint native motif evidence", () => {
  const blank = Buffer.alloc(1920 * 1080 * 3);
  expect(() => verifyMotifPixels(blank, false, 3)).toThrow("pixel mismatch");
  expect(() => verifyMotifPixels(blank, true, 8)).toThrow("pixel mismatch");
  expect(() => verifyMotifPixels(blank.subarray(0, 99), false, 3)).toThrow(
    "frame size"
  );
  const white = Buffer.alloc(blank.length, 255);
  expect(() => verifyMotifPixels(white, false, 3)).toThrow("pixel mismatch");
});

it("rejects a controlled fireball whose outer paint is incorrectly ordered above its core", () => {
  const controlled = Buffer.alloc(1920 * 1080 * 3);
  for (const [x, y, rgb] of [
    [1180, 190, [255, 107, 0]],
    [1290, 180, [255, 204, 0]],
    [1180, 510, [255, 0, 0]],
    [1290, 500, [255, 255, 255]],
  ] as const) {
    controlled.set(rgb, (y * 1920 + x) * 3);
  }
  // This is a controlled oracle input, not captured native success evidence.
  verifyMotifPixels(controlled, false, 3, 3);
  verifyMotifPixels(controlled, false, 8, 3, "encodedInteriors");
  controlled.set([255, 107, 0], (180 * 1920 + 1290) * 3);
  controlled.set([255, 0, 0], (500 * 1920 + 1290) * 3);
  expect(() => verifyMotifPixels(controlled, false, 3, 3)).toThrow(
    "pixel mismatch"
  );
  expect(() =>
    verifyMotifPixels(controlled, false, 8, 3, "encodedInteriors")
  ).toThrow("pixel mismatch");
});

it("rejects blank, wrong-paint and corrupted fixed encoded dot interiors", () => {
  const controlled = Buffer.alloc(1920 * 1080 * 3);
  for (const [left, top, rgb] of [
    [99, 99, [0, 156, 188]],
    [99, 419, [188, 188, 188]],
  ] as const) {
    for (let y = top; y < top + 2; y += 1) {
      for (let x = left; x < left + 2; x += 1) {
        controlled.set(rgb, (y * 1920 + x) * 3);
      }
    }
  }
  // Geometry-defined controlled input only; this is not native artifact evidence.
  verifyMotifPixels(controlled, false, 8, 0, "encodedInteriors");
  expect(() =>
    verifyMotifPixels(
      Buffer.alloc(controlled.length),
      false,
      8,
      0,
      "encodedInteriors"
    )
  ).toThrow("pixel mismatch");
  const wrong = Buffer.from(controlled);
  for (let y = 99; y < 101; y += 1) {
    for (let x = 99; x < 101; x += 1) {
      wrong.set([0, 255, 0], (y * 1920 + x) * 3);
    }
  }
  expect(() =>
    verifyMotifPixels(wrong, false, 8, 0, "encodedInteriors")
  ).toThrow("pixel mismatch");
  controlled.set([0, 0, 0], (99 * 1920 + 99) * 3);
  expect(() =>
    verifyMotifPixels(controlled, false, 8, 0, "encodedInteriors")
  ).toThrow("pixel mismatch");
});

it("uses the dedicated visibility operation when isolating native occurrences", () => {
  headlessEditSchema.parse({
    hidden: true,
    itemId: "occurrence",
    operation: "set_item_visibility",
  });
  expect(
    headlessEditSchema.safeParse({
      hidden: true,
      itemId: "occurrence",
      operation: "update_item",
    }).success
  ).toBe(false);
  headlessEditSchema.parse({
    itemId: "occurrence",
    operation: "update_item",
    transform2d: recipe.galleryInstances[0]?.transform2d,
  });
});
