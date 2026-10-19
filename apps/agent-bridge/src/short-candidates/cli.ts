import { BridgeError } from "../headless";
import {
  PLANNING_LIMITS,
  PLANNING_MESSAGES,
  PlanningError,
  shortPlanningFailureSchema,
} from "./contract";
import type { ShortCandidatePlanner } from "./service";

export const planningFailure = (error: unknown) => {
  if (error instanceof PlanningError) {
    return shortPlanningFailureSchema.parse({
      error: { kind: error.kind, message: PLANNING_MESSAGES[error.kind] },
      ok: false,
      version: 1,
    });
  }
  if (error instanceof BridgeError) {
    return shortPlanningFailureSchema.parse({
      error: {
        code: error.code,
        kind: "core",
        message: PLANNING_MESSAGES.core,
        retryable: error.retryable,
      },
      ok: false,
      version: 1,
    });
  }
  return shortPlanningFailureSchema.parse({
    error: { kind: "internal", message: PLANNING_MESSAGES.internal },
    ok: false,
    version: 1,
  });
};

export const readPlanningInput = async (
  stream: AsyncIterable<Uint8Array>
): Promise<unknown> => {
  const buffers: Uint8Array[] = [];
  let bytes = 0;
  for await (const chunk of stream) {
    bytes += chunk.byteLength;
    if (bytes > PLANNING_LIMITS.inputBytes) {
      throw new PlanningError("limit");
    }
    buffers.push(chunk);
  }
  try {
    return JSON.parse(
      new TextDecoder("utf-8", { fatal: true }).decode(Buffer.concat(buffers))
    );
  } catch (cause) {
    throw new PlanningError("input", { cause });
  }
};

export const runPlanningCli = async (
  stream: AsyncIterable<Uint8Array>,
  planner: Pick<ShortCandidatePlanner, "plan">,
  output: (value: string) => Promise<void>,
  errors: (value: string) => Promise<void>
): Promise<number> => {
  try {
    const input = await readPlanningInput(stream);
    const plan = await planner.plan(input);
    const serialized = `${JSON.stringify(plan, null, 2)}\n`;
    if (Buffer.byteLength(serialized, "utf8") > PLANNING_LIMITS.outputBytes) {
      throw new PlanningError("limit");
    }
    await output(serialized);
    return 0;
  } catch (error) {
    try {
      await errors(`${JSON.stringify(planningFailure(error))}\n`);
    } catch {
      // A closed stderr is also an unsuccessful process, never another success.
    }
    return 1;
  }
};
