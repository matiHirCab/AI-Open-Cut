// Remove only the reviewed additive prompt; historical pins remain immutable.
export const removeMotionWorkflowPrompt = <T>(source: T): T => {
  const result = structuredClone(source);
  const { prompts } = result as { prompts?: unknown };
  if (!Array.isArray(prompts)) {
    throw new Error("Malformed motion workflow prompt catalog");
  }
  const matches = prompts.flatMap((value, index) =>
    value === "create_motion_graphics" ? [index] : []
  );
  if (matches.length > 1) {
    throw new Error("Duplicate motion workflow prompt addition");
  }
  if (matches.length === 1) {
    prompts.splice(matches[0] ?? -1, 1);
  }
  return result;
};
