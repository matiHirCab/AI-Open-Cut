import { BridgeError } from "../headless";

export class NarrationPublicationUnknownError extends BridgeError {
  readonly publicationUnknown = true;
  constructor(options: ErrorOptions) {
    super(
      "INTERNAL_ERROR",
      "Narration publication outcome is unknown; inspect/reconcile the project. These preview tokens cannot be republished.",
      false,
      options
    );
  }
}
