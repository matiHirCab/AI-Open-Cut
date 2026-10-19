import { BridgeError } from "../headless";
import {
  admissionInputSchema,
  durableOutputSchemas,
  identityInputSchema,
  listInputSchema,
} from "../production-requests/contract";
import type { DurableProductionService } from "../production-requests/service";
import {
  DESTRUCTIVE,
  failure,
  READ_ONLY,
  type Server,
  type ServerDependencies,
  success,
  WRITE,
} from "./shared";

export const registerProductionRequestTools = (
  server: Server,
  { production }: ServerDependencies
) => {
  const invoke = async (
    action: (
      service: DurableProductionService
    ) => Promise<Record<string, unknown>>
  ) => {
    try {
      if (!production) {
        throw new BridgeError(
          "DEPENDENCY_UNAVAILABLE",
          "Durable production requests are disabled",
          false
        );
      }
      return success(await action(production));
    } catch (error) {
      return failure(error);
    }
  };
  server.registerTool(
    "production_request_admit",
    {
      annotations: { ...WRITE, idempotentHint: true },
      description:
        "Retain a bounded production intent under a caller UUID without dispatching it. Opt-in durable storage is required.",
      inputSchema: admissionInputSchema,
      outputSchema: durableOutputSchemas.admit,
    },
    (input) => invoke((service) => service.admit(input))
  );
  server.registerTool(
    "production_request_get",
    {
      annotations: READ_ONLY,
      description:
        "Read retained production intent and confirmed outcome. Missing inventory means unknown effects; old job or artifact tokens are never adopted.",
      inputSchema: identityInputSchema,
      outputSchema: durableOutputSchemas.get,
    },
    (input) => invoke((service) => service.get(input))
  );
  server.registerTool(
    "production_request_list",
    {
      annotations: READ_ONLY,
      description:
        "List the bounded retained production inventory and actual storage readiness without dispatching work.",
      inputSchema: listInputSchema,
      outputSchema: durableOutputSchemas.list,
    },
    (input) => invoke((service) => service.list(input))
  );
  server.registerTool(
    "production_request_start",
    {
      annotations: DESTRUCTIVE,
      description:
        "Explicitly start a retained queued production request once. Running, terminal or uncertain requests are never replayed; overwrite follows only the captured explicit intent.",
      inputSchema: identityInputSchema,
      outputSchema: durableOutputSchemas.start,
    },
    (input) => invoke((service) => service.start(input))
  );
  server.registerTool(
    "production_request_cancel_queued",
    {
      annotations: { ...WRITE, idempotentHint: true },
      description:
        "Cancel retained queued intent before production. Running work uses the separate process-local job cancellation surface.",
      inputSchema: identityInputSchema,
      outputSchema: durableOutputSchemas.cancelQueued,
    },
    (input) => invoke((service) => service.cancelQueued(input))
  );
};
