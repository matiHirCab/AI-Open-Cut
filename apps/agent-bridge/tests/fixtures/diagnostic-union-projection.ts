// Exact rollback of the reviewed diagnostic additions and graphic color slots.
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import pins from "./ffmpeg-spawn-predecessor-pins.json";
import {
  removeFfmpegSpawnHeadlessAddition,
  removeFfmpegSpawnMcpAddition,
  removeFfmpegSpawnOwnershipAddition,
} from "./ffmpeg-spawn-projection";
import {
  removeRenderBudgetHeadlessAddition,
  removeRenderBudgetMcpAddition,
  removeRenderBudgetOwnershipAddition,
} from "./render-budget-projection";
import {
  removeSolidGraphicHeadlessAddition,
  removeSolidGraphicMcpAdditions,
  removeSolidGraphicOwnershipAddition,
} from "./solid-graphic-color-slots-projection";

const sorted = (value: unknown): unknown => {
  if (Array.isArray(value)) {
    return value.map(sorted);
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, child]) => [key, sorted(child)])
    );
  }
  return value;
};

export const restoreDiagnosticUnionRaw = (source: string): string => {
  const value = JSON.parse(source);
  let name: keyof typeof pins.rawSha256;
  let projected: unknown;
  if (
    value.$defs?.FfmpegSpawnDiagnostic ||
    value.$defs?.RenderBudgetDiagnostic ||
    value.capabilityIdentifiers?.includes("solid_graphic_color_slots_v1")
  ) {
    name = "mcp-surface-v1";
    projected = removeFfmpegSpawnMcpAddition(
      removeRenderBudgetMcpAddition(removeSolidGraphicMcpAdditions(value))
    );
  } else if (
    value.categories?.ffmpegSpawnDiagnostics ||
    value.categories?.nativeRenderBudgetDiagnostics ||
    value.categories?.solidGraphicColorSlots
  ) {
    name = "contract-ownership-v1";
    projected = removeFfmpegSpawnOwnershipAddition(
      removeRenderBudgetOwnershipAddition(
        removeSolidGraphicOwnershipAddition(value)
      )
    );
  } else if (
    value.ffmpegSpawnError ||
    value.renderBudgetError ||
    value.status?.editorCapabilities?.includes("solid_graphic_color_slots_v1")
  ) {
    name = "headless-protocol-v1";
    projected = removeFfmpegSpawnHeadlessAddition(
      removeRenderBudgetHeadlessAddition(
        removeSolidGraphicHeadlessAddition(value)
      )
    );
  } else {
    return source;
  }
  const raw =
    name === "headless-protocol-v1"
      ? readFileSync(
          new URL(
            "./ffmpeg-spawn-predecessor-headless.raw.txt",
            import.meta.url
          ),
          "utf8"
        )
      : `${JSON.stringify(projected, null, 2)}\n`;
  if (
    JSON.stringify(sorted(JSON.parse(raw))) !==
      JSON.stringify(sorted(projected)) ||
    createHash("sha256").update(raw).digest("hex") !== pins.rawSha256[name]
  ) {
    throw new Error(
      `Unrelated ${name} predecessor contract drift in diagnostic/color union rollback`
    );
  }
  return raw;
};
