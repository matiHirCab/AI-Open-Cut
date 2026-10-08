import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { expect, it } from "vitest";
import { z } from "zod";
import busCatalog from "../../../contracts/audio-buses-v1.json";
import catalog from "../../../contracts/desktop-compositing-controls-v1.json";
import soundCatalog from "../../../contracts/semantic-sound-events-v1.json";
import {
  projectStateSchema,
  publicErrorSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

it("fresh MCP compositing standalone and alias batches preserve full failures order history and reconnect", async () => {
  const root = mkdtempSync(join(tmpdir(), "opencut-desktop-controls-mcp-"));
  const projects = join(root, "projects"),
    media = join(root, "media"),
    exportsDirectory = join(root, "exports");
  for (const p of [projects, media, exportsDirectory]) {
    mkdirSync(p);
  }
  const env = {
    ...Object.fromEntries(
      Object.entries(process.env).filter(
        (e): e is [string, string] => e[1] !== undefined
      )
    ),
    OPENCUT_ALLOWED_MEDIA_DIRS: media,
    OPENCUT_EXPORTS_DIR: exportsDirectory,
    OPENCUT_HEADLESS_PATH:
      process.env.OPENCUT_TEST_HEADLESS_PATH ??
      resolve(
        import.meta.dirname,
        `../../../target/debug/opencut-headless${process.platform === "win32" ? ".exe" : ""}`
      ),
    OPENCUT_PROJECTS_DIR: projects,
  };
  const connect = async () => {
    const client = new Client({
      name: "desktop-controls-conformance",
      version: "1",
    });
    await client.connect(
      new StdioClientTransport({
        args: ["run", resolve(import.meta.dirname, "../src/index.ts")],
        command: "bun",
        env,
        stderr: "inherit",
      })
    );
    return client;
  };
  let client = await connect();
  const call = async (name: string, args: Record<string, unknown>) => {
    const reply = await client.callTool({ arguments: args, name });
    expect(reply.isError, JSON.stringify(reply.structuredContent)).not.toBe(
      true
    );
    return reply.structuredContent;
  };
  const files = (projectId: string) => {
    const result: Record<string, string> = {};
    const directory = join(projects, projectId);
    const visit = (p: string) => {
      for (const entry of readdirSync(p, { withFileTypes: true })) {
        const file = join(p, entry.name);
        if (entry.isDirectory()) {
          visit(file);
        } else {
          result[file.slice(directory.length)] =
            readFileSync(file).toString("base64");
        }
      }
    };
    visit(directory);
    return result;
  };
  try {
    const status = statusSchema.parse(await call("editor_get_status", {}));
    expect(catalog.projectSchemaVersion).toBe(38);
    expect(status.projectSchemaVersion).toBe(soundCatalog.projectSchemaVersion);
    expect(status.protocolVersion).toBe(catalog.headlessProtocolVersion);
    const tools = await client.listTools();
    const addedTools = [
      "speech_markers_generate",
      "audio_bus_set_route",
      "audio_track_route",
      soundCatalog.operation,
    ];
    expect(
      tools.tools.filter((tool) => !addedTools.includes(tool.name))
    ).toHaveLength(catalog.registeredToolCount);
    expect(
      tools.tools.filter((tool) => tool.name === "speech_markers_generate")
    ).toHaveLength(1);
    for (const name of busCatalog.operations) {
      expect(tools.tools.filter((tool) => tool.name === name)).toHaveLength(1);
    }
    expect(tools.tools.some((t) => t.name === catalog.mcpMutation)).toBe(true);
    const { projectId } = writeResultSchema.parse(
      await call("project_create", {
        fps: 10,
        height: 64,
        name: "Desktop control API conformance",
        width: 64,
      })
    );
    const read = async () =>
      projectStateSchema.parse(await call("project_open", { projectId }));
    const initial = await read();
    const rectangle = {
      color: "#ffffff",
      durationMs: 1000,
      height: 64,
      operation: "add_rectangle",
      startMs: 0,
      trackId: initial.project.tracks[1]?.id,
      transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
      width: 64,
    };
    const tint = {
      ...catalog.effectDefaults.color_tint,
      color: {
        a: 0.876_543_210_987_654_3,
        b: 0.987_654_321_098_765_4,
        g: 0.333_333_333_333_333_3,
        r: 0.123_456_789_012_345_66,
      },
      id: "tint",
    };
    const created = writeResultSchema.parse(
      await call("timeline_batch_edit", {
        expectedRevision: 0,
        operations: [
          { ...rectangle, resultAlias: "provider" },
          { itemId: "@provider", matteOnly: true, operation: "update_item" },
          { ...rectangle, resultAlias: "leaf" },
          {
            blendMode: "screen",
            effects: [catalog.effectDefaults.glow, tint],
            itemId: "@leaf",
            masks: [catalog.defaultMask],
            matte: { channel: "luma", sourceId: "@provider" },
            operation: "update_item",
          },
          {
            durationMs: 1000,
            operation: "add_group",
            resultAlias: "owner",
            startMs: 0,
            trackId: rectangle.trackId,
          },
          {
            clip: catalog.clipValue,
            effects: [catalog.effectDefaults.screen_flash],
            itemId: "@owner",
            operation: "update_item",
          },
        ],
        projectId,
      })
    );
    const { leaf, owner } = created.aliases;
    expect(leaf).toBeDefined();
    expect(owner).toBeDefined();
    const baseline = await read();
    const getLeaf = (state: typeof baseline) =>
      state.project.tracks.flatMap((t) => t.items).find((i) => i.id === leaf);
    expect(getLeaf(baseline)).toMatchObject({
      blendMode: "screen",
      effects: [catalog.effectDefaults.glow, tint],
      masks: [catalog.defaultMask],
      matte: { channel: "luma", sourceId: created.aliases.provider },
    });
    const { revision } = baseline.project;
    const before = files(projectId);
    await [
      [
        {
          itemId: leaf,
          masks: [catalog.defaultMask, catalog.defaultMask],
          operation: "update_item",
        },
      ],
      [
        {
          itemId: leaf,
          matte: { channel: "alpha", sourceId: "missing" },
          operation: "update_item",
        },
      ],
      [{ effects: [], itemId: "missing", operation: "update_item" }],
    ].reduce(async (previous, edits) => {
      await previous;
      const response = await client.callTool({
        arguments: {
          expectedRevision: revision,
          operations: [
            { clip: null, itemId: owner, operation: "update_item" },
            ...edits,
          ],
          projectId,
        },
        name: "timeline_batch_edit",
      });
      expect(response.isError).toBe(true);
      expect(await read()).toEqual(baseline);
      expect(files(projectId)).toEqual(before);
      const standalone = await client.callTool({
        arguments: {
          expectedRevision: revision,
          projectId,
          ...Object.fromEntries(
            Object.entries(edits[0] ?? {}).filter(
              ([key]) => key !== "operation"
            )
          ),
        },
        name: catalog.mcpMutation,
      });
      expect(standalone.isError).toBe(true);
      const errorEnvelope = z.object({ error: publicErrorSchema }).strict();
      const batchError = errorEnvelope.parse(response.structuredContent).error;
      const standaloneError = errorEnvelope.parse(
        standalone.structuredContent
      ).error;
      expect(batchError.code).toBe(
        "masks" in (edits[0] ?? {}) ? "INVALID_ARGUMENT" : "ITEM_NOT_FOUND"
      );
      expect(batchError.retryable).toBe(false);
      expect(standaloneError.code).toBe(batchError.code);
      expect(standaloneError.retryable).toBe(batchError.retryable);
      expect(await read()).toEqual(baseline);
      expect(files(projectId)).toEqual(before);
    }, Promise.resolve());
    const conflict = await client.callTool({
      arguments: {
        blendMode: "multiply",
        expectedRevision: revision - 1,
        itemId: leaf,
        projectId,
      },
      name: catalog.mcpMutation,
    });
    expect(conflict.isError).toBe(true);
    expect(JSON.stringify(conflict.structuredContent)).toContain(
      "REVISION_CONFLICT"
    );
    expect(files(projectId)).toEqual(before);
    await call(catalog.mcpMutation, {
      blendMode: "overlay",
      expectedRevision: revision,
      itemId: leaf,
      projectId,
    });
    const updated = await read();
    expect(getLeaf(updated)).toEqual({
      ...getLeaf(baseline),
      blendMode: "overlay",
    });
    await call("timeline_batch_edit", {
      expectedRevision: updated.project.revision,
      operations: [
        {
          effects: [],
          itemId: leaf,
          masks: [],
          matte: null,
          operation: "update_item",
        },
        { clip: null, itemId: owner, operation: "update_item" },
      ],
      projectId,
    });
    const cleared = await read();
    expect(getLeaf(cleared)).toMatchObject({ blendMode: "overlay" });
    expect(getLeaf(cleared)?.masks ?? []).toEqual([]);
    expect(getLeaf(cleared)?.effects ?? []).toEqual([]);
    expect(getLeaf(cleared)?.matte).toBeUndefined();
    await call("project_undo", {
      expectedRevision: cleared.project.revision,
      projectId,
    });
    const undo = await read();
    expect(undo.project.updatedAtMs).toBeGreaterThanOrEqual(
      updated.project.updatedAtMs
    );
    expect({
      ...undo.project,
      revision: updated.project.revision,
      updatedAtMs: updated.project.updatedAtMs,
    }).toEqual(updated.project);
    await call("project_redo", {
      expectedRevision: undo.project.revision,
      projectId,
    });
    const redo = await read();
    expect(redo.project.updatedAtMs).toBeGreaterThanOrEqual(
      cleared.project.updatedAtMs
    );
    expect({
      ...redo.project,
      revision: cleared.project.revision,
      updatedAtMs: cleared.project.updatedAtMs,
    }).toEqual(cleared.project);
    await client.close();
    client = await connect();
    expect(await read()).toEqual(redo);
    await call("track_update", {
      expectedRevision: redo.project.revision,
      locked: true,
      projectId,
      trackId: rectangle.trackId,
    });
    const locked = await read();
    const lockedFiles = files(projectId);
    const rejected = await client.callTool({
      arguments: {
        effects: [],
        expectedRevision: locked.project.revision,
        itemId: leaf,
        projectId,
      },
      name: catalog.mcpMutation,
    });
    expect(rejected.isError).toBe(true);
    expect(JSON.stringify(rejected.structuredContent)).toContain(
      "TRACK_LOCKED"
    );
    expect(await read()).toEqual(locked);
    expect(files(projectId)).toEqual(lockedFiles);

    expect(
      statusSchema.parse(await call("editor_get_status", {}))
        .projectSchemaVersion
    ).toBe(soundCatalog.projectSchemaVersion);
  } finally {
    await client.close();
    rmSync(root, { force: true, recursive: true });
  }
}, 120_000);
