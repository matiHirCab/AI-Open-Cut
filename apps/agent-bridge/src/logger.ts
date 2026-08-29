export type LogLevel = "error" | "warn" | "info" | "debug";

type LogValue = boolean | number | string | null | undefined;
type LogFields = Record<string, LogValue>;

export interface Logger {
  debug: (event: string, fields?: LogFields) => void;
  error: (event: string, fields?: LogFields) => void;
  info: (event: string, fields?: LogFields) => void;
  warn: (event: string, fields?: LogFields) => void;
}

const LEVELS: Record<LogLevel, number> = {
  debug: 10,
  error: 40,
  info: 20,
  warn: 30,
};
const SAFE_FIELDS = new Set([
  "characters",
  "chunks",
  "cleanupOutcome",
  "code",
  "durationMs",
  "jobId",
  "operation",
  "providerId",
  "queueWaitMs",
  "requestId",
  "status",
  "synthesisDurationMs",
]);

export class JsonLineLogger implements Logger {
  readonly #level: LogLevel;
  readonly #write: (line: string) => void;

  constructor(
    level: LogLevel,
    write: (line: string) => void = (line) => process.stderr.write(line)
  ) {
    this.#level = level;
    this.#write = write;
  }

  debug(event: string, fields?: LogFields) {
    this.#log("debug", event, fields);
  }
  info(event: string, fields?: LogFields) {
    this.#log("info", event, fields);
  }
  warn(event: string, fields?: LogFields) {
    this.#log("warn", event, fields);
  }
  error(event: string, fields?: LogFields) {
    this.#log("error", event, fields);
  }

  #log(level: LogLevel, event: string, fields: LogFields = {}) {
    if (LEVELS[level] < LEVELS[this.#level]) {
      return;
    }
    const safe = Object.fromEntries(
      Object.entries(fields).filter(
        ([key, value]) => SAFE_FIELDS.has(key) && value !== undefined
      )
    );
    this.#write(
      `${JSON.stringify({ ...safe, event, level, timestamp: new Date().toISOString() })}\n`
    );
  }
}

export const NOOP_LOGGER: Logger = {
  debug: () => undefined,
  error: () => undefined,
  info: () => undefined,
  warn: () => undefined,
};
