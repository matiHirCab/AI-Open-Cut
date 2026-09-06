import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

const [mode, ...args] = process.argv.slice(2);
if (mode === "ffprobe") {
  if (args.includes("-version")) {
    console.log("ffprobe fake 1.0");
  } else {
    console.log(
      JSON.stringify({
        format: {
          duration: args.some((arg) => arg.endsWith("rule-tone.wav")) ? "1.000" : "0.100",
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
        ],
      })
    );
  }
} else if (args.includes("-filters")) {
  console.log(
    " ... overlay ... drawtext ... amix ... remap ... blend ... nullsrc ... split ... geq ... pad ... crop ... format ... "
  );
} else {
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
