import { mkdirSync, writeFileSync, writeSync } from "node:fs";
import { dirname } from "node:path";

const [mode, ...args] = process.argv.slice(2);
if (mode === "ffprobe") {
  if (args.includes("-version")) {
    console.log("ffprobe fake 1.0");
  } else {
    console.log(
      JSON.stringify({
        format: {
          duration: args.some((arg) => arg.endsWith("rule-tone.wav"))
            ? "1.000"
            : "0.100",
          format_name: "fixture",
        },
        streams: [
          { codec_name: "rawvideo", codec_type: "video", height: 1, width: 1 },
          {
            channels: 1,
            codec_name: "pcm_s16le",
            codec_type: "audio",
            sample_rate: "24000",
          },
        ].filter(
          (stream) =>
            stream.codec_type !== "audio" ||
            !(
              args.includes("-select_streams") ||
              args.some((arg) => arg.includes("repeater-silent"))
            )
        ),
      })
    );
  }
} else if (args.includes("-filters")) {
  console.log(
    " ... overlay ... drawtext ... amix ... remap ... blend ... nullsrc ... split ... geq ... pad ... crop ... format ... "
  );
} else if (
  args.at(-1) === "pipe:1" &&
  args.some((arg, index) => arg === "-f" && args[index + 1] === "rawvideo")
) {
  const filter = args[args.indexOf("-vf") + 1];
  let dimensions;
  if (args.includes("-vf")) {
    dimensions = /^scale=([0-9]+):([0-9]+),format=rgba$/.exec(filter ?? "");
  } else if (
    args.some((arg, index) => arg === "-f" && args[index + 1] === "lavfi") &&
    args.includes("-i") &&
    args.includes("-pix_fmt") &&
    args[args.indexOf("-pix_fmt") + 1] === "rgba"
  ) {
    const source = args[args.indexOf("-i") + 1] ?? "";
    dimensions =
      /^color=c=[^:]+:s=([0-9]+)x([0-9]+):r=1:d=1,format=rgba(?:,drawtext=|$)/.exec(
        source
      );
  }
  const width = Number(dimensions?.[1]);
  const height = Number(dimensions?.[2]);
  const pixels = width * height;
  const bytes = pixels * 4;
  if (
    !(Number.isSafeInteger(width) && Number.isSafeInteger(height)) ||
    width <= 0 ||
    height <= 0 ||
    width > 16_384 ||
    height > 16_384 ||
    !Number.isSafeInteger(pixels) ||
    pixels > 16_777_216 ||
    !Number.isSafeInteger(bytes) ||
    bytes > 67_108_864
  ) {
    process.stderr.write("invalid fixture RGBA dimensions\n");
    process.exit(2);
  }
  const rgba = Buffer.alloc(bytes, Buffer.from([255, 0, 0, 255]));
  let offset = 0;
  while (offset < rgba.length) {
    offset += writeSync(1, rgba, offset, rgba.length - offset);
  }
} else {
  // Stream-discard only the actual image2pipe input, before producing output.
  if (args.some((arg, index) => arg === "-i" && args[index + 1] === "pipe:0")) {
    for await (const _chunk of process.stdin) {
      // Consume chunks through EOF without retaining a timeline buffer.
    }
  }
  // Render commands can append a discarded audio output after the PNG/MP4.
  let output = args.at(-1);
  if (args.includes("-filter_complex_script")) {
    output = args.includes("-frames:v")
      ? args[args.indexOf("[video]") + 1]
      : args[args.indexOf("-y") + 1];
  }
  if (!output) {
    process.exit(2);
  }
  if (output !== "-" && output !== "NUL") {
    const directory = dirname(output);
    if (directory !== ".") {
      mkdirSync(directory, { recursive: true });
    }
    writeFileSync(output, "fake rendered media");
  }
  console.log("out_time_ms=100000");
  console.log("progress=end");
}
