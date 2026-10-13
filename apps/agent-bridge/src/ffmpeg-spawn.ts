import { z } from "zod/v4";

export const FFMPEG_SPAWN_KINDS = [
  "not_found",
  "permission_denied",
  "invalid_input",
  "invalid_filename",
  "argument_list_too_long",
  "resource_busy",
  "out_of_memory",
  "other",
] as const;

export const FFMPEG_SPAWN_PLATFORMS = [
  "linux",
  "macos",
  "windows",
  "other",
] as const;

export const ffmpegSpawnSchema = z.strictObject({
  kind: z.enum(FFMPEG_SPAWN_KINDS),
  nativeCode: z.int().min(-2_147_483_648).max(2_147_483_647).nullable(),
  platform: z.enum(FFMPEG_SPAWN_PLATFORMS),
});

export type FfmpegSpawnDiagnostic = z.infer<typeof ffmpegSpawnSchema>;
