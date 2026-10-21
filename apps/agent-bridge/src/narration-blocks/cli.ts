import { z } from "zod/v4";
import { BridgeError } from "../headless";
import type { JobTaskContext } from "../jobs";
import {
  narrationPlanInputSchema,
  narrationPlanSchema,
  speechMarkerPolicySchema,
} from "../schemas";
import { assertNarrationJsonBudget } from "./budget";
import { NarrationPublicationUnknownError } from "./errors";
import { narrationCaptionOptionsSchema } from "./transport";
import { type NarrationBlocksWorkflow, narrationFailure } from "./workflow";

export const narrationCliCommandSchema = z.discriminatedUnion("command", [
  z.strictObject({
    command: z.literal("plan"),
    input: narrationPlanInputSchema,
  }),
  z.strictObject({
    command: z.literal("preview"),
    groupId: z.string().optional(),
    plan: narrationPlanSchema,
  }),
  z.strictObject({
    caption: narrationCaptionOptionsSchema.optional(),
    command: z.literal("commit"),
    expectedRevision: z.int().nonnegative(),
    groupId: z.string(),
    markerPolicy: speechMarkerPolicySchema.optional(),
    projectId: z.string(),
    startMs: z.int().nonnegative(),
    trackId: z.string(),
  }),
  z.strictObject({ command: z.literal("discard"), groupId: z.string() }),
  z.strictObject({ command: z.literal("reconcile"), groupId: z.string() }),
  z.strictObject({ command: z.literal("abandon"), groupId: z.string() }),
]);

const INPUT_BYTES = 16 * 1024 * 1024;
const OUTPUT_BYTES = 1024 * 1024;
const invalid = (message: string) =>
  new BridgeError("INVALID_ARGUMENT", message);

export const runNarrationCli = async (
  stream: AsyncIterable<Uint8Array>,
  workflow: Pick<
    NarrationBlocksWorkflow,
    | "plan"
    | "preview"
    | "commit"
    | "discard"
    | "reconcile"
    | "publicationStatus"
  >,
  output: (value: string) => Promise<void>,
  context: () => JobTaskContext
): Promise<number> => {
  let failed = false;
  const emit = async (value: unknown) => {
    assertNarrationJsonBudget(value, OUTPUT_BYTES);
    const text = JSON.stringify(value);
    await output(`${text}\n`);
  };
  const dispatch = async (bytes: Uint8Array) => {
    let groupId: string | undefined;
    try {
      let input: unknown;
      try {
        input = JSON.parse(
          new TextDecoder("utf-8", { fatal: true }).decode(bytes),
          (key, value: unknown) => {
            if (key === "__proto__") {
              throw invalid("Unexpected reserved JSON key");
            }
            return value;
          }
        );
      } catch (error) {
        // biome-ignore lint/style/useErrorCause: BridgeError receives ErrorOptions as its fourth argument.
        throw new BridgeError(
          "INVALID_ARGUMENT",
          "Narration command must be a valid UTF8 JSON object",
          false,
          { cause: error }
        );
      }
      const parsed = narrationCliCommandSchema.safeParse(input);
      if (!parsed.success) {
        throw invalid("Invalid closed narration command fields");
      }
      const command = parsed.data;
      if ("groupId" in command) {
        ({ groupId } = command);
      }
      let result: unknown;
      switch (command.command) {
        case "plan":
          result = await workflow.plan(command.input);
          break;
        case "preview":
          result = await workflow.preview(
            command.plan,
            context(),
            command.groupId
          );
          break;
        case "commit": {
          const { command: _command, ...fields } = command;
          result = await workflow.commit(fields, context());
          break;
        }
        case "reconcile":
          result = await workflow.reconcile(command.groupId);
          break;
        case "discard":
          result = await workflow.discard(command.groupId);
          break;
        case "abandon":
          result = await workflow.discard(command.groupId, true);
          break;
        default:
          throw invalid("Unsupported narration command");
      }
      if (result && typeof result === "object" && "error" in result) {
        failed = true;
      }
      await emit({ result });
    } catch (error) {
      failed = true;
      await emit({
        error: narrationFailure(error),
        ...(error instanceof NarrationPublicationUnknownError
          ? {
              publicationUnknown: true,
              ...(groupId ? workflow.publicationStatus(groupId) : {}),
            }
          : {}),
      });
    }
  };
  let fragments: Uint8Array[] = [];
  let buffered = 0;
  try {
    for await (const raw of stream) {
      const chunk = Buffer.from(raw.buffer, raw.byteOffset, raw.byteLength);
      let start = 0;
      while (start < chunk.length) {
        const newline = chunk.indexOf(10, start);
        const end = newline < 0 ? chunk.length : newline;
        const fragment = chunk.subarray(start, end);
        if (buffered + fragment.length > INPUT_BYTES) {
          throw invalid("Narration input line exceeds16MiB");
        }
        fragments.push(fragment);
        buffered += fragment.length;
        if (newline < 0) {
          break;
        }
        // biome-ignore lint/performance/noAwaitInLoops: Commands share retained state and must settle in input order.
        await dispatch(Buffer.concat(fragments, buffered));
        fragments = [];
        buffered = 0;
        start = newline + 1;
      }
    }
    if (buffered > 0) {
      await dispatch(Buffer.concat(fragments, buffered));
    }
  } catch (error) {
    failed = true;
    await emit({ error: narrationFailure(error) });
  }
  return failed ? 1 : 0;
};
