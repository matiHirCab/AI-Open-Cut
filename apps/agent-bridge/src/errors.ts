import catalog from "../../../contracts/error-codes-v1.json";

export type ErrorCode = keyof typeof catalog.codes;

export const errorDefinition = (code: string) =>
  Object.hasOwn(catalog.codes, code)
    ? catalog.codes[code as ErrorCode]
    : undefined;

export const retryableFor = (code: string) =>
  errorDefinition(code)?.retryable ?? false;

export const publicDescriptionFor = (code: string) =>
  errorDefinition(code)?.description ?? "provider request failed";

const PROVIDER_CODES = new Set<ErrorCode>([
  "INVALID_ARGUMENT",
  "PATH_NOT_ALLOWED",
  "TTS_INVALID_OUTPUT",
  "TTS_SYNTHESIS_FAILED",
  "TTS_UNAVAILABLE",
]);

export const normalizeProviderErrorCode = (code: string): ErrorCode =>
  PROVIDER_CODES.has(code as ErrorCode)
    ? (code as ErrorCode)
    : "TTS_PROVIDER_FAILED";
