import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import PRESETS from "../../../contracts/animation-presets-v1.json";
import PACK from "../../../contracts/initial-motion-preset-pack-v1.json";
import { projectStateSchema, writeResultSchema } from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;

export const verifyPresetWorkflow = async (client: Client, call: Call) => {
  const { projectId } = await call(
    "project_create",
    { name: "Preset smoke" },
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
    operations: [
      {
        color: "#ff0000",
        durationMs: 1000,
        height: 32,
        operation: "add_rectangle",
        resultAlias: "seed",
        startMs: 0,
        trackId,
        transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
        width: 32,
      },
      { ...PRESETS.examples.apply, itemId: "@seed" },
    ],
  });
  const itemId = created.aliases.seed;
  expect(created.revision).toBe(1);
  const first = await read();
  expect(first.project.tracks[1]?.items[0]?.animationChannels).toEqual([
    PRESETS.examples.resolvedChannel,
  ]);
  expect(first.project.tracks[1]?.items[0]?.animationPresetProvenance).toEqual({
    "transform.opacity": PRESETS.examples.provenance,
  });
  const { operation: _operation, ...fields } = PRESETS.examples.apply;
  await Promise.all(
    (
      [
        [0, {}, "REVISION_CONFLICT"],
        [1, {}, "INVALID_ARGUMENT"],
        [1, { presetVersion: 2 }, "INVALID_ARGUMENT"],
        [1, { itemId: "missing" }, "ITEM_NOT_FOUND"],
      ] as const
    ).map(async ([expectedRevision, overrides, code]) => {
      const failed = await client.callTool({
        arguments: {
          expectedRevision,
          projectId,
          ...fields,
          itemId,
          ...overrides,
        },
        name: "timeline_apply_animation_preset",
      });
      expect(failed.isError).toBe(true);
      expect(failed.structuredContent).toMatchObject({
        error: { code, retryable: code === "REVISION_CONFLICT" },
      });
    })
  );
  const rollback = await client.callTool({
    arguments: {
      expectedRevision: 1,
      operations: [
        { color: "#00ff00", itemId, operation: "update_item" },
        {
          ...PRESETS.examples.apply,
          collisionPolicy: "replace",
          itemId,
          presetVersion: 2,
        },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(rollback.isError).toBe(true);
  expect(rollback.structuredContent).toMatchObject({
    error: { code: "INVALID_ARGUMENT" },
  });
  expect(await read()).toEqual(first);
  await edit("timeline_apply_animation_preset", 1, {
    ...fields,
    collisionPolicy: "replace",
    itemId,
    parameters: { ...fields.parameters, from: 1, to: 0 },
  });
  const replaced = await read();
  expect(
    replaced.project.tracks[1]?.items[0]?.animationPresetProvenance?.[
      "transform.opacity"
    ]?.parameters
  ).toMatchObject({ from: 1 });
  await edit("timeline_set_animation_channels", 2, {
    animationChannels: replaced.project.tracks[1]?.items[0]?.animationChannels,
    itemId,
  });
  expect(
    (await read()).project.tracks[1]?.items[0]?.animationPresetProvenance
  ).toBeUndefined();
  await edit("project_undo", 3, {});
  expect((await read()).project.tracks).toEqual(replaced.project.tracks);
  await edit("project_redo", 4, {});
  await edit("track_update", 5, { locked: true, trackId });
  const locked = await client.callTool({
    arguments: {
      expectedRevision: 6,
      projectId,
      ...fields,
      collisionPolicy: "replace",
      itemId,
    },
    name: "timeline_apply_animation_preset",
  });
  expect(locked.structuredContent).toMatchObject({
    error: { code: "TRACK_LOCKED", retryable: false },
  });
  await edit("track_update", 6, { locked: false, trackId });

  expect(
    (await read()).project.tracks[1]?.items[0]?.animationPresetProvenance
  ).toBeUndefined();
  const verifyPack = async (entry: (typeof PACK.presets)[number]) => {
    const before = await read();
    const { revision } = before.project;
    const application = {
      itemId: "@motion",
      operation: "apply_animation_preset",
      parameters: entry.parameters,
      presetId: entry.id,
      presetVersion: 1,
    };
    const createdPack = await edit("timeline_batch_edit", revision, {
      operations: [
        {
          color: "#00ff00",
          durationMs: 1000,
          height: 32,
          operation: "add_rectangle",
          resultAlias: "motion",
          startMs: 0,
          trackId,
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
          width: 32,
        },
        application,
      ],
    });
    const saved = await read();
    const motion = saved.project.tracks[1]?.items.find(
      (item) => item.id === createdPack.aliases.motion
    );
    expect(motion?.animationPresetProvenance).toEqual(entry.expectedProvenance);
    const expected = entry.expected.channels.map((channel) => ({
      keyframes: channel.times.map((timeMs, index) => ({
        curve: index + 1 === channel.times.length ? "hold" : "linear",
        timeMs,
        value: { type: "scalar", value: channel.values[index] },
      })),
      property: channel.property,
      ...(entry.expected.loop ? { loop: entry.expected.loop } : {}),
    }));
    expect(motion?.animationChannels).toEqual(expected);
    expect(motion?.motionBlur).toEqual(entry.expected.motionBlur ?? undefined);
    const { operation: _packOperation, ...packFields } = application;
    const rejected = await client.callTool({
      arguments: {
        expectedRevision: revision + 1,
        projectId,
        ...packFields,
        itemId: createdPack.aliases.motion,
      },
      name: "timeline_apply_animation_preset",
    });
    expect(rejected.isError).toBe(true);
    expect(rejected.structuredContent).toMatchObject({
      error: { code: "INVALID_ARGUMENT", retryable: false },
    });
    expect(await read()).toEqual(saved);
    await edit("project_undo", revision + 1, {});
    expect((await read()).project.tracks).toEqual(before.project.tracks);
    await edit("project_redo", revision + 2, {});
    expect((await read()).project.tracks).toEqual(saved.project.tracks);
  };
  await PACK.presets.reduce(
    (prior, entry) => prior.then(() => verifyPack(entry)),
    Promise.resolve()
  );
};
