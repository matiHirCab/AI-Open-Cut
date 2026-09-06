import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import { projectStateSchema, writeResultSchema } from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;

export const verifyRuleCardWorkflow = async (
  client: Client,
  call: Call,
  media: string,
  projects: string
) => {
  const { projectId } = await call(
    "project_create",
    { fps: 10, height: 90, name: "Three rule cards", width: 160 },
    writeResultSchema
  );
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const initial = await read();
  const replacements: Record<string, string> = {
    $audio:
      initial.project.tracks.find((t) => t.trackType === "audio")?.id ?? "",
    $overlay:
      initial.project.tracks.find((t) => t.trackType === "overlay")?.id ?? "",
  };
  for (const [i, color] of [
    [240, 80, 80],
    [80, 240, 80],
    [80, 80, 240],
  ].entries()) {
    const bmp = Buffer.alloc(246);
    bmp.write("BM");
    bmp.writeUInt32LE(246, 2);
    bmp.writeUInt32LE(54, 10);
    bmp.writeUInt32LE(40, 14);
    bmp.writeUInt32LE(8, 18);
    bmp.writeUInt32LE(8, 22);
    bmp.writeUInt16LE(1, 26);
    bmp.writeUInt16LE(24, 28);
    for (let pixel = 54; pixel < 246; pixel += 3) {
      bmp[pixel] = color[2] ?? 0;
      bmp[pixel + 1] = color[1] ?? 0;
      bmp[pixel + 2] = color[0] ?? 0;
    }
    const path = join(media, `rule-icon-${i}.bmp`);
    writeFileSync(path, bmp);
    // biome-ignore lint/performance/noAwaitInLoops: Revisions require ordered imports.
    const result = await call(
      "asset_import",
      { expectedRevision: i, mediaType: "image", path, projectId },
      writeResultSchema
    );
    replacements[`$icon${i}`] = result.changedIds[0] ?? "";
  }
  const tone = Buffer.alloc(96_044);
  tone.write("RIFF");
  tone.writeUInt32LE(96_036, 4);
  tone.write("WAVEfmt ", 8);
  tone.writeUInt32LE(16, 16);
  tone.writeUInt16LE(1, 20);
  tone.writeUInt16LE(1, 22);
  tone.writeUInt32LE(48_000, 24);
  tone.writeUInt32LE(96_000, 28);
  tone.writeUInt16LE(2, 32);
  tone.writeUInt16LE(16, 34);
  tone.write("data", 36);
  tone.writeUInt32LE(96_000, 40);
  for (let n = 0; n < 48_000; n += 1) {
    tone.writeInt16LE(
      Math.trunc(Math.sin((2 * Math.PI * 440 * n) / 48_000) * 4096),
      44 + 2 * n
    );
  }
  const path = join(media, "rule-tone.wav");
  writeFileSync(path, tone);
  const imported = await call(
    "asset_import",
    { expectedRevision: 3, mediaType: "audio", path, projectId },
    writeResultSchema
  );
  replacements.$tone = imported.changedIds[0] ?? "";
  const operations: Record<string, unknown>[] = JSON.parse(
    readFileSync(
      new URL(
        "../../../crates/editor-core/tests/fixtures/rule-card/recipe.json",
        import.meta.url
      ),
      "utf8"
    ),
    (_key, value: unknown) =>
      typeof value === "string" && Object.hasOwn(replacements, value)
        ? replacements[value]
        : value
  );
  const created = await call(
    "timeline_batch_edit",
    { expectedRevision: 4, operations, projectId },
    writeResultSchema
  );
  const original = await read();
  expect(original.project.components).toHaveLength(1);
  expect(original.project.components[0]?.tracks[0]?.items).toHaveLength(6);
  const instances = original.project.tracks
    .flatMap((t) => t.items)
    .filter((i) => i.type === "component_instance");
  expect(instances).toHaveLength(3);
  expect(instances.map((i) => i.slotValues.number)).toEqual(
    [1, 2, 3].map((n) => ({ type: "text", value: String(n) }))
  );
  const { parent } = created.aliases;
  const instance = created.aliases.instance0;
  expect(parent).toBeDefined();
  expect(instance).toBeDefined();
  const snapshot = () =>
    ["project.json", "history.json"].map((name) =>
      readFileSync(join(projects, projectId, name))
    );
  const before = snapshot();
  const failure = await client.callTool({
    arguments: {
      expectedRevision: 5,
      operations: [
        { itemId: instance, operation: "item_set_z_index", zIndex: 4 },
        { itemId: "missing", operation: "delete_item" },
      ],
      projectId,
    },
    name: "timeline_batch_edit",
  });
  expect(failure.isError).toBe(true);
  expect(snapshot()).toEqual(before);
  const conflict = await client.callTool({
    arguments: { expectedRevision: 4, itemId: instance, projectId, zIndex: 4 },
    name: "item_set_z_index",
  });
  expect(conflict.structuredContent).toMatchObject({
    error: { code: "REVISION_CONFLICT", retryable: true },
  });
  expect(snapshot()).toEqual(before);
  await call(
    "item_set_z_index",
    { expectedRevision: 5, itemId: instance, projectId, zIndex: 4 },
    writeResultSchema
  );
  await call(
    "item_set_parent",
    { expectedRevision: 6, itemId: instance, parent: null, projectId },
    writeResultSchema
  );
  await call(
    "item_set_parent",
    {
      expectedRevision: 7,
      itemId: instance,
      parent: { id: parent, scope: "root" },
      projectId,
    },
    writeResultSchema
  );
  const transform2d = {
    anchor: { x: 0, y: 0 },
    opacity: 1,
    position: { unit: "pixels", x: 6, y: 4 },
    rotationDeg: 0,
    scaleX: 1,
    scaleY: 1,
    skewXDeg: 0,
    skewYDeg: 0,
  };
  await call(
    "timeline_batch_edit",
    {
      expectedRevision: 8,
      operations: [{ itemId: parent, operation: "update_item", transform2d }],
      projectId,
    },
    writeResultSchema
  );
  const moved = await read();
  expect(moved.project.components).toEqual(original.project.components);
  await call(
    "project_undo",
    { expectedRevision: 9, projectId },
    writeResultSchema
  );
  expect(
    (await read()).project.tracks
      .flatMap((t) => t.items)
      .find((i) => i.id === parent)?.transform2d
  ).not.toEqual(transform2d);
  await call(
    "project_redo",
    { expectedRevision: 10, projectId },
    writeResultSchema
  );
  expect((await read()).project.tracks).toEqual(moved.project.tracks);
};
