import { createHash } from "node:crypto";
import addition from "./job-restart-ownership-addition.json";
import { orderedEffectDigest } from "./ordered-effect-projection";
import {
  removeSolidGraphicMcpAdditions,
  removeSolidGraphicOwnershipAddition,
  restoreSolidGraphicRaw,
} from "./solid-graphic-color-slots-projection";

const record = (value: unknown): Record<string, unknown> => {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Malformed restart ownership projection");
  }
  return value as Record<string, unknown>;
};
export const removeJobRestartOwnershipAddition = <T>(source: T): T => {
  const result = removeSolidGraphicOwnershipAddition(source);
  const categories = record(record(result).categories);
  if (Object.hasOwn(categories, "jobRestart")) {
    if (
      orderedEffectDigest(categories.jobRestart) !==
      orderedEffectDigest(addition.category)
    ) {
      throw new Error("Incorrect restart ownership addition");
    }
    Reflect.deleteProperty(categories, "jobRestart");
  }
  return result;
};
export const removeJobRestartMcpAddition = <T>(source: T): T => {
  const result = removeSolidGraphicMcpAdditions(source);
  const capabilities = record(result).capabilityIdentifiers;
  if (!Array.isArray(capabilities)) {
    throw new Error("Malformed restart capability projection");
  }
  const matches = capabilities.flatMap((value, index) =>
    value === "ephemeral_jobs_v1" ? [index] : []
  );
  if (matches.length > 1) {
    throw new Error("Duplicate restart capability addition");
  }
  if (matches.length === 1) {
    capabilities.splice(matches[0] ?? -1, 1);
  }
  return result;
};
// Reconstruct only the reviewed additive rollback, then require its exact
// independently captured pre-change raw digest before older projections run.
export const restoreJobRestartRaw = (input: string): string => {
  const source = restoreSolidGraphicRaw(input);
  const value = record(JSON.parse(source));
  let projected: unknown;
  let expected: string;
  if (
    Array.isArray(value.capabilityIdentifiers) &&
    value.capabilityIdentifiers.includes("ephemeral_jobs_v1")
  ) {
    projected = removeJobRestartMcpAddition(value);
    expected = addition.mcpPredecessorSha256;
  } else if (
    value.categories &&
    Object.hasOwn(record(value.categories), "jobRestart")
  ) {
    projected = removeJobRestartOwnershipAddition(value);
    expected = addition.predecessorSha256;
  } else {
    return source;
  }
  const raw = `${JSON.stringify(projected, null, 2)}\n`;
  if (createHash("sha256").update(raw).digest("hex") !== expected) {
    throw new Error("Restart contract predecessor changed");
  }
  return raw;
};
