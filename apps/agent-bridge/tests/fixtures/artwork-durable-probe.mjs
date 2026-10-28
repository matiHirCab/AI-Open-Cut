/** Private owned-file probe for the combined hermetic acceptance, never a provider. */
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { isAbsolute, relative, resolve, sep } from "node:path";
import { crc32, inflateSync } from "node:zlib";

const [manifestPath, ...args] = process.argv.slice(2);
const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
if (args.length === 1 && args[0] === "-version") {
  if (!isAbsolute(manifest.backend)) {
    throw new Error("Explicit probe backend required");
  }
  const result = spawnSync(manifest.backend, args, { stdio: "inherit" });
  if (result.error) {
    throw result.error;
  }
  process.exit(result.status ?? 1);
}
const path = args.at(-1);
if (!(path && isAbsolute(path))) {
  throw new Error("Owned probe input required");
}
const root = realpathSync(manifest.root);
const owned = relative(root, path);
if (
  !owned ||
  owned === ".." ||
  owned.startsWith(`..${sep}`) ||
  isAbsolute(owned) ||
  realpathSync(path) !== resolve(path)
) {
  throw new Error("Probe input is outside ordinary owned files");
}
const stat = lstatSync(path);
if (!stat.isFile() || stat.size < 44 || stat.size > 8_388_608) {
  throw new Error("Probe input bounds/type invalid");
}
const bytes = readFileSync(path);
if (
  createHash("sha256").update(bytes).digest("hex") !== manifest.hashes[path]
) {
  throw new Error("Unknown or changed probe fixture");
}
let result;
if (bytes.subarray(0, 8).equals(Buffer.from("89504e470d0a1a0a", "hex"))) {
  const width = bytes.readUInt32BE(16),
    height = bytes.readUInt32BE(20);
  if (
    bytes.readUInt32BE(8) !== 13 ||
    bytes.toString("ascii", 12, 16) !== "IHDR" ||
    bytes[24] !== 8 ||
    bytes[25] !== 6 ||
    bytes[26] !== 0 ||
    bytes[27] !== 0 ||
    bytes[28] !== 0 ||
    width < 1 ||
    height < 1 ||
    width > 544 ||
    height > 544 ||
    bytes.toString("ascii", bytes.length - 8, bytes.length - 4) !== "IEND"
  ) {
    throw new Error("Malformed fixture PNG");
  }
  const compressed = [];
  let position = 8;
  let ended = false;
  while (position < bytes.length) {
    if (position + 12 > bytes.length || ended) {
      throw new Error("Malformed PNG chunk bounds");
    }
    const length = bytes.readUInt32BE(position),
      end = position + length + 12;
    if (end > bytes.length) {
      throw new Error("Malformed PNG chunk bounds");
    }
    if (
      crc32(bytes.subarray(position + 4, end - 4)) !==
      bytes.readUInt32BE(end - 4)
    ) {
      throw new Error("Malformed PNG checksum");
    }
    const kind = bytes.toString("ascii", position + 4, position + 8);
    if (kind === "IDAT") {
      compressed.push(bytes.subarray(position + 8, end - 4));
    }
    if (kind === "IEND") {
      if (length !== 0) {
        throw new Error("Malformed PNG end");
      }
      ended = true;
    }
    position = end;
  }
  const rowBytes = width * 4 + 1;
  const raster = inflateSync(Buffer.concat(compressed), {
    maxOutputLength: rowBytes * height,
  });
  if (!ended || raster.length !== rowBytes * height) {
    throw new Error("Malformed PNG raster bounds");
  }
  for (let row = 0; row < height; row += 1) {
    if (raster[row * rowBytes] > 4) {
      throw new Error("Malformed PNG filter");
    }
  }
  result = {
    format: { format_name: "png_pipe" },
    frames: [{ height, width }],
    streams: [{ codec_name: "png", codec_type: "video", height, width }],
  };
} else if (
  bytes.toString("ascii", 0, 4) === "RIFF" &&
  bytes.toString("ascii", 8, 12) === "WAVE"
) {
  const channels = bytes.readUInt16LE(22),
    rate = bytes.readUInt32LE(24),
    data = bytes.readUInt32LE(40);
  if (
    bytes.toString("ascii", 12, 16) !== "fmt " ||
    bytes.readUInt32LE(16) !== 16 ||
    bytes.readUInt16LE(20) !== 1 ||
    channels !== 1 ||
    rate !== 24_000 ||
    bytes.readUInt16LE(34) !== 16 ||
    bytes.readUInt16LE(32) !== 2 ||
    bytes.readUInt32LE(28) !== 48_000 ||
    bytes.toString("ascii", 36, 40) !== "data" ||
    data <= 0 ||
    data !== bytes.length - 44 ||
    data % 2 !== 0 ||
    bytes.readUInt32LE(4) !== bytes.length - 8
  ) {
    throw new Error("Malformed fixture WAV");
  }
  const duration = data / (rate * channels * 2);
  result = {
    format: { duration: String(duration), format_name: "wav" },
    streams: [
      {
        channels,
        codec_name: "pcm_s16le",
        codec_type: "audio",
        duration: String(duration),
        sample_rate: String(rate),
      },
    ],
  };
} else {
  throw new Error("Unknown probe format");
}
console.log(JSON.stringify(result));
