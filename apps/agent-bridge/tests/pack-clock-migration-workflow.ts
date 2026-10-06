import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import PACK from "../../../contracts/initial-motion-preset-pack-v1.json";
import { projectStateSchema, writeResultSchema } from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;
interface SavedDocument {
  components: {
    tracks: { items: { id: string; animationPresetProvenance?: unknown }[] }[];
  }[];
  schemaVersion: number;
}
interface SavedHistory {
  redo: SavedDocument[];
  undo: SavedDocument[];
}

export const verifyPackClockMigrationWorkflow = async (
  client: Client,
  call: Call,
  projectsDirectory: string
) => {
  const { projectId } = await call(
    "project_create",
    { name: "Pack clock migration" },
    writeResultSchema
  );
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const edit = (
    name: string,
    expectedRevision: number,
    input: Record<string, unknown>
  ) => call(name, { expectedRevision, projectId, ...input }, writeResultSchema);
  const trackId = (await read()).project.tracks[1]?.id;
  const created = await edit("timeline_batch_edit", 0, {
    operations: PACK.presets.flatMap((entry) => [
      {
        color: "#ffffff",
        durationMs: 1000,
        height: 32,
        operation: "add_rectangle",
        resultAlias: entry.id,
        startMs: 0,
        trackId,
        transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
        width: 32,
      },
      {
        itemId: `@${entry.id}`,
        operation: "apply_animation_preset",
        parameters: entry.parameters,
        presetId: entry.id,
        presetVersion: 1,
      },
    ]),
  });
  const seeded = await read();
  const rootItems = seeded.project.tracks[1]?.items ?? [];
  expect(rootItems).toHaveLength(5);
  await edit("component_create", 1, {
    durationMs: 1000,
    height: 240,
    name: "All migrated Pack sources",
    slots: [],
    tracks: [
      {
        audioRole: "unassigned",
        ducking: null,
        hidden: false,
        id: "local",
        items: rootItems,
        locked: false,
        muted: false,
        name: "Pack",
        trackType: "overlay",
      },
    ],
    width: 320,
  });
  const itemId = created.aliases.impact_slam;
  await edit("timeline_move_item", 2, { itemId, startMs: 1000, trackId });
  await edit("project_undo", 3, {});
  const projectPath = join(projectsDirectory, projectId, "project.json");
  const historyPath = join(projectsDirectory, projectId, "history.json");
  const originalProject = JSON.parse(
    readFileSync(projectPath, "utf8")
  ) as SavedDocument;
  const originalHistory = JSON.parse(
    readFileSync(historyPath, "utf8")
  ) as SavedHistory;
  // Raw component authoring clears attribution; this saved-generation fixture restores
  // authenticated source records to exercise the existing persisted schema-30 contract.
  for (const document of [
    originalProject,
    ...originalHistory.undo,
    ...originalHistory.redo,
  ]) {
    for (const component of document.components) {
      for (const track of component.tracks) {
        for (const item of track.items) {
          item.animationPresetProvenance = rootItems.find(
            (sourceItem) => sourceItem.id === item.id
          )?.animationPresetProvenance;
        }
      }
    }
  }
  expect(originalHistory.undo.length).toBeGreaterThan(0);
  expect(originalHistory.redo.length).toBeGreaterThan(0);
  const oldProject = structuredClone(originalProject);
  const oldHistory = structuredClone(originalHistory);
  oldProject.schemaVersion = 30;
  for (const snapshot of [...oldHistory.undo, ...oldHistory.redo]) {
    snapshot.schemaVersion = 30;
  }
  writeFileSync(projectPath, JSON.stringify(oldProject));
  writeFileSync(historyPath, JSON.stringify(oldHistory));
  const migrated = await read();
  expect(migrated.project.schemaVersion).toBe(34);
  expect(JSON.parse(readFileSync(projectPath, "utf8"))).toEqual(
    originalProject
  );
  expect(JSON.parse(readFileSync(historyPath, "utf8"))).toEqual(
    originalHistory
  );
  expect(migrated.project.components[0]?.tracks[0]?.items).toEqual(rootItems);
  for (const entry of PACK.presets) {
    expect(
      migrated.project.tracks[1]?.items.find(
        (item) => item.id === created.aliases[entry.id]
      )?.animationPresetProvenance
    ).toEqual(entry.expectedProvenance);
  }
  const source = migrated.project.tracks[1]?.items.find(
    (item) => item.id === itemId
  );
  const split = await edit("timeline_split_item", 4, { itemId, splitMs: 17 });
  const rightId = split.changedIds.find((id) => id !== itemId);
  expect(rightId).toBeDefined();
  await edit("timeline_trim_item", 5, {
    durationMs: 978,
    itemId: rightId,
    startMs: 22,
  });
  await edit("timeline_duplicate_items", 6, {
    itemIds: [rightId],
    offsetMs: 2000,
  });
  const edited = await read();
  const copies =
    edited.project.tracks[1]?.items.filter(
      (candidate) => candidate.id === rightId || candidate.startMs === 2022
    ) ?? [];
  expect(copies).toHaveLength(2);
  for (const item of copies) {
    expect(item.animationPresetProvenance).toEqual(
      source?.animationPresetProvenance
    );
    expect(item.animationChannels).toEqual(
      source?.animationChannels?.map((channel) => ({
        ...channel,
        clock: { offsetMs: 22, sourceDurationMs: 1000 },
      }))
    );
  }
  const bytes = [readFileSync(projectPath), readFileSync(historyPath)];
  const rejected = await client.callTool({
    arguments: {
      expectedRevision: 7,
      operations: [
        {
          color: "#ffffff",
          durationMs: 1000,
          height: 32,
          operation: "add_rectangle",
          resultAlias: "discarded",
          startMs: 3000,
          trackId,
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
          width: 32,
        },
        {
          itemId: "@discarded",
          operation: "apply_animation_preset",
          parameters: PACK.presets[0]?.parameters,
          presetId: "impact_slam",
          presetVersion: 1,
        },
        { itemId: "@discarded", operation: "split_item", splitMs: 3000 },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(rejected.structuredContent).toMatchObject({
    error: { code: "VALIDATION_FAILED", retryable: false },
  });
  expect([readFileSync(projectPath), readFileSync(historyPath)]).toEqual(bytes);
  await edit("project_undo", 7, {});
  await edit("project_redo", 8, {});
  expect((await read()).project.tracks).toEqual(edited.project.tracks);
  expect((await read()).project.components).toEqual(
    migrated.project.components
  );
  const retained = JSON.parse(
    readFileSync(historyPath, "utf8")
  ) as SavedHistory;
  expect(
    [...retained.undo, ...retained.redo].every(
      (snapshot) => snapshot.schemaVersion === 34
    )
  ).toBe(true);
};
