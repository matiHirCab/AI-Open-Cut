import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import COMPONENTS from "../../../contracts/component-definitions-v1.json";
import type SLOT_CATALOG from "../../../contracts/template-slots-v1.json";
import { projectStateSchema, writeResultSchema } from "../src/schemas";

// Parse the JSON as data: bundlers may emit __proto__ as object-literal syntax.
const SLOTS: typeof SLOT_CATALOG = JSON.parse(
  readFileSync(
    new URL("../../../contracts/template-slots-v1.json", import.meta.url),
    "utf8"
  )
);
type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;

export const verifyComponentWorkflow = async (
  client: Client,
  call: Call,
  mediaDirectory: string,
  projectsDirectory: string
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
  const slotDefinitions = SLOTS.valid.map(({ slot }) => ({
    ...slot,
    binding: {
      ...slot.binding,
      targetLayerId: slot.kind === "asset" ? "media" : `title_${slot.kind}`,
    },
    defaultValue:
      slot.kind === "asset"
        ? {
            type: "asset",
            value: { id: assetId, kind: "asset", scope: "project" },
          }
        : slot.defaultValue,
  }));
  const items = SLOTS.valid.map(({ slot }, index) =>
    slot.kind === "asset"
      ? {
          assetId,
          audio: { fadeInMs: 0, fadeOutMs: 0, muted: false, volume: 1 },
          durationMs: 1,
          id: "media",
          keyframes: [],
          sourceInMs: 0,
          stackOrder: index,
          startMs: 0,
          type: "media",
        }
      : {
          color: "#ffffff",
          durationMs: 1000,
          fontSize: 24,
          id: `title_${slot.kind}`,
          keyframes: [],
          stackOrder: index,
          startMs: 0,
          text: "Base",
          type: "text",
        }
  );
  const slotFields = {
    durationMs: 1000,
    height: 240,
    name: "Typed slots",
    tracks: [{ id: "slot_track", items, name: "Slots", trackType: "overlay" }],
    width: 320,
  };
  const card = await call(
    "component_create",
    { ...slotFields, expectedRevision: 7, projectId },
    writeResultSchema
  );
  const [cardId] = card.changedIds;
  await call(
    "component_define_slots",
    {
      componentId: cardId,
      expectedRevision: 8,
      projectId,
      slots: slotDefinitions,
    },
    writeResultSchema
  );
  const values = Object.fromEntries(
    slotDefinitions.map((slot) => [slot.id, slot.defaultValue])
  );
  const bound = await call("project_open", { projectId }, projectStateSchema);
  expect(bound.project.components.at(-1)?.slots).toEqual(slotDefinitions);
  const outerTrack = nestedTrack("@slot_card");
  await call(
    "timeline_batch_edit",
    {
      expectedRevision: 9,
      operations: [
        {
          ...slotFields,
          operation: "component_create",
          resultAlias: "slot_card",
        },
        {
          componentId: "@slot_card",
          operation: "component_define_slots",
          slots: slotDefinitions,
        },
        {
          ...fields,
          operation: "component_create",
          tracks: [
            {
              ...outerTrack,
              items: outerTrack.items.map((item) => ({
                ...item,
                slotValues: values,
              })),
            },
          ],
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  const withSlots = await call(
    "project_open",
    { projectId },
    projectStateSchema
  );
  await Promise.all(
    [
      [9, cardId, slotDefinitions, "REVISION_CONFLICT"],
      [10, "missing", slotDefinitions, "ITEM_NOT_FOUND"],
      [
        10,
        cardId,
        [{ ...slotDefinitions[0], defaultValue: { type: "number", value: 1 } }],
        "INVALID_ARGUMENT",
      ],
    ].map(async ([expectedRevision, targetId, slots, code]) => {
      const failed = await client.callTool({
        arguments: {
          componentId: targetId,
          expectedRevision,
          projectId,
          slots,
        },
        name: "component_define_slots",
      });
      expect(failed.isError).toBe(true);
      expect(failed.structuredContent).toMatchObject({ error: { code } });
      expect(
        await call("project_open", { projectId }, projectStateSchema)
      ).toEqual(withSlots);
    })
  );
  const rolledBack = await client.callTool({
    arguments: {
      expectedRevision: 10,
      operations: [
        { componentId: cardId, operation: "component_define_slots", slots: [] },
        {
          componentId: "missing",
          operation: "component_define_slots",
          slots: [],
        },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(rolledBack.isError).toBe(true);
  expect(await call("project_open", { projectId }, projectStateSchema)).toEqual(
    withSlots
  );
  await call(
    "project_undo",
    { expectedRevision: 10, projectId },
    writeResultSchema
  );
  expect(
    (await call("project_open", { projectId }, projectStateSchema)).project
      .components
  ).toEqual(bound.project.components);
  await call(
    "project_redo",
    { expectedRevision: 11, projectId },
    writeResultSchema
  );
  expect(
    (await call("project_open", { projectId }, projectStateSchema)).project
      .components
  ).toEqual(withSlots.project.components);
  await call(
    "component_update",
    {
      ...slotFields,
      componentId: cardId,
      expectedRevision: 12,
      projectId,
      tracks: slotFields.tracks.map((track) => ({ ...track, locked: true })),
    },
    writeResultSchema
  );
  const lockedState = await call(
    "project_open",
    { projectId },
    projectStateSchema
  );
  const locked = await client.callTool({
    arguments: {
      componentId: cardId,
      expectedRevision: 13,
      projectId,
      slots: [],
    },
    name: "component_define_slots",
  });
  expect(locked.isError).toBe(true);
  expect(locked.structuredContent).toMatchObject({
    error: { code: "TRACK_LOCKED", retryable: false },
  });
  expect(await call("project_open", { projectId }, projectStateSchema)).toEqual(
    lockedState
  );
  await verifySlotRegressionWorkflow(client, call, projectsDirectory);
};

const verifySlotRegressionWorkflow = async (
  client: Client,
  call: Call,
  projectsDirectory: string
) => {
  const { projectId } = await call(
    "project_create",
    { name: "Slot regressions" },
    writeResultSchema
  );
  const textItems = SLOTS.regressions.specialKeys.map((key, stackOrder) => ({
    color: "#ffffff",
    durationMs: 1000,
    fontSize: 24,
    id: `title${stackOrder}`,
    keyframes: [],
    stackOrder,
    startMs: 0,
    text: `Base ${key}`,
    type: "text",
  }));
  const groupItems = SLOTS.regressions.groupItems.map((group, index) => ({
    ...group,
    id: `group${index}`,
    stackOrder: textItems.length + index,
  }));
  const textSlots = SLOTS.regressions.specialKeys.map((key, index) => ({
    binding: { property: "text.document", targetLayerId: `title${index}` },
    constraints: {},
    id: key,
    kind: "text",
    name: key,
    required: true,
    ...(index === 0
      ? {}
      : { defaultValue: { type: "text", value: "Default" } }),
  }));
  const groupSlots = groupItems.map((group, index) => ({
    binding: { property: "visual.opacity", targetLayerId: group.id },
    constraints: {},
    defaultValue: { type: "number", value: 0.5 },
    id: `opacity${index}`,
    kind: "number",
    name: "Opacity",
    required: true,
  }));
  const slots = [...textSlots, ...groupSlots];
  const fields = {
    durationMs: 1000,
    height: 240,
    name: "Bound groups",
    tracks: [
      {
        id: "local",
        items: [...textItems, ...groupItems],
        name: "Local",
        trackType: "overlay",
      },
    ],
    width: 320,
  };
  const created = await call(
    "component_create",
    { ...fields, expectedRevision: 0, projectId },
    writeResultSchema
  );
  await call(
    "component_define_slots",
    {
      componentId: created.changedIds[0],
      expectedRevision: 1,
      projectId,
      slots,
    },
    writeResultSchema
  );
  const overrides = (opacity: number) =>
    Object.fromEntries([
      ...Object.entries(SLOTS.regressions.overrides),
      ...groupSlots.map(
        (slot) => [slot.id, { type: "number", value: opacity }] as const
      ),
    ]);
  const outer = (target: string, values: Record<string, unknown>) => ({
    durationMs: 1000,
    height: 240,
    name: "Instance",
    tracks: [
      {
        id: "outer",
        items: [
          {
            componentId: target,
            durationMs: 1000,
            id: "nested",
            slotValues: values,
            startMs: 0,
            timeScale: 1,
            trimStartMs: 0,
            type: "component_instance",
          },
        ],
        name: "Outer",
        trackType: "overlay",
      },
    ],
    width: 320,
  });
  const batched = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 2,
      operations: [
        {
          ...fields,
          operation: "component_create",
          resultAlias: "leaf",
          slots,
        },
        {
          ...outer("@leaf", overrides(0)),
          operation: "component_create",
          resultAlias: "outer",
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  let revision = 3;
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const initial = await read();
  const leafId = batched.aliases.leaf;
  const outerId = batched.aliases.outer;
  if (!(leafId && outerId)) {
    throw new Error("Missing component aliases");
  }
  const baseTracks = initial.project.components.find(
    (c) => c.id === leafId
  )?.tracks;
  for (const opacity of SLOTS.regressions.opacityValues) {
    const values = overrides(opacity);
    // biome-ignore lint/performance/noAwaitInLoops: Each mutation consumes the preceding revision.
    await call(
      "component_update",
      {
        ...outer(leafId, values),
        componentId: outerId,
        expectedRevision: revision,
        projectId,
      },
      writeResultSchema
    );
    revision += 1;
    const state = await read();
    const instance = state.project.components.find((c) => c.id === outerId)
      ?.tracks[0]?.items[0];
    expect(instance).toMatchObject({ slotValues: values });
    if (instance?.type !== "component_instance") {
      throw new Error("Missing nested instance");
    }
    expect(Object.keys(instance.slotValues).sort()).toEqual(
      Object.keys(values).sort()
    );
    expect(Object.getPrototypeOf(instance.slotValues)).toBe(Object.prototype);
    expect(
      state.project.components.find((c) => c.id === leafId)?.tracks
    ).toEqual(baseTracks);
    await call(
      "project_undo",
      { expectedRevision: revision, projectId },
      writeResultSchema
    );
    await call(
      "project_redo",
      { expectedRevision: revision + 1, projectId },
      writeResultSchema
    );
    revision += 2;
    expect((await read()).project.components).toEqual(state.project.components);
  }
  const before = await read();
  await verifyClosedSlotWorkflow({
    client,
    fields,
    leafId,
    outer,
    outerId,
    projectId,
    projectsDirectory,
    read,
    revision,
  });
  for (const [values, code] of [
    [
      {
        ...overrides(0.5),
        [SLOTS.regressions.unknownSlotId]: { type: "text", value: "x" },
      },
      "ITEM_NOT_FOUND",
    ],
    [
      Object.fromEntries(
        Object.entries(overrides(0.5)).filter(([key]) => key !== "__proto__")
      ),
      "INVALID_ARGUMENT",
    ],
    ...SLOTS.regressions.invalidOpacityValues.map(
      (value) => [overrides(value), "INVALID_ARGUMENT"] as const
    ),
  ] as const) {
    // biome-ignore lint/performance/noAwaitInLoops: Assert unchanged state after each rejected mutation.
    const failure = await client.callTool({
      arguments: {
        ...outer(leafId, values),
        componentId: outerId,
        expectedRevision: revision,
        projectId,
      },
      name: "component_update",
    });
    expect(failure.isError).toBe(true);
    expect(failure.structuredContent).toMatchObject({ error: { code } });
    expect(await read()).toEqual(before);
  }
  for (const key of SLOTS.regressions.specialKeys) {
    // biome-ignore lint/performance/noAwaitInLoops: Assert unchanged state after each rejected mutation.
    const failure = await client.callTool({
      arguments: {
        ...outer(leafId, {
          ...overrides(0.5),
          [key]: SLOTS.regressions.invalidValues[0],
        }),
        componentId: outerId,
        expectedRevision: revision,
        projectId,
      },
      name: "component_update",
    });
    expect(failure.isError).toBe(true);
    expect(await read()).toEqual(before);
  }
};

const verifyClosedSlotWorkflow = async ({
  client,
  read,
  projectsDirectory,
  projectId,
  revision,
  leafId,
  outerId,
  fields,
  outer,
}: {
  client: Client;
  read: () => Promise<unknown>;
  projectsDirectory: string;
  projectId: string;
  revision: number;
  leafId: string;
  outerId: string;
  fields: Record<string, unknown>;
  outer: (
    target: string,
    values: Record<string, unknown>
  ) => Record<string, unknown>;
}) => {
  const before = await read();
  const snapshotFiles = () =>
    ["project.json", "history.json"].map((name) =>
      readFileSync(join(projectsDirectory, projectId, name))
    );
  const beforeFiles = snapshotFiles();
  for (const fixture of SLOTS.regressions.closedRecords) {
    const malformed = [
      {
        fields: { componentId: leafId, slots: [fixture.slot] },
        name: "component_define_slots",
      },
      ...(fixture.overridePath
        ? [
            {
              fields: {
                ...outer(
                  leafId,
                  Object.fromEntries([["__proto__", fixture.override]])
                ),
                componentId: outerId,
              },
              name: "component_update",
            },
          ]
        : []),
    ];
    for (const input of malformed) {
      const batchFields =
        input.name === "component_define_slots"
          ? { ...input.fields, componentId: "@candidate" }
          : input.fields;
      for (const batch of [false, true]) {
        // biome-ignore lint/performance/noAwaitInLoops: Verify rollback after each canonical malformed request.
        const failure = await client.callTool({
          arguments: {
            expectedRevision: revision,
            projectId,
            ...(batch
              ? {
                  operations: [
                    {
                      ...fields,
                      operation: "component_create",
                      resultAlias: "candidate",
                    },
                    {
                      ...batchFields,
                      operation: input.name,
                    },
                  ],
                }
              : input.fields),
          },
          name: batch ? "timeline_batch_edit" : input.name,
        });
        expect(failure.isError, fixture.id).toBe(true);
        expect(JSON.stringify(failure), fixture.id).toContain(
          "Unrecognized key"
        );
        expect(
          snapshotFiles().every((bytes, index) => {
            const expected = beforeFiles[index];
            return expected !== undefined && bytes.equals(expected);
          }),
          fixture.id
        ).toBe(true);
      }
    }
  }
  expect(await read()).toEqual(before);
};
