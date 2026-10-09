import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import catalog from "../../../contracts/mcp-surface-v1.json";

export const verifyMotionWorkflowPrompt = async (client: Client) => {
  const listed = await client.listPrompts();
  expect(listed.prompts.map((prompt) => prompt.name).sort()).toEqual(
    catalog.prompts
  );
  expect(
    listed.prompts.find((prompt) => prompt.name === "create_motion_graphics")
      ?.arguments
  ).toEqual([
    { name: "projectId", required: true },
    { name: "request", required: true },
  ]);
  const args = {
    projectId: 'project"\nwith-data',
    request: 'Build a title "rescue"\nscene',
  };
  const result = await client.getPrompt({
    arguments: args,
    name: "create_motion_graphics",
  });
  expect(
    await client.getPrompt({ arguments: args, name: "create_motion_graphics" })
  ).toEqual(result);
  expect(result.messages).toHaveLength(1);
  expect(result.messages[0]?.role).toBe("user");
  const content = result.messages[0]?.content;
  expect(content?.type).toBe("text");
  if (content?.type !== "text") {
    throw new Error("Expected textual workflow guidance");
  }
  expect(content.text).toContain(JSON.stringify(args.projectId));
  expect(content.text).toContain(JSON.stringify(args.request));
  for (const tool of [
    "editor_get_status",
    "project_get_state",
    "component_create",
    "component_define_slots",
    "add_component_instance",
    "marker_create",
    "timeline_batch_edit",
    "timeline_apply_animation_preset",
    "preview_review_range",
    "job_get_status",
    "project_undo",
    "project_redo",
    "project_open",
    "project_export_video",
  ]) {
    expect(Object.hasOwn(catalog.toolDefinitions, tool), tool).toBe(true);
    expect(content.text, tool).toContain(tool);
  }
  for (const concept of [
    "resultAlias",
    "slotValues",
    "marker-relative",
    "integer-millisecond",
    "expectedRevision",
    "REVISION_CONFLICT",
    "atomic rollback",
    "unchanged content and revision",
    "includeAudio=true",
    "artifactResource.uri",
    "immutable revision",
    "overwrite",
    "explicit permission",
    "never blindly retry",
    "does not execute edits",
  ]) {
    expect(content.text, concept).toContain(concept);
  }
  await Promise.all(
    [
      {},
      { projectId: "p" },
      { request: "r" },
      { projectId: "", request: "r" },
      { projectId: "p", request: "" },
    ].map((invalid) =>
      expect(
        client.getPrompt({ arguments: invalid, name: "create_motion_graphics" })
      ).rejects.toThrow()
    )
  );
  await expect(
    client.getPrompt({ arguments: args, name: "missing_prompt" })
  ).rejects.toThrow();
};
