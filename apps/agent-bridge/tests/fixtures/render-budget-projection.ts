import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import catalog from "../../../../contracts/render-budget-diagnostics-v1.json";
import { restoreDiagnosticUnionRaw } from "./diagnostic-union-projection";
import {
  removeNativeMotifOwnershipAddition,
  restoreNativeMotifRaw,
} from "./native-motif-pack-projection";
import { restoreRenderBudgetGraphicRaw } from "./render-budget-graphic-projection";
import ownership from "./render-budget-ownership-addition.json";
import pins from "./render-budget-predecessor-pins.json";
import { removeRenderPreparationOwnershipAddition } from "./render-preparation-progress-projection";

const same = (a: unknown, b: unknown) => {
  const normalize = (value: unknown): unknown => {
    if (Array.isArray(value)) {
      return value.map(normalize);
    }
    if (value && typeof value === "object") {
      return Object.fromEntries(
        Object.entries(value)
          .sort(([left], [right]) => left.localeCompare(right))
          .map(([key, child]) => [key, normalize(child)])
      );
    }
    return value;
  };
  return JSON.stringify(normalize(a)) === JSON.stringify(normalize(b));
};

// Invert exactly the reviewed optional error field; retain all unrelated drift.
export const removeRenderBudgetMcpAddition = <T>(source: T, copy = true): T => {
  const result = copy ? structuredClone(source) : source;
  const errors: Record<string, unknown>[] = [];
  const walk = (value: unknown) => {
    if (!value || typeof value !== "object") {
      return;
    }
    if (!Array.isArray(value)) {
      const object = value as Record<string, unknown>;
      const props = object.properties as Record<string, unknown> | undefined;
      if (props && Object.hasOwn(props, "ffmpegStderrExcerpt")) {
        errors.push(props);
      }
    }
    for (const child of Object.values(value)) {
      walk(child);
    }
  };
  walk(result);
  const withField = errors.filter((props) =>
    Object.hasOwn(props, "renderBudget")
  );
  const hasDefinition = Object.hasOwn(
    (result as { $defs?: Record<string, unknown> }).$defs ?? {},
    "RenderBudgetDiagnostic"
  );
  if (
    (withField.length || hasDefinition) &&
    withField.length !== errors.length
  ) {
    throw new Error("Missing native budget schema addition");
  }
  for (const props of withField) {
    if (
      !(
        same(props.renderBudget, { $ref: "#/$defs/RenderBudgetDiagnostic" }) ||
        same(props.renderBudget, catalog.schema)
      )
    ) {
      throw new Error("Unapproved native budget schema drift");
    }
    Reflect.deleteProperty(props, "renderBudget");
  }
  const definitions = (result as { $defs?: Record<string, unknown> }).$defs;
  if (definitions && Object.hasOwn(definitions, "RenderBudgetDiagnostic")) {
    if (!same(definitions.RenderBudgetDiagnostic, catalog.schema)) {
      throw new Error("Unapproved native budget definition drift");
    }
    Reflect.deleteProperty(definitions, "RenderBudgetDiagnostic");
  }
  return result;
};

export const removeRenderBudgetOwnershipAddition = <T>(source: T): T => {
  const result = removeNativeMotifOwnershipAddition(
    removeRenderPreparationOwnershipAddition(source)
  );
  const { categories } = result as { categories?: Record<string, unknown> };
  if (
    categories &&
    Object.hasOwn(categories, "nativeRenderBudgetDiagnostics")
  ) {
    if (!same(categories.nativeRenderBudgetDiagnostics, ownership)) {
      throw new Error("Unapproved native budget ownership drift");
    }
    Reflect.deleteProperty(categories, "nativeRenderBudgetDiagnostics");
  }
  return result;
};

export const removeRenderBudgetHeadlessAddition = <T>(source: T): T => {
  const result = structuredClone(source) as T & { renderBudgetError?: unknown };
  if (Object.hasOwn(result, "renderBudgetError")) {
    if (!same(result.renderBudgetError, catalog.headlessErrorExample)) {
      throw new Error("Unapproved native budget headless example drift");
    }
    Reflect.deleteProperty(result, "renderBudgetError");
  }
  return result;
};

export const restoreRenderBudgetRaw = (input: string): string => {
  const source = restoreNativeMotifRaw(input);
  const value = JSON.parse(source);
  if (
    value.$defs?.FfmpegSpawnDiagnostic ||
    value.categories?.ffmpegSpawnDiagnostics ||
    value.ffmpegSpawnError
  ) {
    return restoreDiagnosticUnionRaw(source);
  }
  if (
    value.capabilityIdentifiers?.includes("solid_graphic_color_slots_v1") ||
    value.categories?.solidGraphicColorSlots ||
    value.status?.editorCapabilities?.includes("solid_graphic_color_slots_v1")
  ) {
    return restoreRenderBudgetGraphicRaw(source);
  }
  let name: keyof typeof pins.rawSha256;
  let projected: unknown;
  if (value.$defs?.RenderBudgetDiagnostic) {
    name = "mcp-surface-v1";
    projected = removeRenderBudgetMcpAddition(value);
  } else if (value.categories?.nativeRenderBudgetDiagnostics) {
    name = "contract-ownership-v1";
    projected = removeRenderBudgetOwnershipAddition(value);
  } else if (value.renderBudgetError) {
    name = "headless-protocol-v1";
    projected = removeRenderBudgetHeadlessAddition(value);
  } else {
    return source;
  }
  const restored =
    name === "headless-protocol-v1"
      ? readFileSync(
          new URL(
            "./render-budget-predecessor-headless.raw.txt",
            import.meta.url
          ),
          "utf8"
        )
      : `${JSON.stringify(projected, null, 2)}\n`;
  if (!same(JSON.parse(restored), projected)) {
    throw new Error(
      `Unrelated ${name} semantic drift in native budget rollback`
    );
  }
  if (
    createHash("sha256").update(restored).digest("hex") !== pins.rawSha256[name]
  ) {
    throw new Error(`Unrelated ${name} drift in native budget rollback`);
  }
  return restored;
};
