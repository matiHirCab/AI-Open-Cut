import { writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import COMPONENTS from "../../../contracts/component-definitions-v1.json";
import { projectStateSchema, writeResultSchema } from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;

export const verifyComponentWorkflow = async (
  client: Client,
  call: Call,
  mediaDirectory: string
) => {
  const { projectId } = await call(
    "project_create",
    { name: "Components" },
    writeResultSchema
  );
  const { id: _definitionId, ...fields } = COMPONENTS.definition;
  const created = await call(
    "component_create",
    { ...fields, expectedRevision: 0, projectId },
    writeResultSchema
  );
  const [componentId] = created.changedIds;
  const nestedTrack = (target: string | undefined) => ({
    audioRole: "unassigned",
    ducking: null,
    hidden: false,
    id: "local",
    items: [
      {
        componentId: target,
        durationMs: 1000,
        hidden: false,
        id: "nested",
        stackOrder: 0,
        startMs: 0,
        timeScale: 1,
        transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
        trimStartMs: 0,
        type: "component_instance",
        zIndex: 0,
      },
    ],
    locked: false,
    muted: false,
    name: "Local",
    trackType: "overlay",
  });
  await call(
    "component_update",
    { ...fields, componentId, expectedRevision: 1, name: "Updated", projectId },
    writeResultSchema
  );
  const before = await call("project_open", { projectId }, projectStateSchema);
  expect(before.project.components[0]?.name).toBe("Updated");
  const cycle = await client.callTool({
    arguments: {
      ...fields,
      componentId,
      expectedRevision: 2,
      projectId,
      tracks: [nestedTrack(componentId)],
    },
    name: "component_update",
  });
  expect(cycle.isError).toBe(true);
  expect(cycle.structuredContent).toMatchObject({
    error: { code: "INVALID_ARGUMENT", retryable: false },
  });
  const failure = await client.callTool({
    arguments: {
      expectedRevision: 2,
      operations: [
        { ...fields, operation: "component_create", resultAlias: "leaf" },
        { componentId: "missing", operation: "component_delete" },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(failure.isError).toBe(true);
  expect(failure.structuredContent).toMatchObject({
    error: { code: "ITEM_NOT_FOUND", retryable: false },
  });
  expect(await call("project_open", { projectId }, projectStateSchema)).toEqual(
    before
  );
  await call(
    "timeline_batch_edit",
    {
      expectedRevision: 2,
      operations: [
        { ...fields, operation: "component_create", resultAlias: "leaf" },
        {
          ...fields,
          operation: "component_create",
          resultAlias: "consumer",
          tracks: [nestedTrack("@leaf")],
        },
        { componentId: "@consumer", operation: "component_delete" },
        { componentId: "@leaf", operation: "component_delete" },
        { componentId, operation: "component_delete" },
      ],
      projectId,
    },
    writeResultSchema
  );
  expect(
    (await call("project_open", { projectId }, projectStateSchema)).project
      .components
  ).toEqual([]);
  await call(
    "project_undo",
    { expectedRevision: 3, projectId },
    writeResultSchema
  );
  expect(
    (await call("project_open", { projectId }, projectStateSchema)).project
      .components
  ).toEqual(before.project.components);
  await call(
    "project_redo",
    { expectedRevision: 4, projectId },
    writeResultSchema
  );
  expect(
    (await call("project_open", { projectId }, projectStateSchema)).project
      .components
  ).toEqual([]);
  const sourcePath = join(mediaDirectory, "component-caption-source.mp4");
  writeFileSync(sourcePath, "component caption fixture");
  const imported = await call(
    "asset_import",
    { expectedRevision: 5, mediaType: "video", path: sourcePath, projectId },
    writeResultSchema
  );
  const [assetId] = imported.changedIds;
  const moved = COMPONENTS.itemValidationFixtures.find(
    (fixture) => fixture.id === "caption-moved-words"
  );
  if (!(assetId && moved)) {
    throw new Error(
      "component source asset or canonical caption fixture is missing"
    );
  }
  const replaceAsset = (operation: unknown) =>
    JSON.parse(
      JSON.stringify(operation).replaceAll("component-fixture-asset", assetId)
    ) as Record<string, unknown>;
  const { operation: _operation, ...populated } = replaceAsset(moved.operation);
  await call(
    "component_create",
    { ...populated, expectedRevision: 6, projectId },
    writeResultSchema
  );
  const populatedState = await call(
    "project_open",
    { projectId },
    projectStateSchema
  );
  await Promise.all(
    COMPONENTS.itemValidationFixtures
      .filter((entry) => !entry.valid && entry.mcpAccept)
      .map(async (fixture) => {
        const response = await client.callTool({
          arguments: {
            expectedRevision: 7,
            operations: [
              { ...fields, operation: "component_create" },
              replaceAsset(fixture.operation),
            ],
            projectId,
          },
          name: "timeline_batch_edit",
        });
        expect(response.isError, fixture.id).toBe(true);
        expect(response.structuredContent, fixture.id).toMatchObject({
          error: { code: "INVALID_ARGUMENT", retryable: false },
        });
        expect(
          await call("project_open", { projectId }, projectStateSchema)
        ).toEqual(populatedState);
      })
  );
};
