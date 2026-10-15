import TEAM from "../../../../contracts/native-role-team-v1.json";
import ORIGINAL from "../../../../crates/editor-core/tests/fixtures/illustrated-scene/recipe.json";
import { componentFieldsSchema } from "../../src/schemas";

export const TARGET_TIMING = {
  canvas: { fps: 30, height: 1080, width: 1920 },
  durationMs: 3000,
  range: [0, 3000],
  samples: [0, 1500, 2966],
} as const;
export type TargetId = (typeof ORIGINAL.targets)[number]["id"];
const transform = (x: number, y: number, scale = 1) => ({
  anchor: { x: 0, y: 0 },
  opacity: 1,
  position: { unit: "pixels", x, y },
  rotationDeg: 0,
  scaleX: scale,
  scaleY: scale,
  skewXDeg: 0,
  skewYDeg: 0,
});
const base = (id: string, order: number, x: number, y: number) => ({
  durationMs: 3000,
  id,
  keyframes: [],
  stackOrder: order,
  startMs: 0,
  transform2d: transform(x, y),
});
const paint = (rgb: readonly number[]) => ({
  color: {
    a: 1,
    b: (rgb[2] ?? 0) / 255,
    g: (rgb[1] ?? 0) / 255,
    r: (rgb[0] ?? 0) / 255,
  },
  type: "solid",
});
const rectangle = (
  id: string,
  order: number,
  box: number[],
  rgb: number[]
) => ({
  ...base(id, order, box[0] ?? 0, box[1] ?? 0),
  fill: paint(rgb),
  geometry: { height: box[3], type: "rectangle", width: box[2] },
  stroke: null,
  type: "shape",
});
const text = (
  id: string,
  order: number,
  value: string,
  x: number,
  y: number,
  width: number,
  height: number,
  fontSize: number
) => ({
  ...base(id, order, x, y),
  color: "#ffffff",
  fontSize,
  style: {
    layout: { bounds: { heightPx: height, widthPx: width }, wrap: "none" },
  },
  text: value,
  type: "text",
});
const artwork = (assetId: string, x: number, y: number, order: number) => ({
  ...base("artwork", order, x, y),
  assetId,
  audio: { fadeInMs: 0, fadeOutMs: 0, muted: true, volume: 1 },
  sourceInMs: 0,
  transform2d: transform(x, y, 4),
  type: "media",
});
const slots = (assetId: string, layer = "label") => [
  {
    binding: { property: "text.document", targetLayerId: layer },
    constraints: { maxLength: 24, minLength: 1 },
    defaultValue: { type: "text", value: "NATIVE INTRO" },
    id: "label",
    kind: "text",
    name: "Label",
    required: true,
  },
  {
    binding: { property: "media.asset", targetLayerId: "artwork" },
    constraints: { assetKinds: ["image"] },
    defaultValue: {
      type: "asset",
      value: { id: assetId, kind: "asset", scope: "project" },
    },
    id: "artwork",
    kind: "asset",
    name: "Artwork",
    required: true,
  },
];
const component = (name: string, items: unknown[], values: unknown[] = []) =>
  componentFieldsSchema.parse({
    durationMs: 3000,
    height: 1080,
    name,
    slots: values,
    tracks: [
      {
        id: "native_layers",
        items,
        name: "Original illustrated native layers",
        trackType: "overlay",
      },
    ],
    width: 1920,
  });
const pathShape = (
  id: string,
  order: number,
  commands: unknown[],
  rgb: number[],
  origin: [number, number]
) => {
  // Native path rasters normalize their geometric bounding origin. Author local
  // points and an explicit canvas transform so both semantics and pixels agree.
  const localCommands = JSON.parse(JSON.stringify(commands), (_key, value) => {
    if (
      value &&
      typeof value === "object" &&
      typeof value.x === "number" &&
      typeof value.y === "number"
    ) {
      return { x: value.x - origin[0], y: value.y - origin[1] };
    }
    return value;
  });
  return {
    ...base(id, order, origin[0], origin[1]),
    fill: paint(rgb),
    geometry: {
      path: { commands: localCommands, fillRule: "nonzero" },
      type: "path",
    },
    stroke: null,
    type: "shape",
  };
};
const quad = (points: readonly (readonly number[])[]) => [
  ...points.map((p, i) => ({
    to: { x: p[0], y: p[1] },
    type: i === 0 ? "moveTo" : "lineTo",
  })),
  { type: "close" },
];
const instance = (
  alias: string,
  trackId: string,
  componentId: string,
  x = 0,
  y = 0,
  scale = 1,
  slotValues: object = {}
) => ({
  componentId,
  durationMs: 3000,
  operation: "add_component_instance",
  resultAlias: alias,
  slotValues,
  startMs: 0,
  timeScale: 1,
  trackId,
  transform: { opacity: 1, positionX: x, positionY: y, scale },
  trimStartMs: 0,
});

