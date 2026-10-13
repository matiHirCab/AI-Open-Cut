import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import RECIPE from "../../../contracts/native-role-team-v1.json";
import { projectStateSchema, writeResultSchema } from "../src/schemas";
import {
  captureTeamArtifact,
  type TeamCall,
  type TeamNative,
  teamProjectInventory,
  verifyTeamMedia,
} from "./native-role-team-native";

const originalArt = (blue: boolean) => {
  const b = Buffer.alloc(54 + 64 * 64 * 3);
  b.write("BM");
  b.writeUInt32LE(b.length, 2);
  b.writeUInt32LE(54, 10);
  b.writeUInt32LE(40, 14);
  b.writeUInt32LE(64, 18);
  b.writeUInt32LE(64, 22);
  b.writeUInt16LE(1, 26);
  b.writeUInt16LE(24, 28);
  for (let y = 0; y < 64; y += 1) {
    for (let x = 0; x < 64; x += 1) {
      const right = blue ? [0, 0, 255] : [255, 0, 0];
      const rgb = x < 32 ? [255, 255, 255] : right;
      const p = 54 + (y * 64 + x) * 3;
      b[p] = rgb[2] ?? 0;
      b[p + 1] = rgb[1] ?? 0;
      b[p + 2] = rgb[0] ?? 0;
    }
  }
  return b;
};
const resolve = (v: unknown, values: Record<string, string>): unknown =>
  JSON.parse(JSON.stringify(v), (_key, x) =>
    typeof x === "string" && Object.hasOwn(values, x) ? values[x] : x
  );
const rejectTeamRequests = async (
  client: Client,
  call: TeamCall,
  projects: string,
  projectId: string,
  requests: { name: string; input: Record<string, unknown>; code: string }[]
) => {
  const directory = join(projects, projectId);
  const bytes = () => teamProjectInventory(directory);
  const baseline = bytes();
  const state = await call("project_open", { projectId }, projectStateSchema);
  await requests.reduce(
    (previous, request) =>
      previous.then(async () => {
        const reply = await client.callTool({
          arguments: { projectId, ...request.input },
          name: request.name,
        });
        expect(reply.isError).toBe(true);
        expect(reply.structuredContent).toMatchObject({
          error: {
            code: request.code,
            retryable: request.code === "REVISION_CONFLICT",
          },
        });
        expect(bytes()).toEqual(baseline);
        expect(
          await call("project_open", { projectId }, projectStateSchema)
        ).toEqual(state);
      }),
    Promise.resolve()
  );
};
const prepareTeam = async (
  client: Client,
  call: TeamCall,
  media: string,
  projects: string,
  count: 1 | 4
) => {
  const created = await call(
    "project_create",
    { name: `Native role team ${count}`, ...RECIPE.canvas },
    writeResultSchema
  );
  const { projectId } = created;
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const initial = await read();
  const prompt = await client.getPrompt({
    arguments: { projectId, request: "native-role-team-v1" },
    name: "create_motion_graphics",
  });
  const message = prompt.messages[0]?.content;
  if (message?.type !== "text") {
    throw new Error("Missing team prompt");
  }
  const recipe: typeof RECIPE = JSON.parse(
    message.text
      .split("native-role-team-v1 recipe JSON:\n")[1]
      ?.split("\nEND native-role-team-v1 recipe JSON")[0] ?? "null"
  );
  expect(recipe).toEqual(RECIPE);
  expect(await read()).toEqual(initial);
  const assets: string[] = [];
  for (const blue of [false, true]) {
    const path = join(media, `role-team-${count}-${blue ? "blue" : "red"}.bmp`);
    writeFileSync(path, originalArt(blue));
    // biome-ignore lint/performance/noAwaitInLoops: Ordered asset imports require the current optimistic revision.
    const { revision } = (await read()).project;
    const imported = await call(
      "asset_import",
      { expectedRevision: revision, mediaType: "image", path, projectId },
      writeResultSchema
    );
    const [asset] = imported.changedIds;
    if (!asset) {
      throw new Error("Missing team artwork ID");
    }
    assets.push(asset);
  }
  const before = await read();
  const overlay = before.project.tracks.find(
    (t) => t.trackType === "overlay"
  )?.id;
  if (!(overlay && assets[0] && assets[1])) {
    throw new Error("Missing team inputs");
  }
  const operations = resolve(
    [
      recipe.badgeCreate,
      recipe.badgeDefineSlots,
      ...recipe.layouts.grid[String(count) as "1" | "4"],
    ],
    { $artwork: assets[0], $overlay: overlay }
  );
  const batchOperations = operations as Record<string, unknown>[];
  const badAlias = structuredClone(batchOperations);
  if (badAlias[3]) {
    badAlias[3].componentId = "@future_badge";
  }
  await rejectTeamRequests(client, call, projects, projectId, [
    {
      code: "ITEM_NOT_FOUND",
      input: {
        expectedRevision: before.project.revision,
        operations: [
          ...batchOperations,
          { itemId: "absent", operation: "delete_item" },
        ],
      },
      name: "timeline_batch_edit",
    },
    {
      code: "VALIDATION_FAILED",
      input: {
        expectedRevision: before.project.revision,
        operations: badAlias,
      },
      name: "timeline_batch_edit",
    },
  ]);
  const result = await call(
    "timeline_batch_edit",
    { expectedRevision: before.project.revision, operations, projectId },
    writeResultSchema
  );
  expect(result.revision).toBe(before.project.revision + 1);
  const saved = await read();
  const componentId = result.aliases.badge,
    itemId = result.aliases.member_1;
  if (!(componentId && itemId)) {
    throw new Error("Missing stable team alias mapping");
  }
  expect(
    saved.project.components[0]?.tracks[0]?.items.map((i) => i.id)
  ).toEqual(recipe.roles);
  return { assets, componentId, itemId, projectId, read, saved };
};

