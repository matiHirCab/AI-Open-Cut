import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import {
  removeNativeMotifOwnershipAddition,
  restoreNativeMotifRaw,
} from "./native-motif-pack-projection";
import pins from "./render-budget-predecessor-pins.json";
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

// Exact combined rollback to the independently captured shared predecessor.
export const restoreRenderBudgetGraphicRaw = (input: string): string => {
  const source = restoreNativeMotifRaw(input);
  const value = JSON.parse(source);
  let name: keyof typeof pins.rawSha256;
  let projected: unknown;
  if (
    Object.hasOwn(value.$defs ?? {}, "RenderBudgetDiagnostic") ||
    value.capabilityIdentifiers?.includes("solid_graphic_color_slots_v1")
  ) {
    name = "mcp-surface-v1";
    projected = removeSolidGraphicMcpAdditions(
      removeRenderBudgetMcpAddition(value)
    );
  } else if (
    value.categories?.nativeRenderBudgetDiagnostics ||
    value.categories?.solidGraphicColorSlots
  ) {
    name = "contract-ownership-v1";
    projected = removeSolidGraphicOwnershipAddition(
      removeRenderBudgetOwnershipAddition(
        removeNativeMotifOwnershipAddition(value)
      )
    );
  } else if (
    value.renderBudgetError ||
    value.status?.editorCapabilities?.includes("solid_graphic_color_slots_v1")
  ) {
    name = "headless-protocol-v1";
    projected = removeSolidGraphicHeadlessAddition(
      removeRenderBudgetHeadlessAddition(value)
    );
  } else {
    return source;
  }
  const raw =
    name === "headless-protocol-v1"
      ? readFileSync(
          new URL(
            "./render-budget-predecessor-headless.raw.txt",
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
      `Unrelated ${name} predecessor contract drift in native budget/graphic rollback`
    );
  }
  return raw;
};
