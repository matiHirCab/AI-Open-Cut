import type { Client } from "@modelcontextprotocol/client";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import {
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <Output>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<Output>
) => Promise<Output>;

export const verifyExtendedVisualWorkflow = async (
  client: Client,
  call: Call
) => {
  const { projectId } = await call(
    "project_create",
    { name: "Extended visual smoke" },
    writeResultSchema
  );
  const initial = await call("project_open", { projectId }, projectStateSchema);
  const trackId = initial.project.tracks[1]?.id;
  const rotation = {
    keyframes: [
      { curve: "linear", timeMs: 0, value: { type: "scalar", value: 0 } },
      { curve: "hold", timeMs: 500, value: { type: "scalar", value: 180 } },
    ],
    property: "transform.rotation_deg",
  };
  const created = await call(
    "timeline_batch_edit",
    {
      expectedRevision: 0,
      operations: [
        {
          color: "#ffffff",
          durationMs: 1000,
          height: 8,
          operation: "add_rectangle",
          resultAlias: "box",
          startMs: 0,
          trackId,
          transform: { opacity: 1, positionX: 20, positionY: 20, scale: 1 },
          width: 24,
        },
        {
          effects: [{ id: "blur", radiusPx: 0, type: "gaussian_blur" }],
          itemId: "@box",
          motionBlur: { sampleCount: 4, shutterAngleDeg: 180 },
          operation: "update_item",
        },
        {
          animationChannels: [rotation],
          itemId: "@box",
          operation: "set_animation_channels",
        },
        {
          durationMs: 1000,
          fill: { color: { a: 1, b: 1, g: 1, r: 1 }, type: "solid" },
          geometry: {
            path: {
              commands: [
                { to: { x: 0, y: 0 }, type: "moveTo" },
                { to: { x: 10, y: 10 }, type: "lineTo" },
              ],
              fillRule: "nonzero",
            },
            type: "path",
          },
          operation: "add_shape",
          resultAlias: "path",
          startMs: 0,
          stroke: null,
          trackId,
        },
        {
          animationChannels: [
            {
              keyframes: [
                {
                  curve: "hold",
                  timeMs: 0,
                  value: { type: "scalar", value: 0.5 },
                },
              ],
              property: "graphic.path_trim",
              target: { id: "@path", kind: "graphic_geometry", scope: "root" },
            },
          ],
          itemId: "@path",
          operation: "set_animation_channels",
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  const itemId = created.aliases?.box;
  expect(itemId).toBeTruthy();
  const before = await call("project_open", { projectId }, projectStateSchema);
  expect(
    before.project.tracks[1]?.items[1]?.animationChannels?.[0]?.target?.id
  ).toBe(created.aliases.path);
  const result = await client.callTool({
    arguments: {
      effects: [{ id: "blur", radiusPx: 129, type: "gaussian_blur" }],
      expectedRevision: 1,
      itemId,
      projectId,
    },
    name: "timeline_update_item",
  });
  expect(result.isError).toBe(true);
  expect(await call("project_open", { projectId }, projectStateSchema)).toEqual(
    before
  );
  await call(
    "timeline_set_animation_channels",
    { animationChannels: [], expectedRevision: 1, itemId, projectId },
    writeResultSchema
  );
  await call(
    "timeline_update_item",
    {
      expectedRevision: 2,
      itemId,
      motionBlur: { sampleCount: 1, shutterAngleDeg: 0 },
      projectId,
    },
    writeResultSchema
  );
  const state = await call("project_open", { projectId }, projectStateSchema);
  expect(state.project.revision).toBe(3);
  expect(state.project.tracks[1]?.items[0]?.effects).toEqual([
    { id: "blur", radiusPx: 0, type: "gaussian_blur" },
  ]);
  expect(state.project.tracks[1]?.items[0]?.motionBlur).toEqual({
    sampleCount: 1,
    shutterAngleDeg: 0,
  });
  expect(
    (await call("editor_get_status", {}, statusSchema)).capabilities
  ).toContain("motion_blur_sampling_v1");
  expect(
    (await call("editor_get_status", {}, statusSchema)).capabilities
  ).toContain("extended_visual_animation_v1");
};
