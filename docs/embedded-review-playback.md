# Embedded review playback

Render a range through the existing desktop review controls, then choose **Open embedded player**. Playback consumes that completed local MP4; opening, seeking or looping never renders again or edits the project. The first supported backend is Linux with an installed compatible `libmpv.so.2` (client API major 2 and software-render API). The application does not bundle or download it. Windows/macOS and missing/incompatible backends explicitly retain **Reveal MP4 for external playback**. The existing FFmpeg/FFprobe configuration still controls rendering, independently of playback readiness.

Use Play/Pause, Seek start, Seek −1 s / +1 s, and Loop. Seek is clamped to the captured interval. The clock shows artifact-relative milliseconds and the corresponding captured project time; Loop repeats this artifact, including its original audio, rather than playing the current timeline. Close player releases playback while retaining review metadata and the external-file fallback. A paused review does not advance. Native timing verification permits 200ms tolerance.

Captured revision, interval, dimensions, fps and requested audio setting remain unchanged. Edits, Refresh, Undo and Redo can mark the review STALE; the old artifact is never relabeled as current. Rendering a replacement stops its old player. A new window starts without retained review metadata. Normal window close releases its player; the existing refusal to close during an active render/edit remains unchanged.

A missing or evicted file requires rendering a new review. Unsupported media, unavailable video/audio output and failed controls show a fixed playback error; they do not silently regenerate, retry rendering or publish project changes. There is no path/URL entry, external-track discovery, user mpv configuration/script loading or network playback. Native diagnostic strings and absolute paths are not displayed in playback errors.

Display frames fit within 1280×720 while preserving aspect. Original metadata remains visible even when the display is scaled. The worker admits at most eight pending controls and retains only its latest decoded frame. The presentation owns one displayed and one pending image and evicts retired decoded assets and active-window atlas entries. Native render context, client and library reference are released in that order. After context creation, controls and properties use only render-thread-safe asynchronous libmpv APIs.

## Verification

The original synthetic GUI fixture creates a new disposable project containing red 0–1500ms, blue 1500–3000ms and an original 880Hz tone:

```sh
cargo run -p opencut-desktop --example review_playback_fixture -- /tmp/new-review-playback-fixture
```

Launch desktop with the printed project arguments. Set range 0–3000ms and Project resolution, render, open the player, seek to 2000ms, and inspect blue. Play from the start and inspect red then blue, pause, loop, Undo/Redo and render a replacement. Inspect captured/STALE identity, repeated close/open, absent media and unavailable backend, then close the actual window while playing. Preserve screenshots and observed commands/results. On a virtual Linux display, a working window manager and Vulkan driver are essential; a black mapped window is not GUI evidence.

Run the controller tests and an explicit real backend control:

```sh
cargo test -p opencut-desktop playback
OPENCUT_TEST_MPV=1 OPENCUT_FFMPEG_PATH=/usr/bin/ffmpeg \
  OPENCUT_TEST_AUDIO_CAPTURE=/tmp/owned-isolated-output-monitor.pcm \
  cargo test -p opencut-desktop \
  review::playback::tests::native_playback_pixels_audio_pause_seek_loop_and_cleanup \
  -- --ignored --exact --nocapture
```

Start an isolated capture of the actual selected output monitor into the named PCM file before the native command. It must contain this test's audio, with no competing audio clients. A realtime output clock (for example a dedicated PulseAudio null sink and its monitor) is required: an unpaced ALSA file/null sink can capture samples yet stall playback timing and is not valid timing evidence. The exact native test must report one executed pass, never a readiness skip. It checks independent red/blue pixel oracles, real samples, pause, seek, continued clock/frame progress after loop, repeated cleanup, valid short playback, short/long failed audio output without loop, missing files and invalid codecs. Physical speaker quality is not established by monitor capture.

Also run the existing explicit native desktop review test from [desktop review](desktop-review.md), required Rust/TS/contract/source/packaged/provider suites and protected OpenSpec gates. Unsupported platforms remain explicitly unsupported; source compilation alone does not prove playback on them.

Actual GPUI observations of the original fixture (Linux virtual display/software Vulkan):

- [Paused seek at 2000ms displays blue](fixtures/embedded-review/paused-seek-blue.png).
- [Playing after crossing the end displays red with Loop on](fixtures/embedded-review/playing-after-loop.png).
- [Undo keeps captured revision 4 and marks it STALE against revision 5](fixtures/embedded-review/captured-revision-stale.png).

These screenshots show performed controls and captured identity; they are not a measurement of physical audio or a hard GUI display-latency guarantee.
