import { expect, it } from "vitest";

import { JsonLineLogger } from "../src/logger";

it("writes valid filtered JSONL with correlation and timing fields", () => {
  const lines: string[] = [];
  const logger = new JsonLineLogger("debug", (line) => lines.push(line));
  logger.info("speech.synthesis.completed", {
    characters: 42,
    chunks: 2,
    durationMs: 125,
    jobId: "job-1",
    path: "C:/private/speech.wav",
    providerId: "kokoro",
    providerStderr: "secret provider output",
    requestId: "request-1",
    text: "private speech text",
    token: "opaque-secret-token",
  } as never);
  expect(lines).toHaveLength(1);
  const event = JSON.parse(lines[0] ?? "") as Record<string, unknown>;
  expect(event).toMatchObject({
    characters: 42,
    chunks: 2,
    durationMs: 125,
    event: "speech.synthesis.completed",
    jobId: "job-1",
    level: "info",
    providerId: "kokoro",
    requestId: "request-1",
  });
  expect(event.timestamp).toEqual(expect.any(String));
  expect(JSON.stringify(event)).not.toContain("private");
  expect(JSON.stringify(event)).not.toContain("opaque-secret-token");
  expect(JSON.stringify(event)).not.toContain("provider output");
});

it("filters entries below the configured log level", () => {
  const lines: string[] = [];
  const logger = new JsonLineLogger("warn", (line) => lines.push(line));
  logger.debug("debug");
  logger.info("info");
  logger.warn("warn");
  logger.error("error");
  expect(lines.map((line) => JSON.parse(line).level)).toEqual([
    "warn",
    "error",
  ]);
});
