// Context-matched media references consume only frozen mathematical RGBA/PCM, never rendered outputs.
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { expect } from "vitest";
import catalog from "../../../contracts/masked-hero-reveal-v1.json";
import {
  decodedFrame,
  independentSsim,
} from "./group-compositing-media-oracle";
export const heroReferenceRoot = resolve(
  import.meta.dirname,
  "../../..",
  catalog.referenceDirectory
);
export function heroPlate(generation: string, time: number) {
  return readFileSync(
    join(
      heroReferenceRoot,
      `${generation}-${String(time).padStart(4, "0")}.rgba`
    )
  );
}
const graph =
  "[0:v]format=yuv420p[base0];[1:v]fps=10,settb=AVTB,setpts=PTS-STARTPTS+0/TB,format=rgba[plate];[base0][plate]overlay=format=auto:x=0:y=0:eof_action=pass[composed];[composed]scale=64:64:force_original_aspect_ratio=decrease,pad=64:64:(ow-iw)/2:(oh-ih)/2,format=yuv420p[video]";
const conversionCache = new Map<string, Buffer>();
export function convertedHero(
  ffmpeg: string,
  root: string,
  generation: string,
  time: number
) {
  const key = JSON.stringify([ffmpeg, root, generation, time]);
  const cached = conversionCache.get(key);
  if (cached) {
    return cached;
  }
  const pam = join(root, "independent-hero.pam"),
    png = join(root, "independent-hero.png");
  writeFileSync(
    pam,
    Buffer.concat([
      Buffer.from(
        "P7\nWIDTH 64\nHEIGHT 64\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n"
      ),
      heroPlate(generation, time),
    ])
  );
  const result = spawnSync(ffmpeg, [
    "-v",
    "error",
    "-nostdin",
    "-f",
    "lavfi",
    "-i",
    "color=c=black:s=64x64:r=10:d=1",
    "-loop",
    "1",
    "-i",
    pam,
    "-filter_complex_threads",
    "1",
    "-filter_complex",
    graph,
    "-map",
    "[video]",
    "-frames:v",
    "1",
    "-y",
    png,
  ]);
  expect(result.status, result.stderr.toString()).toBe(0);
  const pixels = decodedFrame(ffmpeg, png, 0);
  conversionCache.set(key, pixels);
  return pixels;
}
export function heroBytes(
  actual: Uint8Array,
  expected: Uint8Array,
  label: string
) {
  expect(actual.length, label).toBe(expected.length);
  let maximum = 0;
  for (let i = 0; i < actual.length; i += 1) {
    maximum = Math.max(
      maximum,
      Math.abs((actual[i] ?? 0) - (expected[i] ?? Number.NaN))
    );
  }
  expect(maximum, label).toBeLessThanOrEqual(1);
}
export function decodedStereo(ffmpeg: string, path: string) {
  const result = spawnSync(
    ffmpeg,
    [
      "-v",
      "error",
      "-i",
      path,
      "-map",
      "0:a:0",
      "-ar",
      "48000",
      "-ac",
      "2",
      "-f",
      "f32le",
      "pipe:1",
    ],
    { maxBuffer: 1024 * 1024 }
  );
  expect(result.status, result.stderr.toString()).toBe(0);
  expect(result.stdout.length % 8).toBe(0);
  return result.stdout;
}
export function referenceHeroAudio(
  ffmpeg: string,
  root: string,
  start: number,
  restart = false
) {
  const path = join(root, `independent-audio-${start}-${restart}.m4a`);
  if (existsSync(path)) {
    return decodedStereo(ffmpeg, path);
  }
  const result = spawnSync(ffmpeg, [
    "-v",
    "error",
    "-nostdin",
    "-i",
    join(heroReferenceRoot, "source.wav"),
    "-ss",
    String(restart ? 0 : start / 1000),
    "-t",
    String((800 - start) / 1000),
    "-c:a",
    "aac",
    "-y",
    path,
  ]);
  expect(result.status, result.stderr.toString()).toBe(0);
  return decodedStereo(ffmpeg, path);
}
export function stereoRms(actual: Buffer, expected: Buffer) {
  expect(actual.length % 8).toBe(0);
  expect(expected.length % 8).toBe(0);
  const a = actual.length / 8,
    b = expected.length / 8,
    shorter = Math.min(a, b);
  expect(Math.abs(a - b)).toBeLessThanOrEqual(4800);
  expect(shorter).toBeGreaterThan(4800);
  for (const pcm of [actual, expected]) {
    let finite = true;
    for (let i = 0; i < pcm.length; i += 4) {
      finite = finite && Number.isFinite(pcm.readFloatLE(i));
    }
    expect(finite, "decoded stereo PCM must be finite").toBe(true);
  }
  const score = (offset: number) => {
    const ai = Math.max(offset, 0),
      bi = Math.max(-offset, 0),
      length = Math.min(a - ai, b - bi);
    expect(length).toBeGreaterThanOrEqual(shorter - 4800);
    let left = 0,
      right = 0;
    for (let i = 0; i < length; i += 1) {
      left +=
        (actual.readFloatLE((i + ai) * 8) -
          expected.readFloatLE((i + bi) * 8)) **
        2;
      right +=
        (actual.readFloatLE((i + ai) * 8 + 4) -
          expected.readFloatLE((i + bi) * 8 + 4)) **
        2;
    }
    return [
      Math.sqrt(left / length),
      Math.sqrt(right / length),
      Math.sqrt((left + right) / (2 * length)),
    ];
  };
  let best = score(0);
  if (best.every((v) => v <= 0.0001)) {
    return best;
  }
  for (let offset = -4800; offset <= 4800; offset += 1) {
    const value = score(offset);
    if (
      (value[2] ?? Number.POSITIVE_INFINITY) <
      (best[2] ?? Number.POSITIVE_INFINITY)
    ) {
      best = value;
    }
    if (value.every((v) => v <= 0.0001)) {
      return value;
    }
  }
  return best;
}
export function heroMovie(
  ffmpeg: string,
  ffprobe: string,
  root: string,
  path: string,
  generation: string,
  start: number
) {
  const result = spawnSync(ffprobe, [
    "-v",
    "error",
    "-show_streams",
    "-show_format",
    "-of",
    "json",
    path,
  ]);
  expect(result.status, result.stderr.toString()).toBe(0);
  const probe = JSON.parse(result.stdout.toString()) as {
    format: { duration: string };
    streams: {
      codec_type: string;
      codec_name: string;
      width?: number;
      height?: number;
      nb_frames?: string;
      sample_rate?: string;
      channels?: number;
      duration: string;
      start_time: string;
      r_frame_rate?: string;
    }[];
  };
  const video = probe.streams.find((s) => s.codec_type === "video"),
    audio = probe.streams.find((s) => s.codec_type === "audio");
  if (!(video && audio)) {
    throw new Error("actual hero audiovisual streams missing");
  }
  expect(video.codec_name).toBe("h264");
  expect(audio.codec_name).toBe("aac");
  expect(video.width).toBe(64);
  expect(video.height).toBe(64);
  expect(video.r_frame_rate).toBe("10/1");
  expect(Number(video.nb_frames)).toBe((800 - start) / 100);
  expect(audio.sample_rate).toBe("48000");
  expect(audio.channels).toBe(2);
  expect(
    Math.abs(Number(probe.format.duration) - (800 - start) / 1000)
  ).toBeLessThanOrEqual(0.1);
  expect(
    Math.abs(Number(video.duration) - Number(audio.duration))
  ).toBeLessThanOrEqual(0.1);
  expect(
    Math.abs(Number(video.start_time) - Number(audio.start_time))
  ).toBeLessThanOrEqual(0.1);
  const context = independentHeroFilm(ffmpeg, root, generation, start);
  for (let t = start; t < 800; t += 100) {
    expect(
      independentSsim(
        heroDecodedFrame(ffmpeg, path, (t - start) / 1000),
        heroDecodedFrame(ffmpeg, context, (t - start) / 1000)
      ),
      "independent encoded context"
    ).toBeGreaterThanOrEqual(0.99);
    expect(
      independentSsim(
        heroDecodedFrame(ffmpeg, path, (t - start) / 1000),
        convertedHero(ffmpeg, root, generation, t)
      ),
      `${generation}/${start}/${t}`
    ).toBeGreaterThanOrEqual(0.99);
  }
  const pcm = decodedStereo(ffmpeg, path);
  for (const channel of [0, 1]) {
    let energy = 0;
    for (let i = channel * 4; i < pcm.length; i += 8) {
      energy += pcm.readFloatLE(i) ** 2;
    }
    expect(Math.sqrt(energy / (pcm.length / 8))).toBeGreaterThan(0.01);
  }
  expect(
    Math.abs(pcm.length / 8 / 48_000 - (800 - start) / 1000)
  ).toBeLessThanOrEqual(0.1);
  for (const rms of stereoRms(pcm, referenceHeroAudio(ffmpeg, root, start))) {
    expect(rms).toBeLessThanOrEqual(0.0001);
  }
  return pcm;
}
function independentHeroFilm(
  ffmpeg: string,
  root: string,
  generation: string,
  start: number
) {
  const manifest = JSON.parse(
    readFileSync(
      join(heroReferenceRoot, "reference-film-provenance.json"),
      "utf8"
    )
  ) as { records: { generation: string; intent: string; command: string[] }[] };
  const intent = start === 0 ? "full" : "partial",
    record = manifest.records.find(
      (r) => r.generation === generation && r.intent === intent
    );
  if (!record) {
    throw new Error("frozen independent film context missing");
  }
  const raw = join(root, `independent-${generation}-full.rgba`),
    output = join(root, `independent-${generation}-${intent}.mp4`);
  if (existsSync(output)) {
    return output;
  }
  writeFileSync(
    raw,
    Buffer.concat(catalog.plates.samples.map((t) => heroPlate(generation, t)))
  );
  const args = record.command.slice(1).map((arg) => {
    if (arg.endsWith("/source.wav")) {
      return join(heroReferenceRoot, "source.wav");
    }
    if (arg.endsWith(`/${generation}-full.rgba`)) {
      return raw;
    }
    return arg;
  });
  args[args.length - 1] = output;
  const result = spawnSync(ffmpeg, args);
  expect(result.status, result.stderr.toString()).toBe(0);
  return output;
}

const videoCache = new Map<string, Buffer>();
export function decodedHeroVideo(ffmpeg: string, path: string) {
  const facts = statSync(path, { bigint: true });
  const key = JSON.stringify([
    ffmpeg,
    path,
    String(facts.mtimeNs),
    String(facts.size),
  ]);
  const cached = videoCache.get(key);
  if (cached) {
    return cached;
  }
  const result = spawnSync(
    ffmpeg,
    [
      "-v",
      "error",
      "-i",
      path,
      "-map",
      "0:v:0",
      "-f",
      "rawvideo",
      "-pix_fmt",
      "rgb24",
      "pipe:1",
    ],
    { maxBuffer: 1024 * 1024 }
  );
  expect(result.status, result.stderr.toString()).toBe(0);
  videoCache.set(key, result.stdout);
  return result.stdout;
}

export function heroDecodedFrame(ffmpeg: string, path: string, time: number) {
  const frames = decodedHeroVideo(ffmpeg, path);
  const size = 64 * 64 * 3,
    offset = Math.round(time * 10) * size;
  const pixels = frames.subarray(offset, offset + size);
  expect(pixels.length).toBe(size);
  return pixels;
}
