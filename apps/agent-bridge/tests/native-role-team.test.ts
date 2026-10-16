import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { Client } from "@modelcontextprotocol/client";
import { InMemoryTransport, McpServer } from "@modelcontextprotocol/server";
import { expect, it } from "vitest";
import owners from "../../../contracts/contract-ownership-v1.json";
import recipe from "../../../contracts/native-role-team-v1.json";
import { headlessEditSchema } from "../src/schemas";
import { registerWorkflowPrompts } from "../src/server/context";
import { restoreNarrationProfileRaw } from "./fixtures/narration-profiles-projection";
import pins from "./fixtures/native-role-team-predecessor-pins.json";
import {
  assertCurrentRoleTeamOwnership,
  removeRoleTeamOwnershipAddition,
  restoreRoleTeamOwnershipRaw,
} from "./fixtures/native-role-team-projection";
import { removeRenderBudgetOwnershipAddition } from "./fixtures/render-budget-projection";
import { removeSolidGraphicOwnershipAddition } from "./fixtures/solid-graphic-color-slots-projection";
import {
  verifyEncodedTeamPixels,
  verifyTeamPixels,
} from "./native-role-team-native";

const paintControlRegion = (
  data: Buffer,
  x: number,
  y: number,
  width: number,
  height: number,
  rgb: readonly number[]
) => {
  for (let dy = 0; dy < height; dy += 1) {
    for (let dx = 0; dx < width; dx += 1) {
      for (let c = 0; c < 3; c += 1) {
        data[((y + dy) * 1920 + x + dx) * 3 + c] = rgb[c] ?? 0;
      }
    }
  }
};

