import { BridgeError } from "../headless-events";

const confirmed = new WeakSet<BridgeError>();

/** Mark only an error constructed from a validated native terminal error event. */
export const confirmedCoreError = (error: BridgeError) => {
  confirmed.add(error);
  return error;
};

export const isConfirmedCoreError = (error: unknown): error is BridgeError =>
  error instanceof BridgeError && confirmed.has(error);
