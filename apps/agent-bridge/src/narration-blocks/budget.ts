import { BridgeError } from "../headless";

const invalid = (message: string) =>
  new BridgeError("VALIDATION_FAILED", message);
const highSurrogate = (code: number) => code >= 0xd8_00 && code <= 0xdb_ff;
const lowSurrogate = (code: number) => code >= 0xdc_00 && code <= 0xdf_ff;
function scalarBytes(code: number) {
  if (code === 34 || code === 92) {
    return 2;
  }
  if (code < 32) {
    return [8, 9, 10, 12, 13].includes(code) ? 2 : 6;
  }
  if (code < 128) {
    return 1;
  }
  if (code < 2048) {
    return 2;
  }
  return 3;
}
class JsonBudget {
  bytes = 0;
  readonly ancestors = new WeakSet<object>();
  readonly limit: number;
  constructor(limit: number) {
    this.limit = limit;
  }
  add(count: number) {
    this.bytes += count;
    if (this.bytes > this.limit) {
      throw invalid("Narration JSON byte budget exceeded");
    }
  }
  text(input: string) {
    this.add(2);
    for (let index = 0; index < input.length; index += 1) {
      const code = input.charCodeAt(index);
      if (highSurrogate(code) && lowSurrogate(input.charCodeAt(index + 1))) {
        this.add(4);
        index += 1;
      } else if (highSurrogate(code) || lowSurrogate(code)) {
        this.add(6);
      } else {
        this.add(scalarBytes(code));
      }
    }
  }
  container(input: object, depth: number) {
    if (this.ancestors.has(input)) {
      throw invalid("Narration metadata must be acyclic JSON");
    }
    this.ancestors.add(input);
    this.add(2);
    if (Array.isArray(input)) {
      for (let index = 0; index < input.length; index += 1) {
        if (index > 0) {
          this.add(1);
        }
        this.visit(input[index] === undefined ? null : input[index], depth + 1);
      }
    } else {
      let count = 0;
      for (const [key, value] of Object.entries(input)) {
        if (value === undefined) {
          continue;
        }
        if (count > 0) {
          this.add(1);
        }
        count += 1;
        this.text(key);
        this.add(1);
        this.visit(value, depth + 1);
      }
    }
    this.ancestors.delete(input);
  }
  visit(input: unknown, depth: number) {
    if (depth > 64) {
      throw invalid("Narration JSON nesting exceeds 64");
    }
    if (input === null) {
      this.add(4);
      return;
    }
    switch (typeof input) {
      case "string":
        this.text(input);
        return;
      case "boolean":
        this.add(input ? 4 : 5);
        return;
      case "number":
        if (!Number.isFinite(input)) {
          throw invalid("Narration metadata must contain finite JSON numbers");
        }
        this.add(JSON.stringify(input).length);
        return;
      case "object":
        this.container(input, depth);
        return;
      default:
        throw invalid("Narration metadata must be acyclic JSON");
    }
  }
}
// Count exact JSON UTF8 bytes before allocating a serialized string, stopping
// as soon as the bounded payload cannot fit.
export const assertNarrationJsonBudget = (value: unknown, limit: number) => {
  const counter = new JsonBudget(limit);
  counter.visit(value, 0);
  return counter.bytes;
};
