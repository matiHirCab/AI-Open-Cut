// Process-lifetime/report fixture only. Native target and PCM correctness use
// independently authored signals and actual FFmpeg, never these stub metrics.
import { spawn } from "node:child_process";
import {
  appendFileSync,
  existsSync,
  readFileSync,
  statSync,
  writeFileSync,
  writeSync,
} from "node:fs";
import { basename, dirname, join } from "node:path";

const [mode, ...args] = process.argv.slice(2);
const scriptIndex = args.indexOf("-filter_complex_script");
const inputIndex = args.indexOf("-i");
let input;
if (scriptIndex >= 0) {
  input = args[scriptIndex + 1];
} else if (inputIndex >= 0) {
  input = args[inputIndex + 1];
}
const root = input ? dirname(dirname(input)) : undefined;
const filter = args[args.indexOf("-af") + 1] ?? "";
const name = input ? basename(input) : "";
const faultPath = root ? join(root, ".normalization-test-fault") : undefined;
const fault =
  faultPath && existsSync(faultPath)
    ? JSON.parse(readFileSync(faultPath, "utf8"))
    : undefined;
let phase;
if (scriptIndex >= 0 && name === "master-normalization-capture-filter.txt") {
  phase = "capture";
} else if (
  name === "master-normalization-original.pcm" &&
  args.at(-1) === "pipe:1"
) {
  phase = filter.includes("volume=0.3") ? "correction" : "processing";
} else if (
  name === "master-normalization-original.pcm" &&
  filter === "loudnorm=I=-24:TP=-2:LRA=7:print_format=json"
) {
  phase = "original_measurement";
} else if (
  name === "master-normalization-original.pcm" &&
  filter.includes("loudnorm=")
) {
  phase = "target_measurement";
} else if (
  name.startsWith("master-normalization-") &&
  filter.includes("ebur128=")
) {
  phase = name.includes("corrected")
    ? "final_ebu_verification"
    : "ebu_verification";
} else if (
  name.startsWith("master-normalization-") &&
  filter.includes("loudnorm=")
) {
  phase = name.includes("corrected")
    ? "final_truepeak_verification"
    : "truepeak_verification";
}
if (root && phase) {
  appendFileSync(
    join(root, ".normalization-phase-events.jsonl"),
    `${JSON.stringify({ phase, pid: process.pid })}\n`
  );
  const control = join(root, ".normalization-test-phase");
  if (existsSync(control) && readFileSync(control, "utf8") === phase) {
    const descendant = spawn(
      process.execPath,
      ["-e", "setInterval(()=>{},1000)"],
      { stdio: "ignore", windowsHide: true }
    );
    writeFileSync(
      join(root, `.normalization-test-${phase}.pid`),
      JSON.stringify({ backend: process.pid, descendant: descendant.pid })
    );
    await new Promise(() => setInterval(() => undefined, 1000));
  }
}
if (fault && fault.phase === phase && fault.kind === "exit") {
  process.exit(23);
}
if (
  mode === "ffprobe" ||
  args.includes("-version") ||
  args.includes("-filters")
) {
  await import("./fake-media-tool.mjs");
} else if (args.at(-1) === "pipe:1" && args.includes("f32le")) {
  let frames;
  if (scriptIndex >= 0) {
    const script = readFileSync(input, "utf8");
    const range = /atrim=start_sample=(\d+):end_sample=(\d+)/.exec(script);
    frames = Number(range?.[2]) - Number(range?.[1]);
  } else {
    frames = statSync(input).size / 8;
  }
  if (!Number.isSafeInteger(frames) || frames <= 0 || frames > 28_800_000) {
    process.exit(2);
  }
  let sample = phase === "capture" ? 0.125 : 0.1;
  if (fault && fault.phase === phase && fault.kind === "nonfinite") {
    sample = Number.NaN;
  }
  const samples = Buffer.alloc(8192);
  for (let i = 0; i < samples.length; i += 4) {
    samples.writeFloatLE(sample, i);
  }
  let remaining = frames * 8;
  if (fault && fault.phase === phase && fault.kind === "partial") {
    remaining -= 1;
  }
  if (fault && fault.phase === phase && fault.kind === "short") {
    remaining -= 8;
  }
  if (fault && fault.phase === phase && fault.kind === "extra") {
    remaining += 8;
  }
  while (remaining > 0) {
    remaining -= writeSync(1, samples, 0, Math.min(remaining, samples.length));
  }
} else if (filter.includes("ebur128=")) {
  let integrated = name.includes("corrected") ? "-16.000" : "-16.300";
  if (fault && fault.phase === phase && fault.kind === "poison") {
    integrated = "nan";
  }
  console.log(`frame:1\nlavfi.r128.I=${integrated}`);
} else if (filter.includes("loudnorm=")) {
  console.error(
    JSON.stringify({
      input_i: "-18.00",
      input_lra: "0.00",
      input_thresh: "-28.00",
      input_tp:
        fault && fault.phase === phase && fault.kind === "infeasible"
          ? "0.00"
          : "-20.00",
      target_offset:
        fault && fault.phase === phase && fault.kind === "poison"
          ? "nan"
          : "0.00",
    })
  );
} else {
  await import("./fake-media-tool.mjs");
}
