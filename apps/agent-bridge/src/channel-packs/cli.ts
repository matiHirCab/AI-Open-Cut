import { BridgeError, type HeadlessClient } from "../headless";
import {
  CHANNEL_CLI_BYTES,
  CHANNEL_PACKAGE_BYTES,
  channelApplyResultSchema,
  channelCaptureResultSchema,
  channelListSchema,
  channelPackSchema,
  channelRequestSchema,
} from "./contract";

const OPERATIONS = {
  apply: "channel_pack_apply",
  capture: "channel_pack_capture",
  export: "channel_pack_get",
  get: "channel_pack_get",
  import: "channel_pack_import",
  list: "channel_pack_list",
  update: "channel_pack_update",
} as const;

const invalid = (message: string, cause?: unknown) =>
  new BridgeError("INVALID_ARGUMENT", message, false, { cause });

const commandFields = (raw: string): Record<string, unknown> => {
  try {
    const input: unknown = raw.trim() ? JSON.parse(raw) : {};
    if (
      input === null ||
      typeof input !== "object" ||
      Array.isArray(input) ||
      Object.hasOwn(input, "operation")
    ) {
      throw invalid("Provide command fields without an operation tag");
    }
    return input as Record<string, unknown>;
  } catch (cause) {
    throw invalid(
      "Channel command input must be a JSON object without an operation tag",
      cause
    );
  }
};

export const channelFailure = (error: unknown) => ({
  error:
    error instanceof BridgeError
      ? {
          code: error.code,
          message: error.message,
          retryable: error.retryable,
        }
      : {
          code: "INTERNAL_ERROR",
          message:
            "Channel pack command failed; inspect the current pack/project version before retrying",
          retryable: false,
        },
});

export const readChannelInput = async (
  stream: AsyncIterable<Uint8Array>
): Promise<string> => {
  const chunks: Uint8Array[] = [];
  let bytes = 0;
  for await (const chunk of stream) {
    bytes += chunk.byteLength;
    if (bytes > CHANNEL_CLI_BYTES) {
      throw invalid("Channel CLI input exceeds 9 MiB");
    }
    chunks.push(chunk);
  }
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(
      Buffer.concat(chunks)
    );
  } catch (cause) {
    throw invalid("Channel CLI input must be valid UTF-8", cause);
  }
};

export const runChannelCli = async (
  args: readonly string[],
  stream: AsyncIterable<Uint8Array>,
  headless: Pick<HeadlessClient, "call">,
  output: (value: string) => Promise<void>,
  errors: (value: string) => Promise<void>
): Promise<number> => {
  try {
    const [command, name, ...extra] = args;
    if (
      !(command && Object.hasOwn(OPERATIONS, command)) ||
      extra.length > 0 ||
      (command === "import" ? name === undefined : name !== undefined)
    ) {
      throw invalid(
        "Use list/get/capture/update/import/export/apply; import requires an explicit name argument"
      );
    }
    const raw = await readChannelInput(stream);
    let fields: Record<string, unknown>;
    if (command === "import") {
      if (Buffer.byteLength(raw, "utf8") > CHANNEL_PACKAGE_BYTES) {
        throw invalid("Channel package exceeds 8 MiB");
      }
      fields = { name, packageJson: raw };
    } else {
      fields = commandFields(raw);
    }
    const parsed = channelRequestSchema.safeParse({
      ...fields,
      operation: OPERATIONS[command as keyof typeof OPERATIONS],
    });
    if (!parsed.success) {
      throw invalid("Invalid closed channel command fields");
    }
    const request = parsed.data;
    if (
      Buffer.byteLength(JSON.stringify(request), "utf8") > CHANNEL_CLI_BYTES
    ) {
      throw invalid("Serialized channel request exceeds 9 MiB");
    }
    let result: unknown;
    switch (request.operation) {
      case "channel_pack_list":
        result = await headless.call(request, channelListSchema);
        break;
      case "channel_pack_get": {
        const saved = await headless.call(request, channelPackSchema);
        result = command === "export" ? saved.package : saved;
        break;
      }
      case "channel_pack_capture":
      case "channel_pack_update":
        result = await headless.call(request, channelCaptureResultSchema);
        break;
      case "channel_pack_import":
        result = await headless.call(request, channelPackSchema);
        break;
      case "channel_pack_apply":
        result = await headless.call(request, channelApplyResultSchema);
        break;
      default:
        throw invalid("Unknown channel operation");
    }
    const serialized = `${JSON.stringify(result)}\n`;
    if (Buffer.byteLength(serialized, "utf8") > CHANNEL_CLI_BYTES) {
      throw invalid(
        "Channel output exceeds 9 MiB; native effects may have completed, inspect current versions before retrying"
      );
    }
    await output(serialized);
    return 0;
  } catch (error) {
    await errors(`${JSON.stringify(channelFailure(error))}\n`).catch(() => {
      // A closed error stream still produces an unsuccessful process status.
    });
    return 1;
  }
};
