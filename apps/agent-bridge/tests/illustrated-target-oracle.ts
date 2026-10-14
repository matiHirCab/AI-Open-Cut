import type { z } from "zod/v4";
import TEAM from "../../../contracts/native-role-team-v1.json";
import type { projectStateSchema } from "../src/schemas";
import {
  ORIGINAL_TARGETS,
  TARGET_TIMING,
  type TargetId,
} from "./fixtures/illustrated-targets";

type State = z.infer<typeof projectStateSchema>;
type Project = State["project"];
type Root = Project["tracks"][number]["items"][number];
type Child = Project["components"][number]["tracks"][number]["items"][number];
type Item = Root | Child;
export interface LeafObservation {
  animatedChannels: number;
  artworkId?: string | undefined;
  box: number[];
  id: string;
  kind: string;
  pathCommands: number;
  text: string;
}
export interface TargetObservation {
  artworkDimensions: number[];
  durationMs: number;
  groups: number;
  id: TargetId;
  instanceAnimatedChannels?: number;
  instances: number;
  itemDurations: number[];
  leaves: LeafObservation[];
  settings: { width: number; height: number; fps: number };
  storedArtworkAssets: number;
}
const must = (condition: boolean, message: string) => {
  if (!condition) {
    throw new Error(`Illustrated target: ${message}`);
  }
};
const scalarCount = (text: string) => [...text].length;
const pathPoints = (value: object): { x: number; y: number }[] => {
  const points: { x: number; y: number }[] = [];
  const visit = (v: unknown) => {
    if (!v || typeof v !== "object") {
      return;
    }
    if (
      "x" in v &&
      "y" in v &&
      typeof v.x === "number" &&
      typeof v.y === "number"
    ) {
      points.push({ x: v.x, y: v.y });
    } else {
      for (const child of Object.values(v)) {
        visit(child);
      }
    }
  };
  visit(value);
  return points;
};
const localBox = (item: Item, project: Project): number[] => {
  if (item.type === "shape") {
    if (
      item.geometry.type === "rectangle" ||
      item.geometry.type === "ellipse"
    ) {
      return [0, 0, item.geometry.width, item.geometry.height];
    }
    const points = pathPoints(item.geometry);
    const xs = points.map((p) => p.x),
      ys = points.map((p) => p.y);
    return [
      0,
      0,
      Math.max(...xs) - Math.min(...xs),
      Math.max(...ys) - Math.min(...ys),
    ];
  }
  if (item.type === "grid") {
    return [0, 0, item.grid.width, item.grid.height];
  }
  if (item.type === "text") {
    const bounds = item.style.layout?.bounds;
    return [0, 0, bounds?.widthPx ?? 0, bounds?.heightPx ?? 0];
  }
  if (item.type === "media") {
    const asset = project.assets.find((a) => a.id === item.assetId);
    return [0, 0, asset?.probe.videoWidth ?? 0, asset?.probe.videoHeight ?? 0];
  }
  throw new Error(`Unsupported fixture leaf ${item.type}`);
};
const textValue = (value: unknown, fallback: string) =>
  value &&
  typeof value === "object" &&
  "value" in value &&
  typeof value.value === "string"
    ? value.value
    : fallback;
const assetValue = (value: unknown, fallback: string | undefined) => {
  if (
    value &&
    typeof value === "object" &&
    "value" in value &&
    value.value &&
    typeof value.value === "object" &&
    "id" in value.value &&
    typeof value.value.id === "string"
  ) {
    return value.value.id;
  }
  return fallback;
};
const slotOverrides = (
  source: Project["components"][number],
  item: Extract<Item, { type: "component_instance" }>,
  child: Item
) =>
  Object.fromEntries(
    source.slots
      .filter((slot) => slot.binding.targetLayerId === child.id)
      .map((slot) => [slot.kind, item.slotValues[slot.id] ?? slot.defaultValue])
  );
