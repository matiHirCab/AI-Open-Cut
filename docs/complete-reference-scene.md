# Complete motion-graphics reference scene

`contracts/complete-reference-scene-v1.json` is a test-owned recipe for a six-second, 192×108, 10fps editable project. It uses schema44 and headless protocol1 through existing operations. It combines the previously shipped capabilities without changing public DTOs, persisted state, renderer behavior, or historical migration requirements. The independent `narration-driven-scene-v1`, rule-card and masked-hero references remain separate and unchanged.

| Group | Authored witness |
| --- | --- |
| Compositions | One six-layer shared card definition, four typed slots, three instances with Plan/Build/Check overrides and a shared parent |
| Vectors | Diagonal procedural grid, decorative ellipse, three-copy repeater and stroked radar ellipse |
| Typography | Three bold colored rich-text words with outlines and tracking |
| Animation | Versioned slide entrances, parent stagger, looping grid scan/radar, marker-bound impact typography |
| Compositing | Scoped hero owner, mask path animation, matte provider, blend mode, ordered blur/glow/tint and particles |
| Stacking | Explicit tracks, negative grid z-index, stable creation order and scoped parent identities |
| Atomic authoring | Ordered creation aliases and one complete construction batch; standalone core construction is also exercised |
| Narration | Six explicitly synthetic cue times: 500, 1000, 1500, 2400, 3200 and 4300ms |
| Audio | Voice/music/sfx/master routes, seeded shared semantic events, voice-driven ducking, master compressor and normalization, native LUFS/true-peak analysis |
| Review | Genuine frame, audiovisual range, cold/warm540p/720p cache evidence, final export and history/reopen rendering |

The core fixture writes actual stereo48kHz PCM16 sources from declared oscillators and literal voice windows; it stores estimated synthetic alignment provenance. The MCP fixture runs a local synthetic worker that writes actual PCM. The unchanged synthesis-worker response has no alignment field, so MCP supplies the explicitly estimated recipe alignment to the existing `speech_markers_generate` operation. This is deterministic timing evidence, not speech-model accuracy or a listening/intelligibility claim.

Preset visual channels currently require the legacy transform representation. The recipe explicitly clears `transform2d` through `update_item` before applying those presets; group parents retain their existing scoped transforms. It does not weaken preset validation.

## Build and inspect locally

Supply a new directory; the example refuses an existing directory instead of overwriting it:

```sh
cargo run -p opencut-editor-core --example reference_scene -- /absolute/new/reference-store
```

The example prints the exact `--project-store` and `--project-id` arguments for `opencut-desktop`. All authored layers, slots, markers and buses remain editable through the existing core/agent APIs and the [desktop inspectors](desktop-review.md).

```sh
cargo test -p opencut-editor-core --test reference_scene
```

This covers independent closed ten-group acceptance and changed-witness negative controls, standalone/atomic construction, invalid slot input, missing references, stale revisions, later batch rollback, immutable authoritative files/assets, duplicate overrides, a parent edit, undo/redo and fresh reopen. Standalone construction deliberately tests all operations independently and can take several minutes in debug builds due to retained history validation. No timing budget is inferred from that observation.

## Required native MCP verification

Install actual FFmpeg/FFprobe, Rust, Bun and Python, plus all four DejaVu Sans faces (regular, bold, oblique, bold-oblique) beside the configured regular font. The harness copies the family into its own configured font root. Missing dependencies or incomplete faces fail acceptance. Build the existing private cache instrumentation:

```sh
cargo build -p opencut-headless --features raster-cache-test-hooks
cd apps/agent-bridge
OPENCUT_REFERENCE_REQUIRED=1 \
OPENCUT_FFMPEG_PATH=/absolute/ffmpeg \
OPENCUT_FFPROBE_PATH=/absolute/ffprobe \
OPENCUT_TEST_FONT_PATH=/absolute/DejaVuSans.ttf \
OPENCUT_TEST_PYTHON=/absolute/python \
OPENCUT_REFERENCE_REPORT_DIR=/absolute/evidence \
bunx vitest run --config vitest.unit.config.ts tests/reference-scene-native.test.ts
```

Use the equivalent environment-variable syntax on Windows. Both source and compiled MCP clients must execute; the two default optional skips are not acceptance evidence. Private cache counters prove that matching warm requests avoid final rendering while producing separately owned artifact paths. The test checks actual error codes/retryability and project/history/asset inventory after rejected writes, and renders restored history/reopened content through the same native renderer.

At matched project settings, full-scene range/export equivalence retains SSIM≥0.99, aligned float-PCM RMS≤0.0001 and one-frame timing tolerance. Literal authored/timing witnesses and an opaque-gold decoded pixel witness with a changed-output negative control supplement this comparison. Cross-intent agreement is equivalence evidence, not an independent newly generated full-frame golden. Existing frozen predecessor pixel/audio oracles remain required.

Native reports preserve the final MP4 and measured artifact/cache/audio facts in separate source/compiled directories. Platform availability/packaging and release stress/performance orchestration belong to #78/#79; local Linux execution does not establish Windows/macOS media results. A proposed15–20-second rescue-video quality benchmark remains a separate creative plan and does not replace this fixture or #70's synthetic marker acceptance.

Original observed media and source/compiled reports are available in [recorded native evidence](verification/complete-reference-scene/README.md). They supplement the tests and are not new independent golden references.