const verifyTeamCount = async (
  client: Client,
  call: TeamCall,
  media: string,
  projects: string,
  count: 1 | 4,
  native?: TeamNative
) => {
  const { projectId, read, saved, componentId, itemId, assets } =
    await prepareTeam(client, call, media, projects, count);
  const frames: string[] = [];
  const directory = join(projects, projectId);
  const authoritative = () => [
    readFileSync(join(directory, "project.json")),
    readFileSync(join(directory, "history.json")),
  ];
  const bytes = authoritative();
  if (native) {
    frames.push(
      await captureTeamArtifact(
        call,
        projects,
        projectId,
        saved.project.revision,
        "preview_render_frame",
        { timeMs: 500 }
      )
    );
    expect(authoritative()).toEqual(bytes);
  }
  const update = {
    componentId,
    durationMs: 1000,
    itemId,
    projectId,
    startMs: 0,
    timeScale: 1,
    trimStartMs: 0,
  };
  await rejectTeamRequests(client, call, projects, projectId, [
    {
      code: "REVISION_CONFLICT",
      input: {
        ...update,
        expectedRevision: saved.project.revision - 1,
        slotValues: {},
      },
      name: "component_instance_update",
    },
    {
      code: "INVALID_ARGUMENT",
      input: {
        ...update,
        expectedRevision: saved.project.revision,
        slotValues: { role_label: { type: "text", value: "X".repeat(25) } },
      },
      name: "component_instance_update",
    },
    {
      code: "ASSET_NOT_FOUND",
      input: {
        ...update,
        expectedRevision: saved.project.revision,
        slotValues: {
          artwork: {
            type: "asset",
            value: { id: "absent", kind: "asset", scope: "project" },
          },
        },
      },
      name: "component_instance_update",
    },
  ]);
  await call(
    "component_instance_update",
    {
      ...update,
      expectedRevision: saved.project.revision,
      slotValues: {
        artwork: {
          type: "asset",
          value: { id: assets[1], kind: "asset", scope: "project" },
        },
        background_color: { type: "color", value: "#00ff00" },
        rays_hidden: { type: "boolean", value: true },
        role_label: { type: "text", value: "TEAM LEAD" },
      },
    },
    writeResultSchema
  );
  const changed = await read();
  expect(
    changed.project.tracks
      .flatMap((t) => t.items)
      .find((item) => item.id === itemId)
  ).toMatchObject({
    slotValues: {
      artwork: {
        type: "asset",
        value: { id: assets[1], kind: "asset", scope: "project" },
      },
      background_color: { type: "color", value: "#00ff00" },
      rays_hidden: { type: "boolean", value: true },
      role_label: { type: "text", value: "TEAM LEAD" },
    },
  });

  expect(changed.project.components).toEqual(saved.project.components);
  for (const t of saved.project.tracks) {
    expect(
      changed.project.tracks
        .find((x) => x.id === t.id)
        ?.items.filter((i) => i.id !== itemId)
    ).toEqual(t.items.filter((i) => i.id !== itemId));
  }
  if (native && count === 4) {
    frames.push(
      await captureTeamArtifact(
        call,
        projects,
        projectId,
        changed.project.revision,
        "preview_render_frame",
        { timeMs: 500 }
      )
    );
  }
  await call(
    "project_undo",
    { expectedRevision: changed.project.revision, projectId },
    writeResultSchema
  );
  const undone = await read();
  expect(undone.project.tracks).toEqual(saved.project.tracks);
  if (native && count === 4) {
    frames.push(
      await captureTeamArtifact(
        call,
        projects,
        projectId,
        undone.project.revision,
        "preview_review_range",
        { endMs: 400, includeAudio: false, resolution: "project", startMs: 0 }
      )
    );
  }
  if (native && count === 4) {
    for (const timeMs of [40, 80, 120, 280]) {
      // biome-ignore lint/performance/noAwaitInLoops: Fixed native samples use sequential renderer requests.
      const artifact = await captureTeamArtifact(
        call,
        projects,
        projectId,
        undone.project.revision,
        "preview_render_frame",
        { timeMs }
      );
      frames.push(artifact);
    }
  }

  await call(
    "project_redo",
    { expectedRevision: undone.project.revision, projectId },
    writeResultSchema
  );
  const reopened = await read();
  expect(reopened.project.tracks).toEqual(changed.project.tracks);
  expect(reopened.project.components).toEqual(changed.project.components);
  await call(
    "component_instance_update",
    {
      ...update,
      expectedRevision: reopened.project.revision,
      slotValues: {},
    },
    writeResultSchema
  );
  const reset = await read();
  expect(
    reset.project.tracks.flatMap((t) => t.items).find((i) => i.id === itemId)
  ).toMatchObject({ slotValues: {} });
  if (native && count === 4) {
    const team = saved.project.tracks
      .flatMap((t) => t.items)
      .find((i) => i.type === "group");
    if (!team) {
      throw new Error("Missing editable team parent");
    }
    await call(
      "timeline_update_item",
      {
        expectedRevision: reset.project.revision,
        itemId: team.id,
        projectId,
        transform2d: {
          anchor: { x: 0, y: 0 },
          opacity: 1,
          position: { unit: "pixels", x: 20, y: 30 },
          rotationDeg: 0,
          scaleX: 1,
          scaleY: 1,
          skewXDeg: 0,
          skewYDeg: 0,
        },
      },
      writeResultSchema
    );
    const shifted = await read();
    frames.push(
      await captureTeamArtifact(
        call,
        projects,
        projectId,
        shifted.project.revision,
        "preview_render_frame",
        { timeMs: 500 }
      )
    );
    await call(
      "project_undo",
      { expectedRevision: shifted.project.revision, projectId },
      writeResultSchema
    );
    expect((await read()).project.tracks).toEqual(reset.project.tracks);
  }
  return frames;
};

export const verifyNativeRoleTeamWorkflow = async (
  client: Client,
  call: TeamCall,
  media: string,
  projects: string,
  native?: TeamNative
) => {
  const frames = [
    ...(await verifyTeamCount(client, call, media, projects, 1, native)),
    ...(await verifyTeamCount(client, call, media, projects, 4, native)),
  ];
  if (native) {
    const [minimum, maximum, changed, range, ...additional] = frames;
    const moved = additional.at(-1);
    if (!(minimum && maximum && changed && range && moved)) {
      throw new Error("Missing bounded team native witnesses");
    }
    verifyTeamMedia(
      native,
      minimum,
      maximum,
      changed,
      range,
      additional.slice(0, -1),
      moved
    );
  }
};