const recordLeaf = (
  item: Item,
  project: Project,
  result: TargetObservation,
  placement: number[],
  overrides: Record<string, unknown>
) => {
  const [nx = 0, ny = 0, nsx = 1, nsy = 1] = placement;
  if (item.type === "media") {
    const original = project.assets.find((a) => a.id === item.assetId);
    if (original?.mediaType === "audio") {
      return;
    }
  }
  const box = localBox(item, project);
  const text = textValue(overrides.text, item.type === "text" ? item.text : "");
  const artworkId = assetValue(
    overrides.asset,
    item.type === "media" ? item.assetId : undefined
  );
  if (artworkId) {
    const asset = project.assets.find((a) => a.id === artworkId);
    result.artworkDimensions.push(
      Math.max(asset?.probe.videoWidth ?? 0, asset?.probe.videoHeight ?? 0)
    );
  }
  result.leaves.push({
    animatedChannels: item.animationChannels?.length ?? 0,
    artworkId,
    box: [
      nx + (box[0] ?? 0) * nsx,
      ny + (box[1] ?? 0) * nsy,
      (box[2] ?? 0) * nsx,
      (box[3] ?? 0) * nsy,
    ],
    id: item.id,
    kind: item.type,
    pathCommands:
      item.type === "shape" && item.geometry.type === "path"
        ? item.geometry.path.commands.length
        : 0,
    text,
  });
};
export const observeIllustratedTarget = (
  id: TargetId,
  state: State
): TargetObservation => {
  const result: TargetObservation = {
    artworkDimensions: [],
    durationMs: state.durationMs,
    groups: 0,
    id,
    instanceAnimatedChannels: 0,
    instances: 0,
    itemDurations: [],
    leaves: [],
    settings: state.project.settings,
    storedArtworkAssets: state.project.assets.filter(
      (asset) => asset.mediaType === "image"
    ).length,
  };
  const walk = (
    item: Item,
    x: number,
    y: number,
    sx: number,
    sy: number,
    overrides: Record<string, unknown> = {},
    ancestors: string[] = []
  ) => {
    result.itemDurations.push(item.durationMs);
    const t = item.transform2d;
    must(
      !t ||
        (t.position.unit === "pixels" &&
          t.anchor.x === 0 &&
          t.anchor.y === 0 &&
          t.rotationDeg === 0 &&
          t.skewXDeg === 0 &&
          t.skewYDeg === 0),
      "unsupported fixture transform"
    );
    const px = t?.position.x ?? item.transform.positionX,
      py = t?.position.y ?? item.transform.positionY;
    const nx = x + px * sx,
      ny = y + py * sy,
      nsx = sx * (t?.scaleX ?? item.transform.scale),
      nsy = sy * (t?.scaleY ?? item.transform.scale);
    if (item.type === "group") {
      result.groups += 1;
      result.instanceAnimatedChannels =
        (result.instanceAnimatedChannels ?? 0) +
        (item.animationChannels?.length ?? 0);
      return;
    }
    if (item.type === "component_instance") {
      must(!ancestors.includes(item.componentId), "component cycle");
      result.instances += 1;
      result.instanceAnimatedChannels =
        (result.instanceAnimatedChannels ?? 0) +
        (item.animationChannels?.length ?? 0);
      const source = state.project.components.find(
        (c) => c.id === item.componentId
      );
      if (!source) {
        throw new Error("Missing source component");
      }
      result.itemDurations.push(source.durationMs);
      for (const child of source.tracks.flatMap((track) => track.items)) {
        walk(child, nx, ny, nsx, nsy, slotOverrides(source, item, child), [
          ...ancestors,
          item.componentId,
        ]);
      }
      return;
    }
    recordLeaf(item, state.project, result, [nx, ny, nsx, nsy], overrides);
  };
  for (const track of state.project.tracks) {
    for (const item of track.items) {
      // This fixture authors an identity root parent; independently reject drift.
      if (item.parent) {
        const group = state.project.tracks
          .flatMap((t) => t.items)
          .find((i) => i.id === item.parent?.id);
        must(
          group?.type === "group" &&
            group.transform.positionX === 0 &&
            group.transform.positionY === 0 &&
            group.transform.scale === 1 &&
            (!group.transform2d ||
              (group.transform2d.position.unit === "pixels" &&
                group.transform2d.position.x === 0 &&
                group.transform2d.position.y === 0 &&
                group.transform2d.scaleX === 1 &&
                group.transform2d.scaleY === 1 &&
                group.transform2d.anchor.x === 0 &&
                group.transform2d.anchor.y === 0 &&
                group.transform2d.rotationDeg === 0 &&
                group.transform2d.skewXDeg === 0 &&
                group.transform2d.skewYDeg === 0)),
          "team parent transform changed"
        );
      }
      walk(item, 0, 0, 1, 1);
    }
  }
  return result;
};
const inside = (box: number[], envelope: number[]) => {
  const [x = -1, y = -1, width = 0, height = 0] = box;
  const [left = 0, top = 0, right = 0, bottom = 0] = envelope;
  return (
    box.length === 4 &&
    box.every(Number.isFinite) &&
    width > 0 &&
    height > 0 &&
    x >= left - 0.01 &&
    y >= top - 0.01 &&
    x + width <= right + 0.01 &&
    y + height <= bottom + 0.01
  );
};
export const validateIllustratedTarget = (observation: TargetObservation) => {
  const target = ORIGINAL_TARGETS.find((t) => t.id === observation.id);
  if (!target) {
    throw new Error("Unknown original target");
  }
  must(
    observation.settings.width === TARGET_TIMING.canvas.width &&
      observation.settings.height === TARGET_TIMING.canvas.height &&
      observation.settings.fps === TARGET_TIMING.canvas.fps,
    "canvas changed"
  );
  must(
    observation.durationMs === 3000 &&
      observation.itemDurations.every((d) => d === 3000),
    "three-second duration changed"
  );
  const roles = (id: string) =>
    observation.leaves.filter((leaf) => leaf.id === id);
  const one = (id: string, envelope: number[]) => {
    const matching = roles(id);
    must(
      matching.length === 1 && inside(matching[0]?.box ?? [], envelope),
      `${id} role/layout missing`
    );
  };
  if (observation.id === "illustrated_device_intro") {
    one("device_frame", [260, 180, 1140, 860]);
    one("screen_panel", [260, 180, 1140, 860]);
    one("artwork", [620, 360, 876, 616]);
    one("label", [1200, 260, 1700, 620]);
    one("accent", [260, 180, 1140, 860]);
    one("backdrop", [0, 0, 1920, 1080]);
  } else if (observation.id === "sticker_motif_intro") {
    one("authored_sticker_contour", [560, 140, 1360, 940]);
    one("artwork", [560, 140, 1360, 940]);
    one("label", [560, 900, 1360, 1020]);
    one("dot_backdrop", [0, 0, 1920, 1080]);
    one("speed_lines", [0, 0, 1920, 1080]);
    one("sun_rays", [0, 0, 1920, 1080]);
    const box = roles("sun_rays")[0]?.box ?? [];
    must(
      Math.abs((box[0] ?? 0) + (box[2] ?? 0) / 2 - 960) < 0.01 &&
        Math.abs((box[1] ?? 0) + (box[3] ?? 0) / 2 - 540) < 0.01,
      "ray centre changed"
    );
  } else {
    must(observation.groups === 1, "team_parent missing");
    must(
      TEAM.roles.every((role) => roles(role).length === 3),
      "native role_badges missing"
    );
    must(
      roles("background").length === 3 &&
        roles("artwork").length === 3 &&
        roles("role_label").length === 3,
      "three role badges missing"
    );
    for (const [index, left] of [160, 720, 1280].entries()) {
      must(
        inside(roles("background")[index]?.box ?? [], [
          left,
          0,
          left + 480,
          1080,
        ]),
        "badge column changed"
      );
      must(
        inside(roles("artwork")[index]?.box ?? [], [
          left,
          260,
          left + 480,
          660,
        ]),
        "member artwork layout changed"
      );
      must(
        inside(roles("role_label")[index]?.box ?? [], [
          left,
          700,
          left + 480,
          820,
        ]),
        "member label layout changed"
      );
    }
    one("team_title", [0, 80, 1920, 200]);
    one("backdrop", [0, 0, 1920, 1080]);
  }
  const inventory = {
    animatedChannels: observation.leaves.reduce(
      (sum, leaf) => sum + leaf.animatedChannels,
      observation.instanceAnimatedChannels ?? 0
    ),
    artworkCount: observation.leaves.filter((leaf) => leaf.artworkId).length,
    artworkDimension: Math.max(0, ...observation.artworkDimensions),
    distinctArtworkAssets: new Set(
      observation.leaves.flatMap((leaf) =>
        leaf.artworkId ? [leaf.artworkId] : []
      )
    ).size,
    instances: observation.instances,
    pathCommands: observation.leaves.reduce(
      (sum, leaf) => sum + leaf.pathCommands,
      0
    ),
    storedArtworkAssets: observation.storedArtworkAssets,
    textScalars: observation.leaves.reduce(
      (sum, leaf) => sum + scalarCount(leaf.text),
      0
    ),
    visualLeaves: observation.leaves.length,
  };
  for (const [key, cap] of Object.entries(target.caps)) {
    must(
      inventory[key as keyof typeof target.caps] <= cap,
      `${key} cap exceeded`
    );
  }
  must(inventory.artworkDimension > 0, "unprobed artwork");
  return inventory;
};
