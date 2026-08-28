import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

const [mode, ...args] = process.argv.slice(2);
if (mode === "ffprobe") {
  if (args.includes("-version")) {
    console.log("ffprobe fake 1.0");
  } else {
    console.log(
      JSON.stringify({
        format: { duration: "0.100", format_name: "fixture" },
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
  console.log(" ... overlay ... drawtext ... amix ... ");
} else {
  const output = args.at(-1);
  if (!output) {
    process.exit(2);
  }
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, "fake rendered media");
  console.log("out_time_ms=100000");
  console.log("progress=end");
}