it("preserves exact cumulative predecessor ownership and effective corruption controls", () => {
  const raw = readFileSync(
    new URL("../../../contracts/contract-ownership-v1.json", import.meta.url),
    "utf8"
  );
  assertCurrentRoleTeamOwnership(owners);
  const restored = restoreRoleTeamOwnershipRaw(raw);
  expect(createHash("sha256").update(restored).digest("hex")).toBe(
    pins["contract-ownership-v1"]
  );
  expect(restoreRoleTeamOwnershipRaw(restored)).toBe(restored);
  expect(removeRoleTeamOwnershipAddition(JSON.parse(restored))).toEqual(
    JSON.parse(restored)
  );
  expect(() => assertCurrentRoleTeamOwnership(JSON.parse(restored))).toThrow(
    "Missing"
  );
  const partial = structuredClone(owners);
  partial.categories.nativeRoleTeam.consumers.pop();
  expect(() => restoreRoleTeamOwnershipRaw(JSON.stringify(partial))).toThrow(
    "incomplete"
  );
  const drift = JSON.parse(restored);
  drift.reviewer = "unexpected";
  expect(() => restoreRoleTeamOwnershipRaw(JSON.stringify(drift))).toThrow(
    "Unrelated"
  );
  expect(() => removeRoleTeamOwnershipAddition({ categories: [] })).toThrow(
    "Malformed"
  );
  const unchanged = JSON.stringify(owners);
  for (const project of [
    removeRenderBudgetOwnershipAddition,
    removeSolidGraphicOwnershipAddition,
  ]) {
    expect(project(owners).categories).not.toHaveProperty("nativeRoleTeam");
    expect(() => project(partial)).toThrow("incomplete");
    const unrelated = JSON.parse(unchanged);
    unrelated.reviewer = "unexpected";
    expect(project(unrelated).reviewer).toBe("unexpected");
  }
  expect(JSON.stringify(owners)).toBe(unchanged);
});
it("retains exact prior wire catalogs hero and motif source", () => {
  for (const [name, path] of [
    ["headless-protocol-v1", "../../../contracts/headless-protocol-v1.json"],
    ["mcp-surface-v1", "../../../contracts/mcp-surface-v1.json"],
    ["native-hero-card-v1", "../src/recipes/native-hero-card-v1.json"],
    ["native-motif-pack-v1", "../../../contracts/native-motif-pack-v1.json"],
  ] as const) {
    expect(
      createHash("sha256")
        .update(
          restoreNarrationProfileRaw(
            readFileSync(new URL(path, import.meta.url), "utf8")
          )
        )
        .digest("hex")
    ).toBe(pins[name]);
  }
});
it("publishes all supported explicit count layouts through existing typed operations", () => {
  const resolve = (value: unknown) =>
    JSON.parse(JSON.stringify(value), (_key, x) => {
      if (x === "$overlay") {
        return "overlay";
      }
      return x === "$artwork" ? "image" : x;
    });
  expect(recipe.roles).toEqual([
    "background",
    "device_frame",
    "screen_panel",
    "label_panel",
    "accent",
    "artwork",
    "role_label",
    "rays",
    "sun_disc",
  ]);
  expect(recipe.badgeCreate.tracks[0]?.items.length).toBe(9);
  expect(
    recipe.badgeDefineSlots.slots.filter((s) => s.kind === "color")
  ).toHaveLength(7);
  expect(recipe.caps).toMatchObject({
    maximumAnimationChannels: 4,
    maximumMembers: 4,
    maximumVisualLeaves: 36,
    minimumMembers: 1,
  });
  for (const [layout, counts] of Object.entries(recipe.layouts)) {
    expect(Object.keys(counts)).toEqual(["1", "2", "3", "4"]);
    for (const [size, operations] of Object.entries(counts)) {
      for (const operation of [
        recipe.badgeCreate,
        recipe.badgeDefineSlots,
        ...operations,
      ]) {
        headlessEditSchema.parse(resolve(operation));
      }
      const members = operations.filter(
        (x) => x.operation === "add_component_instance"
      );
      expect(members).toHaveLength(Number(size));
      members.forEach((member, i) => {
        expect(member.resultAlias).toBe(`member_${i + 1}`);
        expect(member.transform).toEqual({
          opacity: 1,
          positionX: layout === "row" ? 100 + 450 * i : 480 + 560 * (i % 2),
          positionY: layout === "row" ? 390 : 160 + 400 * Math.floor(i / 2),
          scale: 0.5,
        });
      });
    }
  }
});
it("retrieves one deterministic read-only role block while preserving original final hero reader", async () => {
  const server = new McpServer({ name: "role-guidance", version: "1" });
  registerWorkflowPrompts(server);
  const client = new Client({ name: "role-reader", version: "1" });
  const [s, c] = InMemoryTransport.createLinkedPair();
  try {
    await server.connect(s);
    await client.connect(c);
    const prompt = await client.getPrompt({
      arguments: { projectId: "p", request: "native role team" },
      name: "create_motion_graphics",
    });
    expect(prompt.messages).toHaveLength(1);
    const content = prompt.messages[0]?.content;
    if (content?.type !== "text") {
      throw new Error("Missing read-only team guidance");
    }
    expect(
      JSON.parse(
        content.text
          .split("native-role-team-v1 recipe JSON:\n")[1]
          ?.split("\nEND native-role-team-v1 recipe JSON")[0] ?? "null"
      )
    ).toEqual(recipe);
    expect(
      JSON.parse(
        content.text.split("native-hero-card-v1 recipe JSON:\n")[1] ?? "null"
      ).recipeId
    ).toBe("native-hero-card-v1");
    expect(
      JSON.parse(
        content.text
          .split("native-motif-pack-v1 recipe JSON:\n")[1]
          ?.split("\nEND native-motif-pack-v1 recipe JSON")[0] ?? "null"
      ).recipeId
    ).toBe("native-motif-pack-v1");
  } finally {
    await client.close();
    await server.close();
  }
});
it("rejects blank truncated wrong-paint and false temporal evidence with independent valid controls", () => {
  const data = Buffer.alloc(1920 * 1080 * 3);
  const point = (x: number, y: number, rgb: readonly number[]) => {
    for (let c = 0; c < 3; c += 1) {
      data[(y * 1920 + x) * 3 + c] = rgb[c] ?? 0;
    }
  };
  const backgrounds = [
    [24, 38, 61],
    [32, 64, 64],
    [64, 32, 64],
    [64, 48, 32],
  ];
  for (let i = 0; i < 4; i += 1) {
    const x = 480 + 560 * (i % 2),
      y = 160 + 400 * Math.floor(i / 2);
    point(x + 5, y + 5, backgrounds[i] ?? []);
    for (const [dx, dy, rgb] of [
      [25, 25, [238, 238, 238]],
      [35, 35, [22, 32, 51]],
      [25, 250, [48, 73, 104]],
      [40, 220, [255, 204, 51]],
      [180, 130, [255, 255, 255]],
      [230, 130, [255, 0, 0]],
      [320, 60, [255, 107, 0]],
      [320, 25, [255, 204, 0]],
    ] as const) {
      point(x + dx, y + dy, rgb);
    }
    for (let dx = 32; dx < 72; dx += 1) {
      point(x + dx, y + 248, [255, 255, 255]);
    }
  }
  // Synthetic verification-function controls are not claimed as rendered artifacts.
  verifyTeamPixels(data, 4, 500);
  expect(() => verifyTeamPixels(Buffer.alloc(data.length), 4, 500)).toThrow();
  expect(() =>
    verifyTeamPixels(data.subarray(0, data.length - 1), 4, 500)
  ).toThrow();
  expect(() => verifyTeamPixels(data, 4, 0, 8)).toThrow();
  for (let i = 0; i < 4; i += 1) {
    const x = 480 + 560 * (i % 2),
      y = 160 + 400 * Math.floor(i / 2);
    point(x + 27, y + 25, [238, 238, 238]);
    paintControlRegion(data, x + 45, y + 219, 60, 5, [255, 204, 51]);
    paintControlRegion(data, x + 317, y + 14, 6, 16, [255, 204, 0]);
  }
  verifyEncodedTeamPixels(data, 4, 500);
  const absentAccent = Buffer.from(data);
  paintControlRegion(absentAccent, 525, 379, 60, 5, [238, 238, 238]);
  expect(() => verifyEncodedTeamPixels(absentAccent, 4, 500)).toThrow();
  const absentRays = Buffer.from(data);
  paintControlRegion(absentRays, 797, 174, 6, 16, [238, 238, 238]);
  expect(() => verifyEncodedTeamPixels(absentRays, 4, 500)).toThrow();
  const absentLabel = Buffer.from(data);
  paintControlRegion(absentLabel, 512, 408, 335, 34, [48, 73, 104]);
  expect(() => verifyEncodedTeamPixels(absentLabel, 4, 500)).toThrow();
  point(485, 165, [255, 255, 255]);
  expect(() => verifyTeamPixels(data, 4, 500)).toThrow();
});
