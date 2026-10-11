import { Client } from "@modelcontextprotocol/client";
import { InMemoryTransport, McpServer } from "@modelcontextprotocol/server";
import { expect, it } from "vitest";
import { headlessEditSchema } from "../src/schemas";
import { registerWorkflowPrompts } from "../src/server/context";
import { verifyHeroPixels } from "./native-hero-workflow";

it("retrieves a versioned native hero through unchanged read-only MCP discovery", async () => {
  const server = new McpServer({ name: "hero-guidance", version: "1" });
  registerWorkflowPrompts(server);
  const client = new Client({ name: "hero-reader", version: "1" });
  const [s, c] = InMemoryTransport.createLinkedPair();
  try {
    await server.connect(s);
    await client.connect(c);
    const prompt = (await client.listPrompts()).prompts.find(
      (p) => p.name === "create_motion_graphics"
    );
    expect(prompt?.description).toContain("native-hero-card-v1");
    expect(prompt?.arguments).toEqual([
      { name: "projectId", required: true },
      { name: "request", required: true },
    ]);
    const args = {
      arguments: { projectId: "p", request: "native hero" },
      name: "create_motion_graphics",
    };
    const result = await client.getPrompt(args);
    expect(await client.getPrompt(args)).toEqual(result);
    const content = result.messages[0]?.content;
    if (content?.type !== "text") {
      throw new Error("Missing textual recipe");
    }
    const recipe = JSON.parse(
      content.text.split("native-hero-card-v1 recipe JSON:\n")[1] ?? "null"
    );
    expect(recipe.recipeId).toBe("native-hero-card-v1");
    expect(recipe.version).toBe(1);
    expect(recipe.canvas).toEqual({ fps: 30, height: 1080, width: 1920 });
    expect(recipe.caps.expandedVisualLeaves).toBe(14);
    expect(recipe.roles).toEqual([
      "background",
      "device_frame",
      "screen_panel",
      "label_panel",
      "accent",
      "artwork",
      "title",
    ]);
    const operations = JSON.parse(
      JSON.stringify(recipe.operations),
      (_key, value) =>
        typeof value === "string" &&
        Object.hasOwn({ $artwork: "asset0", $overlay: "overlay" }, value)
          ? (
              { $artwork: "asset0", $overlay: "overlay" } as Record<
                string,
                string
              >
            )[value]
          : value
    );
    for (const operation of operations) {
      headlessEditSchema.parse(operation);
    }
    expect(operations.map((o: { operation: string }) => o.operation)).toEqual([
      "component_create",
      "add_component_instance",
      "add_component_instance",
      "set_animation_channels",
    ]);
    expect(
      operations[0].tracks[0].items.map((i: { type: string }) => i.type)
    ).toEqual(["shape", "shape", "shape", "shape", "shape", "media", "text"]);
    expect(
      operations[0].slots.map((slot: { kind: string }) => slot.kind)
    ).toEqual(["text", "asset"]);
    expect(operations[3].animationChannels[0]).toMatchObject({
      keyframes: [
        { timeMs: 0, value: { type: "scalar", value: 0 } },
        { timeMs: 100, value: { type: "scalar", value: 1 } },
      ],
      property: "transform.opacity",
    });
  } finally {
    await client.close();
    await server.close();
  }
});

it("rejects blank and incorrect native hero evidence independently", () => {
  const blank = Buffer.alloc(1920 * 1080 * 3);
  expect(() => verifyHeroPixels(blank, false, 3)).toThrow();
  expect(() => verifyHeroPixels(blank, true, 8)).toThrow();
  expect(() => verifyHeroPixels(blank.subarray(0, 100), false, 3)).toThrow();
});