// These are new original test inputs. The historical #181 inventory and public
// recipe assets remain immutable. Explicit geometry carries every final role.
export const illustratedTargetOperations = (
  id: TargetId,
  assetId: string,
  overlay: string
): Record<string, unknown>[] => {
  if (id === "illustrated_device_intro") {
    const fields = component(
      id,
      [
        rectangle("backdrop", 0, [0, 0, 1920, 1080], [24, 38, 61]),
        rectangle("device_frame", 1, [260, 180, 880, 680], [238, 238, 238]),
        rectangle("screen_panel", 2, [300, 220, 800, 520], [22, 32, 51]),
        rectangle("label_panel", 3, [1200, 260, 500, 360], [48, 73, 104]),
        {
          ...rectangle("accent", 4, [300, 770, 220, 32], [255, 204, 51]),
          animationChannels: [
            {
              keyframes: [
                {
                  curve: "linear",
                  timeMs: 0,
                  value: { type: "scalar", value: 300 },
                },
                {
                  curve: "linear",
                  timeMs: 1500,
                  value: { type: "scalar", value: 320 },
                },
                {
                  curve: "hold",
                  timeMs: 2966,
                  value: { type: "scalar", value: 300 },
                },
              ],
              property: "transform.position_x",
            },
          ],
          transform: { opacity: 1, positionX: 300, positionY: 770, scale: 1 },
          transform2d: null,
        },
        artwork(assetId, 620, 360, 5),
        text("label", 6, "NATIVE INTRO", 1230, 330, 440, 100, 36),
      ],
      slots(assetId)
    );
    return [
      { operation: "component_create", resultAlias: "source", ...fields },
      instance("primary", overlay, "@source"),
    ];
  }
  if (id === "sticker_motif_intro") {
    const rays = Array.from({ length: 8 }, (_, index) => {
      const angle = (index * Math.PI) / 4;
      return quad(
        [
          [330, -16],
          [440, -16],
          [440, 16],
          [330, 16],
        ].map(([x = 0, y = 0]) => [
          960 + x * Math.cos(angle) - y * Math.sin(angle),
          540 + x * Math.sin(angle) + y * Math.cos(angle),
        ])
      );
    }).flat();
    const speed = Array.from({ length: 6 }, (_, i) =>
      quad([
        [160, 390 + i * 42],
        [440, 390 + i * 42],
        [440, 402 + i * 42],
        [160, 402 + i * 42],
      ])
    ).flat();
    const vertices = [
      [640, 140],
      [1280, 140],
      [1360, 220],
      [1360, 860],
      [1280, 940],
      [640, 940],
      [560, 860],
      [560, 220],
    ];
    const contour = [
      ...vertices.map(([x = 0, y = 0], index) => ({
        to: { x, y },
        type: index === 0 ? "moveTo" : "lineTo",
      })),
      { type: "close" },
    ];
    const fields = component(
      id,
      [
        rectangle("backdrop", 0, [0, 0, 1920, 1080], [24, 38, 61]),
        {
          ...base("dot_backdrop", 1, 0, 0),
          grid: {
            height: 1080,
            pattern: {
              paint: paint([0, 212, 255]),
              radius: 5,
              spacingX: 320,
              spacingY: 320,
              type: "dot",
            },
            width: 1920,
          },
          type: "grid",
        },
        pathShape("sun_rays", 2, rays, [255, 204, 51], [520, 100]),
        pathShape("speed_lines", 3, speed, [255, 107, 0], [160, 390]),
        pathShape(
          "authored_sticker_contour",
          4,
          contour,
          [48, 73, 104],
          [560, 140]
        ),
        artwork(assetId, 832, 412, 5),
        text("label", 6, "NATIVE INTRO", 650, 940, 700, 80, 36),
      ],
      slots(assetId)
    );
    return [
      { operation: "component_create", resultAlias: "source", ...fields },
      instance("primary", overlay, "@source"),
    ];
  }
  if (id !== "role_badge_team_intro") {
    throw new Error("Unknown illustrated target");
  }
  const badge = structuredClone(TEAM.badgeCreate);
  badge.durationMs = 3000;
  for (const track of badge.tracks) {
    for (const item of track.items) {
      item.durationMs = 3000;
      if (item.type === "media" && "assetId" in item) {
        item.assetId = assetId;
      }
    }
  }
  const surround = component("team_surround", [
    rectangle("backdrop", 0, [0, 0, 1920, 1080], [24, 38, 61]),
    text("team_title", 1, "NATIVE TEAM", 600, 100, 900, 100, 64),
  ]);
  const defineSlots = JSON.parse(
    JSON.stringify(TEAM.badgeDefineSlots),
    (_key, value) => (value === "$artwork" ? assetId : value)
  );
  const operations: Record<string, unknown>[] = [
    { operation: "component_create", resultAlias: "surround", ...surround },
    instance("surround_instance", overlay, "@surround"),
    badge,
    defineSlots,
    {
      durationMs: 3000,
      operation: "add_group",
      resultAlias: "team",
      startMs: 0,
      trackId: overlay,
    },
  ];
  for (let i = 0; i < 3; i += 1) {
    const alias = `member_${i + 1}`;
    operations.push(
      instance(alias, overlay, "@badge", 160 + 560 * i, 403, 0.6, {
        role_label: { type: "text", value: `ROLE 0${i + 1}` },
      }),
      {
        itemId: `@${alias}`,
        operation: "item_set_parent",
        parent: { id: "@team", scope: "root" },
      }
    );
  }
  return operations;
};
export const TARGET_IDS = ORIGINAL.targets.map((t) => t.id);
export const ORIGINAL_TARGETS = ORIGINAL.targets;
