import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import catalog from "../../../../contracts/ffmpeg-spawn-diagnostics-v1.json";
import ownership from "./ffmpeg-spawn-ownership-addition.json";
import pins from "./ffmpeg-spawn-predecessor-pins.json";

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
export const removeFfmpegSpawnMcpAddition = <T>(source: T, copy = true): T => {
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
    Object.hasOwn(props, "ffmpegSpawn")
  );
  const hasDefinition = Object.hasOwn(
    (result as { $defs?: Record<string, unknown> }).$defs ?? {},
    "FfmpegSpawnDiagnostic"
  );
  if (
    (withField.length || hasDefinition) &&
    withField.length !== errors.length
  ) {
    throw new Error("Missing FFmpeg spawn schema addition");
  }
  for (const props of withField) {
    if (
      !(
        same(props.ffmpegSpawn, { $ref: "#/$defs/FfmpegSpawnDiagnostic" }) ||
        same(props.ffmpegSpawn, catalog.schema)
      )
    ) {
      throw new Error("Unapproved FFmpeg spawn schema drift");
    }
    Reflect.deleteProperty(props, "ffmpegSpawn");
  }
  const definitions = (result as { $defs?: Record<string, unknown> }).$defs;
  if (definitions && Object.hasOwn(definitions, "FfmpegSpawnDiagnostic")) {
    if (!same(definitions.FfmpegSpawnDiagnostic, catalog.schema)) {
      throw new Error("Unapproved FFmpeg spawn definition drift");
    }
    Reflect.deleteProperty(definitions, "FfmpegSpawnDiagnostic");
  }
  return result;
};

export const removeFfmpegSpawnOwnershipAddition = <T>(source: T): T => {
  const result = structuredClone(source);
  const { categories } = result as { categories?: Record<string, unknown> };
  if (categories && Object.hasOwn(categories, "ffmpegSpawnDiagnostics")) {
    if (!same(categories.ffmpegSpawnDiagnostics, ownership)) {
      throw new Error("Unapproved FFmpeg spawn ownership drift");
    }
    Reflect.deleteProperty(categories, "ffmpegSpawnDiagnostics");
  }
  return result;
};

export const removeFfmpegSpawnHeadlessAddition = <T>(source: T): T => {
  const result = structuredClone(source) as T & { ffmpegSpawnError?: unknown };
  if (Object.hasOwn(result, "ffmpegSpawnError")) {
    if (!same(result.ffmpegSpawnError, catalog.headlessErrorExample)) {
      throw new Error("Unapproved FFmpeg spawn headless example drift");
    }
    Reflect.deleteProperty(result, "ffmpegSpawnError");
  }
  return result;
};

export const restoreFfmpegSpawnRaw = (source: string): string => {
  const value = JSON.parse(source);
  let name: keyof typeof pins.rawSha256;
  let projected: unknown;
  if (value.$defs?.FfmpegSpawnDiagnostic) {
    name = "mcp-surface-v1";
    projected = removeFfmpegSpawnMcpAddition(value);
  } else if (value.categories?.ffmpegSpawnDiagnostics) {
    name = "contract-ownership-v1";
    projected = removeFfmpegSpawnOwnershipAddition(value);
  } else if (value.ffmpegSpawnError) {
    name = "headless-protocol-v1";
    projected = removeFfmpegSpawnHeadlessAddition(value);
  } else {
    return source;
  }
  const restored =
    name === "headless-protocol-v1"
      ? readFileSync(
          new URL(
            "./ffmpeg-spawn-predecessor-headless.raw.txt",
            import.meta.url
          ),
          "utf8"
        )
      : `${JSON.stringify(projected, null, 2)}\n`;
  if (!same(JSON.parse(restored), projected)) {
    throw new Error(
      `Unrelated ${name} semantic drift in FFmpeg spawn rollback`
    );
  }
  if (
    createHash("sha256").update(restored).digest("hex") !== pins.rawSha256[name]
  ) {
    throw new Error(`Unrelated ${name} drift in FFmpeg spawn rollback`);
  }
  return restored;
};
