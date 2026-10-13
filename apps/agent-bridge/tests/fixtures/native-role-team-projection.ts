import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import addition from "./native-role-team-ownership-addition.json";
import pins from "./native-role-team-predecessor-pins.json";

const record = (value: unknown): Record<string, unknown> => {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Malformed role team ownership");
  }
  return value as Record<string, unknown>;
};
export const assertCurrentRoleTeamOwnership = (source: unknown): void => {
  const categories = record(record(source).categories);
  if (JSON.stringify(categories.nativeRoleTeam) !== JSON.stringify(addition)) {
    throw new Error("Missing or incomplete current role team ownership");
  }
};
export const removeRoleTeamOwnershipAddition = <T>(source: T): T => {
  const result = structuredClone(source);
  const categories = record(record(result).categories);
  if (Object.hasOwn(categories, "nativeRoleTeam")) {
    assertCurrentRoleTeamOwnership(result);
    Reflect.deleteProperty(categories, "nativeRoleTeam");
  }
  return result;
};
export const restoreRoleTeamOwnershipRaw = (source: string): string => {
  const projected = removeRoleTeamOwnershipAddition(JSON.parse(source));
  const raw = readFileSync(
    new URL("./native-role-team-predecessor-ownership.raw", import.meta.url),
    "utf8"
  );
  if (
    JSON.stringify(projected) !== JSON.stringify(JSON.parse(raw)) ||
    createHash("sha256").update(raw).digest("hex") !==
      pins["contract-ownership-v1"]
  ) {
    throw new Error("Unrelated role team predecessor drift");
  }
  return raw;
};
