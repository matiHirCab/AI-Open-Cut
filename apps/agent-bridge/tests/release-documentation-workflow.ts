import { readFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import {
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";
import {
  bindDocumentationArguments,
  readDocumentationExample,
} from "./release-documentation-fixture";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;
export const verifyReleaseDocumentationWorkflow = async (
  client: Client,
  call: Call,
  projectsDirectory: string
) => {
  const example = readDocumentationExample();
  const status = await call(
    "editor_get_status",
    { protocolVersion: example.protocolVersion },
    statusSchema
  );
  expect(status.protocolVersion).toBe(example.protocolVersion);
  expect(status.projectSchemaVersion).toBe(example.projectSchemaVersion);
  expect(status.subsystems.editor.ready).toBe(true);
  expect(status.subsystems.editor.capabilities).toEqual(
    expect.arrayContaining(example.requiredEditorCapabilities)
  );
  const discovered = new Set(
    (await client.listTools()).tools.map((tool) => tool.name)
  );
  for (const name of [
    example.standalone.tool,
    example.batch.tool,
    ...example.failures.map((entry) => entry.tool),
    "add_component_instance",
    "component_instance_update",
    "tts_get_status",
    "speech_generate_and_insert",
    "tts_generate_and_insert",
    "tts_commit_generated_artifact",
    "preview_render_frame",
    "preview_render_range",
    "preview_review_range",
    "project_export_video",
    "audio_analyze_mix",
    "component_define_slots",
  ]) {
    expect(discovered.has(name), `documented tool ${name}`).toBe(true);
  }
  const created = await call(
    "project_create",
    example.projectCreate,
    writeResultSchema
  );
  expect(created.revision).toBe(0);
  const { projectId } = created;
  const initial = await call(
    "project_get_state",
    { projectId },
    projectStateSchema
  );
  const overlayTrackId = initial.project.tracks.find(
    (track) => track.trackType === "overlay"
  )?.id;
  if (!overlayTrackId) {
    throw new Error("documentation project has no overlay track");
  }
  const bindings: Record<string, string> = { overlayTrackId, projectId };
  const standalone = await call(
    example.standalone.tool,
    bindDocumentationArguments(example.standalone.arguments, bindings),
    writeResultSchema
  );
  expect(standalone.revision).toBe(1);
  const [standaloneComponentId] = standalone.changedIds;
  if (!standaloneComponentId) {
    throw new Error("documentation definition ID missing");
  }
  bindings.standaloneComponentId = standaloneComponentId;
  const beforeBatch = await call(
    "project_get_state",
    { projectId },
    projectStateSchema
  );
  const dir = join(projectsDirectory, projectId);
  const history = () =>
    JSON.parse(readFileSync(join(dir, "history.json"), "utf8")) as {
      undo: unknown[];
      redo: unknown[];
    };
  const beforeUndoCount = history().undo.length;
  const batch = await call(
    example.batch.tool,
    bindDocumentationArguments(example.batch.arguments, bindings),
    writeResultSchema
  );
  expect(batch.revision).toBe(standalone.revision + 1);
  expect(history().undo).toHaveLength(beforeUndoCount + 1);
  expect(history().redo).toHaveLength(0);
  const definitionId = batch.aliases.definition;
  const instanceId = batch.aliases.instance;
  expect(definitionId).toBeTruthy();
  expect(instanceId).toBeTruthy();
  expect(definitionId).not.toBe("@definition");
  expect(instanceId).not.toBe("@instance");
  const authored = await call(
    "project_get_state",
    { projectId },
    projectStateSchema
  );
  expect(authored.project.components.map((component) => component.id)).toEqual(
    expect.arrayContaining([standaloneComponentId, definitionId])
  );
  expect(
    authored.project.tracks.flatMap((track) => track.items)
  ).toContainEqual(
    expect.objectContaining({
      componentId: definitionId,
      durationMs: 500,
      id: instanceId,
      slotValues: { opacity: { type: "number", value: 0.6 } },
      startMs: 250,
      type: "component_instance",
    })
  );
  const snapshot = () =>
    ["project.json", "history.json"].map((name) =>
      readFileSync(join(dir, name))
    );
  const beforeFailures = snapshot();
  await example.failures.reduce(async (previous, entry) => {
    await previous;
    const rejected = await client.callTool({
      arguments: bindDocumentationArguments(entry.arguments, bindings),
      name: entry.tool,
    });
    expect(rejected.isError, entry.id).toBe(true);
    expect(rejected.structuredContent, entry.id).toMatchObject({
      error: entry.expectedError,
    });
    expect(snapshot(), entry.id).toEqual(beforeFailures);
  }, Promise.resolve());
  const undo = await call(
    "project_undo",
    { expectedRevision: batch.revision, projectId },
    writeResultSchema
  );
  expect(undo.revision).toBe(batch.revision + 1);
  const restored = await call(
    "project_get_state",
    { projectId },
    projectStateSchema
  );
  expect(restored.project.components).toEqual(beforeBatch.project.components);
  expect(restored.project.tracks).toEqual(beforeBatch.project.tracks);
  expect(history().undo).toHaveLength(beforeUndoCount);
  expect(history().redo).toHaveLength(1);
  const redo = await call(
    "project_redo",
    { expectedRevision: undo.revision, projectId },
    writeResultSchema
  );
  expect(redo.revision).toBe(undo.revision + 1);
  const redone = await call(
    "project_get_state",
    { projectId },
    projectStateSchema
  );
  expect(redone.project.components).toEqual(authored.project.components);
  expect(redone.project.tracks).toEqual(authored.project.tracks);
  expect(history().undo).toHaveLength(beforeUndoCount + 1);
  expect(history().redo).toHaveLength(0);
  const beforeReopen = snapshot();
  expect(await call("project_open", { projectId }, projectStateSchema)).toEqual(
    redone
  );
  expect(snapshot()).toEqual(beforeReopen);
};
