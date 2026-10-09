import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { Client } from "@modelcontextprotocol/client";
import { InMemoryTransport, McpServer } from "@modelcontextprotocol/server";
import { expect, it } from "vitest";
import catalog from "../../../contracts/mcp-surface-v1.json";
import { registerWorkflowPrompts } from "../src/server/context";
import { removeMotionWorkflowPrompt } from "./fixtures/motion-workflow-projection";
import { verifyMotionWorkflowPrompt } from "./motion-workflow-prompt";

it("discovers and validates deterministic read-only motion workflow guidance over MCP", async () => {
  // No editor/provider dependencies are passed: retrieving guidance cannot edit.
  const server = new McpServer({ name: "prompt-server", version: "1" });
  registerWorkflowPrompts(server);
  const client = new Client({ name: "prompt-client", version: "1" });
  const [s, c] = InMemoryTransport.createLinkedPair();
  try {
    await server.connect(s);
    await client.connect(c);
    await verifyMotionWorkflowPrompt(client);
    await expect(
      client.getPrompt({
        // Malformed wire data independently exercises the SDK's string typing.
        arguments: { projectId: 42, request: "r" } as unknown as Record<
          string,
          string
        >,
        name: "create_motion_graphics",
      })
    ).rejects.toThrow();
    expect(
      (
        await client.getPrompt({
          arguments: { text: "Hi" },
          name: "create_intro_video",
        })
      ).messages[0]?.content.type
    ).toBe("text");
  } finally {
    await client.close();
    await server.close();
  }
});

it("removes exactly the additive prompt and preserves independently captured predecessor bytes", () => {
  const previous = removeMotionWorkflowPrompt(catalog);
  expect(previous.prompts).toEqual([
    "add_narration",
    "assemble_clips",
    "create_intro_video",
    "transcribe_and_caption",
  ]);
  expect({ ...previous, prompts: catalog.prompts }).toEqual(catalog);
  const raw = readFileSync(
    new URL("../../../contracts/mcp-surface-v1.json", import.meta.url),
    "utf8"
  );
  const predecessor = raw.replace('    "create_motion_graphics",\n', "");
  expect(JSON.parse(predecessor)).toEqual(previous);
  expect(createHash("sha256").update(predecessor).digest("hex")).toBe(
    "5eff94792ee8f029910c655365d7690631b443dc506e30d7f74194c5d0e504bd"
  );
  expect(() =>
    removeMotionWorkflowPrompt({
      prompts: ["create_motion_graphics", "create_motion_graphics"],
    })
  ).toThrow();
  expect(() => removeMotionWorkflowPrompt({ prompts: null })).toThrow();
  expect(removeMotionWorkflowPrompt(previous)).toEqual(previous);
});
