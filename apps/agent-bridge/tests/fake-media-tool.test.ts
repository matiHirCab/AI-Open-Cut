import { spawn, spawnSync } from "node:child_process";
import { mkdtempSync, readdirSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve as resolvePath } from "node:path";
import { afterEach, expect, it } from "vitest";

const fixture = resolvePath(
  import.meta.dirname,
  "fixtures/fake-media-tool.mjs"
);
const roots: string[] = [];
afterEach(() => {
  for (const root of roots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});
const run = (args: string[], input?: Buffer, mode = "ffmpeg") => {
  const root = mkdtempSync(join(tmpdir(), "fake-media-port-"));
  roots.push(root);
  const result = spawnSync(process.execPath, [fixture, mode, ...args], {
    cwd: root,
    input,
    maxBuffer: 67_108_864 + 1024,
  });
  return { root, ...result };
};
const decoder = (size: string) => [
  "-v",
  "error",
  "-nostdin",
  "-ss",
  "0.050",
  "-i",
  "fixture.mp4",
  "-frames:v",
  "1",
  "-vf",
  `scale=${size},format=rgba`,
  "-f",
  "rawvideo",
  "pipe:1",
];
const raster = (source: string) => [
  "-v",
  "error",
  "-nostdin",
  "-f",
  "lavfi",
  "-i",
  source,
  "-frames:v",
  "1",
  "-threads",
  "1",
  "-pix_fmt",
  "rgba",
  "-f",
  "rawvideo",
  "pipe:1",
];

it("emits exact opaque RGBA decoder bytes without progress or artifacts", () => {
  const result = run(decoder("10:2"));
  expect(result.error).toBeUndefined();
  expect(result.status).toBe(0);
  expect(result.stderr.length).toBe(0);
  expect(result.stdout).toEqual(
    Buffer.alloc(80, Buffer.from([255, 0, 0, 255]))
  );
  expect(readdirSync(result.root)).toEqual([]);
});

it.each(["black@0", "0x203060@0.75"])(
  "derives dimensions from the canonical %s Caption source",
  (color) => {
    const result = run(
      raster(
        `color=c=${color}:s=2x3:r=1:d=1,format=rgba,drawtext=text='literal'`
      )
    );
    expect(result.status).toBe(0);
    expect(result.stdout).toEqual(
      Buffer.alloc(24, Buffer.from([255, 0, 0, 255]))
    );
    expect(readdirSync(result.root)).toEqual([]);
  }
);

it.each(["16384:1", "4096:4096"])(
  "accepts genuine dimension/pixel boundary %s",
  (size) => {
    const [width, height] = size.split(":").map(Number);
    const result = run(decoder(size));
    expect(result.error).toBeUndefined();
    expect(result.status).toBe(0);
    expect(result.stdout.length).toBe((width ?? 0) * (height ?? 0) * 4);
    expect(result.stdout.subarray(0, 4)).toEqual(Buffer.from([255, 0, 0, 255]));
    expect(result.stdout.subarray(-4)).toEqual(Buffer.from([255, 0, 0, 255]));
    expect(readdirSync(result.root)).toEqual([]);
  }
);

it.each([
  "0:1",
  "1:0",
  "-1:1",
  "1.5:2",
  "1e3:1",
  "16385:1",
  "8192:4096",
  "999999999999999999999:1",
])(
  "rejects malformed or over-budget decoder size %s before allocation/output",
  (size) => {
    const result = run(decoder(size));
    expect(result.status).not.toBe(0);
    expect(result.stdout.length).toBe(0);
    expect(result.stderr.toString()).toContain(
      "invalid fixture RGBA dimensions"
    );
    expect(readdirSync(result.root)).toEqual([]);
  }
);

it.each(
  [
    ["-f", "rawvideo", "pipe:1"],
    ["-vf", "scale=1:1,format=rgb24", "-f", "rawvideo", "pipe:1"],
    raster("color=c=black:s=2x3:r=30:d=1,format=rgba"),
    raster("color=c=black:s=2x3:r=1:d=1,format=rgb24"),
  ].map((args) => ({ args }))
)("rejects missing/noncanonical raw port arguments %j", ({ args }) => {
  const result = run(args);
  expect(result.status).not.toBe(0);
  expect(result.stdout.length).toBe(0);
  expect(readdirSync(result.root)).toEqual([]);
});

it("preserves version, probe geometry/audio selection and filter capabilities", () => {
  expect(run(["-version"], undefined, "ffprobe").stdout.toString()).toBe(
    "ffprobe fake 1.0\n"
  );
  const probe = JSON.parse(
    run(["fixture.mp4"], undefined, "ffprobe").stdout.toString()
  );
  expect(probe.streams).toEqual([
    { codec_name: "rawvideo", codec_type: "video", height: 1, width: 1 },
    {
      channels: 1,
      codec_name: "pcm_s16le",
      codec_type: "audio",
      sample_rate: "24000",
    },
  ]);
  const selected = JSON.parse(
    run(
      ["-select_streams", "v:0", "fixture.mp4"],
      undefined,
      "ffprobe"
    ).stdout.toString()
  );
  expect(selected.streams).toEqual([probe.streams[0]]);
  expect(run(["-filters"]).stdout.toString()).toBe(
    " ... overlay ... drawtext ... amix ... remap ... blend ... nullsrc ... split ... geq ... pad ... crop ... format ... \n"
  );
});

it("preserves normal artifact/progress handling and ignores pipe:0 outside input", () => {
  const result = run([
    "-metadata",
    "comment=pipe:0",
    "-filter_complex_script",
    "fixture.filter",
    "-frames:v",
    "1",
    "-map",
    "[video]",
    "preview.png",
    "-map",
    "[audio]",
    "-f",
    "null",
    "NUL",
  ]);
  expect(result.status).toBe(0);
  expect(readFileSync(join(result.root, "preview.png"), "utf8")).toBe(
    "fake rendered media"
  );
  expect(result.stdout.toString()).toBe("out_time_ms=100000\nprogress=end\n");
  expect(readdirSync(result.root)).toEqual(["preview.png"]);
});

it("stream-discards a complete large PAM input before unchanged artifact/progress output", async () => {
  const header = Buffer.from(
    "P7\nWIDTH 512\nHEIGHT 512\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n"
  );
  const pam = Buffer.concat([header, Buffer.alloc(512 * 512 * 4, 255)]);
  expect(pam.length).toBeGreaterThan(1_048_576);
  const root = mkdtempSync(join(tmpdir(), "fake-media-stream-"));
  roots.push(root);
  const child = spawn(
    process.execPath,
    [
      fixture,
      "ffmpeg",
      ...[
        "-v",
        "error",
        "-nostdin",
        "-f",
        "image2pipe",
        "-vcodec",
        "pam",
        "-r",
        "30",
        "-i",
        "pipe:0",
        "-frames:v",
        "1",
        "-c:v",
        "ffv1",
        "-pix_fmt",
        "rgba",
        "-y",
        "sample.mkv",
      ],
    ],
    { cwd: root, stdio: ["pipe", "pipe", "pipe"] }
  );
  const stdout: Buffer[] = [];
  const stderr: Buffer[] = [];
  child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
  child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
  const completion = new Promise<number | null>((resolve, reject) => {
    child.once("error", reject);
    child.once("close", resolve);
  });
  let acceptedBytes = 0;
  let finished = false;
  await new Promise<void>((resolve, reject) => {
    child.stdin.once("error", reject);
    child.stdin.write(pam, (error) => {
      if (error) {
        reject(error);
        return;
      }
      acceptedBytes = pam.length;
      child.stdin.end(() => {
        finished = true;
        resolve();
      });
    });
  });
  expect(acceptedBytes).toBe(pam.length);
  expect(finished).toBe(true);
  expect(await completion).toBe(0);
  expect(Buffer.concat(stderr).length).toBe(0);
  expect(readFileSync(join(root, "sample.mkv"), "utf8")).toBe(
    "fake rendered media"
  );
  expect(Buffer.concat(stdout).toString()).toBe(
    "out_time_ms=100000\nprogress=end\n"
  );
  expect(readdirSync(root)).toEqual(["sample.mkv"]);
});
