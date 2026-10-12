import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import pins from "./ffmpeg-spawn-predecessor-pins.json";
import {
  removeFfmpegSpawnHeadlessAddition,
  removeFfmpegSpawnMcpAddition,
  removeFfmpegSpawnOwnershipAddition,
} from "./ffmpeg-spawn-projection";
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

// Exact combined rollback to the independently captured shared predecessor.
export const restoreFfmpegSpawnGraphicRaw = (source: string): string => {
  const value = JSON.parse(source);
  let name: keyof typeof pins.rawSha256;
  let projected: unknown;
  if (
    Object.hasOwn(value.$defs ?? {}, "FfmpegSpawnDiagnostic") ||
    value.capabilityIdentifiers?.includes("solid_graphic_color_slots_v1")
  ) {
    name = "mcp-surface-v1";
    projected = removeSolidGraphicMcpAdditions(
      removeFfmpegSpawnMcpAddition(value)
    );
  } else if (
    value.categories?.ffmpegSpawnDiagnostics ||
    value.categories?.solidGraphicColorSlots
  ) {
    name = "contract-ownership-v1";
    projected = removeSolidGraphicOwnershipAddition(
      removeFfmpegSpawnOwnershipAddition(value)
    );
  } else if (
    value.ffmpegSpawnError ||
    value.status?.editorCapabilities?.includes("solid_graphic_color_slots_v1")
  ) {
    name = "headless-protocol-v1";
    projected = removeSolidGraphicHeadlessAddition(
      removeFfmpegSpawnHeadlessAddition(value)
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
      `Unrelated ${name} predecessor contract drift in FFmpeg spawn/graphic rollback`
    );
  }
  return raw;
};
