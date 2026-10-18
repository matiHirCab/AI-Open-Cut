# Native visual decoder diagnostics (#177)

Visual decode failures use the existing nonretryable `FFMPEG_FAILED` error and `failedStage: "visual_decode"`. Existing `ffmpegExitCode` and `ffmpegStderrExcerpt` fields retain the available child status and bounded, path-redacted stderr, including when stdout is empty or ends with an incomplete RGBA frame.

The message identifies a fixed failure class: decoder startup, unavailable stdout/stderr pipe, stdout I/O failure, incomplete RGBA frame, no RGBA frame, exit-status collection failure, or unsuccessful child exit. Messages do not interpolate raw OS errors, commands or media paths. A read/output failure takes precedence over a simultaneous nonzero child exit; the exit code and stderr still describe the producer. A complete frame followed by an incomplete one is a failure, never a successful last-frame result. Complete successful streams still select the last complete frame and retain only two frame buffers.

On empty or incomplete-frame EOF, the adapter gives the child a bounded 100ms exit-status collection grace. If it has exited, its actual status is retained; if it remains alive, it is terminated and reaped. Non-EOF I/O failures and missing pipes terminate/reap immediately. Successful complete output retains its existing wait behavior. This adds an internal failure-cleanup guard, not a public render timeout or cancellation protocol. Interrupted stderr reads are retried; a later stderr I/O failure preserves the bounded fragment already captured for diagnostics.

Deterministic tests cover short/interrupted reads, injected I/O failures, empty output, incomplete initial/trailing frames, successful and unsuccessful processes, bounded Unicode/path-safe stderr, missing executable, a producer that remains alive after closing stdout, and child cleanup:

```sh
cargo test -p opencut-editor-core visual_decode
```

The native control creates original lossless red/blue 1280×720 RGBA frames at 30fps, encodes them locally with FFV1 and checks exact pixels around the color boundary:

```sh
OPENCUT_FFMPEG_PATH=/usr/bin/ffmpeg cargo test -p opencut-editor-core --lib render_process::tests::native_visual_decode_720p30_boundary_control -- --ignored --exact
```

This is a synthetic successful-decode control, not reproduction of the private Windows recording from #177. It does not identify that failure's root cause or establish Windows acceptance. Keep the issue open until an authorized affected-runtime fixture reproduces or resolves the native failure. The diagnostic improvement neither accesses private media nor changes seek/filter behavior, public fields, project/history state or render publication rules.
