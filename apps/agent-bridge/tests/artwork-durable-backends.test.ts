import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, expect, it } from "vitest";
import { artworkDurableBackends } from "./artwork-durable-workflow";
import { syntheticNarrationWav } from "./fixtures/narration-blocks-provider";

const roots: string[] = [];
afterEach(() => {
  for (const root of roots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});
const png = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAMAAAADCAYAAABWKLW/AAAAFElEQVR4nGP4jwQYQMQdETkEBwYAlZkh7kUp3TIAAAAASUVORK5CYII=",
  "base64"
);
const digest = (value: Buffer) =>
  createHash("sha256").update(value).digest("hex");
function setup() {
  const root = mkdtempSync(join(tmpdir(), "artwork-probe-unit-"));
  roots.push(root);
  const backend = process.execPath;
  const manifest = join(root, "allowlist.json"),
    image = join(root, "original.png"),
    wav = join(root, "owned.wav");
  writeFileSync(image, png);
  writeFileSync(wav, syntheticNarrationWav(400, 220));
  const hashes: Record<string, string> = {};
  const allow = (...paths: string[]) => {
    for (const path of paths) {
      hashes[path] = digest(readFileSync(path));
    }
    writeFileSync(manifest, JSON.stringify({ backend, hashes, root }));
  };
  allow(image, wav);
  const probe = resolve(
    import.meta.dirname,
    "fixtures/artwork-durable-probe.mjs"
  );
  const run = (path: string) =>
    spawnSync(process.execPath, [probe, manifest, "-v", "error", path], {
      encoding: "utf8",
      env: { ...process.env, PATH: "" },
    });
  return { allow, backend, hashes, image, manifest, probe, root, run, wav };
}
it("uses only supplied backends and requires explicit valid genuine configuration", () => {
  const f = setup(),
    base = { OPENCUT_FFMPEG_PATH: f.backend, OPENCUT_FFPROBE_PATH: f.backend };
  expect(artworkDurableBackends(base, {})).toEqual({
    ffmpeg: f.backend,
    ffprobe: f.backend,
    genuine: false,
  });
  expect(() => artworkDurableBackends({}, {})).toThrow();
  expect(() =>
    artworkDurableBackends(base, { OPENCUT_ARTWORK_DURABLE_GENUINE: "1" })
  ).toThrow();
  expect(
    artworkDurableBackends(
      {},
      { ...base, OPENCUT_ARTWORK_DURABLE_GENUINE: "1" }
    )
  ).toMatchObject({ genuine: true });
  for (const value of ["0", "true", ""]) {
    expect(() =>
      artworkDurableBackends(base, { OPENCUT_ARTWORK_DURABLE_GENUINE: value })
    ).toThrow();
  }
  for (const path of ["ffprobe", join(f.root, "missing")]) {
    expect(() =>
      artworkDurableBackends({ ...base, OPENCUT_FFPROBE_PATH: path }, {})
    ).toThrow();
  }
});
it("probes actual owned headers with empty PATH and delegates configured version checks", () => {
  const f = setup();
  const image = f.run(f.image),
    wav = f.run(f.wav);
  expect(image.status, image.stderr).toBe(0);
  expect(JSON.parse(image.stdout)).toMatchObject({
    frames: [{ height: 3, width: 3 }],
    streams: [{ codec_name: "png", height: 3, width: 3 }],
  });
  expect(wav.status, wav.stderr).toBe(0);
  expect(JSON.parse(wav.stdout)).toMatchObject({
    format: { duration: "0.4" },
    streams: [{ channels: 1, sample_rate: "24000" }],
  });
  const direct = spawnSync(f.backend, ["-version"], {
    encoding: "utf8",
    env: { ...process.env, PATH: "" },
  });
  const delegated = spawnSync(
    process.execPath,
    [f.probe, f.manifest, "-version"],
    { encoding: "utf8", env: { ...process.env, PATH: "" } }
  );
  expect(delegated.status).toBe(direct.status);
  expect(delegated.stdout).toBe(direct.stdout);
  expect(delegated.stderr).toBe(direct.stderr);
});
it("refuses unknown changed malformed oversized and out-of-root fixture inputs", () => {
  const f = setup(),
    unknown = join(f.root, "unknown.png");
  writeFileSync(unknown, png);
  expect(f.run(unknown).status).not.toBe(0);
  writeFileSync(f.image, Buffer.concat([png, Buffer.from([1])]));
  expect(f.run(f.image).status).not.toBe(0);
  const bad = Buffer.from(png);
  bad.writeUInt32BE(0, 16);
  writeFileSync(f.image, bad);
  f.allow(f.image);
  expect(f.run(f.image).status).not.toBe(0);
  const corrupt = Buffer.from(png);
  corrupt[corrupt.length - 15] = 99;
  writeFileSync(f.image, corrupt);
  f.allow(f.image);
  expect(f.run(f.image).status).not.toBe(0);
  const badWav = syntheticNarrationWav(400, 220);
  badWav.writeUInt16LE(3, 32);
  writeFileSync(f.wav, badWav);
  f.allow(f.wav);
  expect(f.run(f.wav).status).not.toBe(0);
  writeFileSync(unknown, Buffer.alloc(8_388_609));
  expect(f.run(unknown).status).not.toBe(0);
  const outside = setup();
  f.hashes[outside.image] = digest(png);
  f.allow();
  expect(f.run(outside.image).status).not.toBe(0);
  if (process.platform !== "win32") {
    const link = join(f.root, "link.png");
    symlinkSync(outside.image, link);
    f.hashes[link] = digest(png);
    f.allow();
    expect(f.run(link).status).not.toBe(0);
  }
  expect(readFileSync(outside.image)).toEqual(png);
});
