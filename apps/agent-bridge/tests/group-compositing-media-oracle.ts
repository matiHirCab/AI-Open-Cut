import { spawnSync } from "node:child_process";
import { expect } from "vitest";
import { z } from "zod/v4";

export function decodedFrame(ffmpeg: string, path: string, time: number) {
  const result = spawnSync(ffmpeg, [
    "-v",
    "error",
    "-i",
    path,
    "-ss",
    String(time),
    "-frames:v",
    "1",
    "-f",
    "rawvideo",
    "-pix_fmt",
    "rgb24",
    "pipe:1",
  ]);
  expect(result.status, result.stderr.toString()).toBe(0);
  expect(result.stdout.length).toBe(64 * 64 * 3);
  return result.stdout;
}
export function independentSsim(actual: Uint8Array, expected: Uint8Array) {
  expect(actual.length).toBe(expected.length);
  const n = actual.length;
  let ma = 0,
    mb = 0,
    va = 0,
    vb = 0,
    covariance = 0;
  for (let i = 0; i < n; i += 1) {
    ma += actual[i] ?? 0;
    mb += expected[i] ?? 0;
  }
  ma /= n;
  mb /= n;
  for (let i = 0; i < n; i += 1) {
    const a = (actual[i] ?? 0) - ma,
      b = (expected[i] ?? 0) - mb;
    va += a * a;
    vb += b * b;
    covariance += a * b;
  }
  va /= n;
  vb /= n;
  covariance /= n;
  return (
    ((2 * ma * mb + 6.5025) * (2 * covariance + 58.5225)) /
    ((ma * ma + mb * mb + 6.5025) * (va + vb + 58.5225))
  );
}
const probeSchema = z.object({
  format: z.object({ duration: z.string() }),
  streams: z.array(
    z.object({
      codec_type: z.string(),
      duration: z.string(),
      height: z.number().optional(),
      nb_frames: z.string().optional(),
      start_time: z.string(),
      width: z.number().optional(),
    })
  ),
});
export function audiovisualFacts(
  ffmpeg: string,
  ffprobe: string,
  path: string,
  frames: number
) {
  const probed = spawnSync(ffprobe, [
    "-v",
    "error",
    "-show_streams",
    "-show_format",
    "-of",
    "json",
    path,
  ]);
  expect(probed.status, probed.stderr.toString()).toBe(0);
  const probe = probeSchema.parse(JSON.parse(probed.stdout.toString()));
  const video = probe.streams.find((stream) => stream.codec_type === "video");
  const audio = probe.streams.find((stream) => stream.codec_type === "audio");
  if (!(video && audio)) {
    throw new Error("Actual audiovisual streams missing");
  }
  expect(Number(video.nb_frames)).toBe(frames);
  expect(video.width).toBe(64);
  expect(video.height).toBe(64);
  expect(
    Math.abs(Number(probe.format.duration) - frames / 10)
  ).toBeLessThanOrEqual(0.1);
  expect(
    Math.abs(Number(audio.duration) - Number(video.duration))
  ).toBeLessThanOrEqual(0.1);
  expect(
    Math.abs(Number(audio.start_time) - Number(video.start_time))
  ).toBeLessThanOrEqual(0.1);
  const result = spawnSync(ffmpeg, [
    "-v",
    "error",
    "-i",
    path,
    "-map",
    "0:a:0",
    "-ac",
    "1",
    "-ar",
    "48000",
    "-f",
    "f32le",
    "pipe:1",
  ]);
  expect(result.status, result.stderr.toString()).toBe(0);
  const pcm = result.stdout;
  expect(pcm.length % 4).toBe(0);
  expect(Math.abs(pcm.length / 4 / 48_000 - frames / 10)).toBeLessThanOrEqual(
    0.1
  );
  let energy = 0;
  for (let i = 0; i < pcm.length; i += 4) {
    energy += pcm.readFloatLE(i) ** 2;
  }
  expect(Math.sqrt(energy / (pcm.length / 4))).toBeGreaterThan(0.01);
  return pcm;
}
export function alignedPcm(actual: Buffer, expected: Buffer) {
  expect(actual.length).toBe(expected.length);
  let error = 0;
  for (let i = 0; i < actual.length; i += 4) {
    error += (actual.readFloatLE(i) - expected.readFloatLE(i)) ** 2;
  }
  expect(Math.sqrt(error / (actual.length / 4))).toBeLessThanOrEqual(0.0001);
}
