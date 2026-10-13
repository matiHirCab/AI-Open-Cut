import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import RECIPE from "../../../contracts/native-motif-pack-v1.json";
import { projectStateSchema, writeResultSchema } from "../src/schemas";

type Call = <T>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<T>
) => Promise<T>;
export const resolveMotifValues = (
  value: unknown,
  values: Record<string, string>
): unknown =>
  JSON.parse(JSON.stringify(value), (_key, x) =>
    typeof x === "string" && Object.hasOwn(values, x) ? values[x] : x
  );

const verifyMotifCustomizationHistory = async (
  call: Call,
  projectId: string,
  componentId: string
) => {
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const saved = await read();
  const [target] = saved.project.tracks
    .flatMap((t) => t.items)
    .filter(
      (i) => i.type === "component_instance" && i.componentId === componentId
    );
  if (target?.type !== "component_instance") {
    throw new Error("Missing editable motif occurrence");
  }
  const update = {
    componentId,
    durationMs: target.durationMs,
    itemId: target.id,
    projectId,
    startMs: target.startMs,
    timeScale: target.timeScale,
    trimStartMs: target.trimStartMs,
  };
  await call(
    "component_instance_update",
    {
      ...update,
      expectedRevision: saved.project.revision,
      slotValues: { dot_color: { type: "color", value: "#00fFaa" } },
    },
    writeResultSchema
  );
  const changed = await read();
  expect(changed.project.components).toEqual(saved.project.components);
  for (const track of saved.project.tracks) {
    const others = track.items.filter((i) => i.id !== target.id);
    expect(
      changed.project.tracks
        .find((t) => t.id === track.id)
        ?.items.filter((i) => i.id !== target.id)
    ).toEqual(others);
  }
  await call(
    "component_instance_update",
    { ...update, expectedRevision: changed.project.revision, slotValues: {} },
    writeResultSchema
  );
  const reset = await read();
  expect(reset.project.tracks).toEqual(saved.project.tracks);
  await call(
    "project_undo",
    { expectedRevision: reset.project.revision, projectId },
    writeResultSchema
  );
  const restoredOverride = await read();
  expect(restoredOverride.project.tracks).toEqual(changed.project.tracks);
  await call(
    "project_undo",
    { expectedRevision: restoredOverride.project.revision, projectId },
    writeResultSchema
  );
  const restoredOriginal = await read();
  expect(restoredOriginal.project.tracks).toEqual(saved.project.tracks);
  await call(
    "project_redo",
    { expectedRevision: restoredOriginal.project.revision, projectId },
    writeResultSchema
  );
  const redoneOverride = await read();
  expect(redoneOverride.project.tracks).toEqual(changed.project.tracks);
  await call(
    "project_redo",
    { expectedRevision: redoneOverride.project.revision, projectId },
    writeResultSchema
  );
  const reopened = await read();
  expect(reopened.project.tracks).toEqual(saved.project.tracks);
  expect(reopened.project.components).toEqual(saved.project.components);
};

export const verifyNativeMotifWorkflow = async (
  client: Client,
  call: Call,
  heroProjectId: string
) => {
  const gallery = await call(
    "project_create",
    { name: "Editable native motif gallery", ...RECIPE.canvas },
    writeResultSchema
  );
  for (const [projectId, entries] of [
    [gallery.projectId, RECIPE.galleryInstances],
    [heroProjectId, RECIPE.aroundHeroInstances],
  ] as const) {
    const read = () => call("project_open", { projectId }, projectStateSchema);
    // biome-ignore lint/performance/noAwaitInLoops: Each scene uses revision-ordered writes and immutable-state comparisons.
    const initial = await read();
    const overlay = initial.project.tracks.find(
      (track) => track.trackType === "overlay"
    )?.id;
    if (!overlay) {
      throw new Error("Missing motif overlay");
    }
    const prompt = await client.getPrompt({
      arguments: { projectId, request: "Native editable motifs" },
      name: "create_motion_graphics",
    });
    const content = prompt.messages[0]?.content;
    if (content?.type !== "text") {
      throw new Error("Missing motif prompt");
    }
    const recipe: typeof RECIPE = JSON.parse(
      content.text
        .split("native-motif-pack-v1 recipe JSON:\n")[1]
        ?.split("\nEND native-motif-pack-v1 recipe JSON")[0] ?? "null"
    );
    expect(recipe).toEqual(RECIPE);
    expect(await read()).toEqual(initial);
    const phaseA = await call(
      "timeline_batch_edit",
      {
        expectedRevision: initial.project.revision,
        operations: recipe.motifs.map((m) => m.seed),
        projectId,
      },
      writeResultSchema
    );
    const values: Record<string, string> = { $overlay: overlay };
    for (const motif of recipe.motifs) {
      const id = phaseA.aliases[motif.phaseAAlias];
      if (!id) {
        throw new Error("Missing actual motif definition ID");
      }
      values[motif.componentIdPlaceholder] = id;
    }
    const operations = resolveMotifValues(
      [...recipe.motifs.map((m) => m.populate), ...entries],
      values
    );
    const before = await read();
    const failed = await client.callTool({
      arguments: {
        expectedRevision: before.project.revision,
        operations: [
          ...(operations as unknown[]),
          { itemId: "missing", operation: "delete_item" },
        ],
        projectId,
      },
      name: "timeline_batch_edit",
    });
    expect(failed.isError).toBe(true);
    expect(failed.structuredContent).toMatchObject({
      error: { code: "ITEM_NOT_FOUND", retryable: false },
    });
    expect(await read()).toEqual(before);
    await call(
      "timeline_batch_edit",
      { expectedRevision: before.project.revision, operations, projectId },
      writeResultSchema
    );
    const saved = await read();
    expect(saved.project.assets).toEqual(initial.project.assets);
    expect(
      saved.project.components.slice(0, initial.project.components.length)
    ).toEqual(initial.project.components);
    for (const track of initial.project.tracks) {
      const current = saved.project.tracks.find((t) => t.id === track.id);
      expect(current?.items.slice(0, track.items.length)).toEqual(track.items);
    }
    await call(
      "project_undo",
      { expectedRevision: saved.project.revision, projectId },
      writeResultSchema
    );
    const undone = await read();
    expect(undone.project.components).toEqual(before.project.components);
    expect(undone.project.tracks).toEqual(before.project.tracks);
    await call(
      "project_redo",
      { expectedRevision: undone.project.revision, projectId },
      writeResultSchema
    );
    const reopened = await read();
    expect(reopened.project.components).toEqual(saved.project.components);
    expect(reopened.project.tracks).toEqual(saved.project.tracks);
    const dotId = values.$dotComponentId;
    if (!dotId) {
      throw new Error("Missing actual dot definition ID");
    }
    await verifyMotifCustomizationHistory(call, projectId, dotId);
  }
  return { galleryProjectId: gallery.projectId, heroProjectId };
};
