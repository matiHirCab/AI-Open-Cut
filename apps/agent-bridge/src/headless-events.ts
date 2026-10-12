import { z } from "zod/v4";
import { retryableFor } from "./errors";
import { type FfmpegSpawnDiagnostic, ffmpegSpawnSchema } from "./ffmpeg-spawn";

const errorSchema = z
  .object({
    code: z.string(),
    failedStage: z.string().nullable().default(null),
    ffmpegExitCode: z.number().int().nullable().default(null),
    ffmpegSpawn: ffmpegSpawnSchema.optional(),
    ffmpegStderrExcerpt: z.string().nullable().default(null),
    message: z.string(),
    retryable: z.boolean(),
  })
  .strict();

export const eventSchema = z.discriminatedUnion("type", [
  z.object({ progress: z.number(), type: z.literal("progress") }).strict(),
  z.object({ result: z.unknown(), type: z.literal("result") }).strict(),
  z.object({ error: errorSchema, type: z.literal("error") }).strict(),
]);

export class BridgeError extends Error {
  readonly code: string;
  readonly retryable: boolean;
  readonly failedStage: string | null;
  readonly ffmpegExitCode: number | null;
  readonly ffmpegSpawn: FfmpegSpawnDiagnostic | undefined;
  readonly ffmpegStderrExcerpt: string | null;

  constructor(
    code: string,
    message: string,
    _retryable = retryableFor(code),
    options?: ErrorOptions,
    details: {
      failedStage?: string | null;
      ffmpegExitCode?: number | null;
      ffmpegSpawn?: FfmpegSpawnDiagnostic | undefined;
      ffmpegStderrExcerpt?: string | null;
    } = {}
  ) {
    super(message, options);
    this.name = "BridgeError";
    this.code = code;
    this.retryable = _retryable;
    this.failedStage = details.failedStage ?? null;
    this.ffmpegExitCode = details.ffmpegExitCode ?? null;
    this.ffmpegSpawn = details.ffmpegSpawn;
    this.ffmpegStderrExcerpt = details.ffmpegStderrExcerpt ?? null;
  }
}
