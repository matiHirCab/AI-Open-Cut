# Narration-driven inspection fixture

Issue #70 adds bounded read-only desktop inspection and a reproducible synthetic
project. It uses the existing saved/explicit `speech_markers_generate` API,
marker expressions, animation presets, semantic sound events, audio buses,
ducking, and master normalization. There is no speech-worker, protocol, schema,
operation catalog, or migration change. The recipe under
`contracts/narration-driven-fixture-v1.json` is test input, not a public wire
contract or a new provider capability.

## Generate and inspect

From the repository root, provide a directory that does not already exist:

```sh
cargo run -p opencut-editor-core --example narration_fixture -- /tmp/narration-demo
cargo build -p opencut-desktop --release
# Use the project-store and project-id printed by the generator:
target/release/opencut-desktop --project-store /tmp/narration-demo/projects --project-id PROJECT_ID
```

The generator uses local 48kHz stereo PCM oscillators; it performs no inference
or network requests. Its estimated alignment is deliberately synthetic. The
alignment producer is `synthetic-narration-fixture`, with null model metadata.
The saved speech source and an imported unaligned source have the same samples.
Supplying explicit alignment to the latter does not persist synthesis provenance.

| Cue | Local time | Sentence end | Visual | Event |
| --- | ---: | ---: | --- | --- |
| EVERY | 500ms | 900ms | red | narration_accent |
| SINGLE | 1000ms | 1400ms | green | narration_accent |
| ONE | 1500ms | 1900ms | blue | narration_accent |
| rules | 2400ms | 2900ms | yellow | narration_accent |
| Starting_with | 3200ms | 3900ms | cyan | narration_accent |
| Venusaur | 4300ms | 5000ms | magenta | narration_accent |

Every visual has a 200ms opacity tween and a 400ms interval. Each event has a
300ms interval, variant 1/seed 1, default gain -6dB, authored gain -3dB and the
`sfx` bus. Music routes to `music`, with `voiceover` ducking gain 0.25 and zero
attack/release. The narration item occupies the complete 6000ms interval, so the
bus activity envelope spans that interval; its gated waveform is not an inferred
speech-activity detector. Master normalization is enabled at -24LUFS, 7LU range,
and -2dBTP ceiling.

The inspector appears without an item selection. Navigate root and component
scopes and pages of at most 32 cues. Select a cue to inspect its ID, owning scope
and local time. Select the narration, music or event in the hierarchy to inspect
saved alignment, truthful absence, authored audio controls or captured event data.
Use granularity/segment navigation to inspect one asset-relative segment at a
time. Component times are definition-local, not inferred root instance times.
Bus summaries report authored settings; they do not claim measured effective DSP
output. All four currently supported buses fit within the 16-summary bound.
Refresh, item selection, undo/redo and reopening clear narration cursor state.
The desktop preview remains the existing placeholder; native media verification
uses the renderer, not this placeholder.

## Independent verification

`tests/support/narration_fixture.rs` supplies fresh-store authoring only. Core
lifecycle tests compare literal cue timing and captured metadata, complete owned
file inventories, and authoritative undo/redo/reopen state. Existing alignment,
marker and contract suites remain required, including malformed/null inputs,
source/scope validation and selected-word policy failures.

The native fixture oracle declares source frequencies, PCM equations, cue colors
and clock times independently. It checks full/range/draft preview and export
visuals against constant plates, including half-opacity compositing in linear
light using the IEC sRGB transfer function. It retains SSIM >= 0.99 and a
one-output-frame (100ms at 10fps) timing tolerance. Independent sample summation
checks the ordinary full-root precodec mix at RMS <= 0.0001. A separate literal
FFmpeg reference applies the fixed routing/ducking/event equations and the
established six-decimal gain representation, encodes AAC, then checks decoded
float PCM at RMS <= 0.0001. No production output becomes an expected fixture.

Active normalization is independently metered on complete-root ordinary-render
precodec PCM, including cropped preview and export. The target tolerance is
0.1LU and the true-peak ceiling tolerance is 0.01dB. Cropping preserves the
complete-root normalization decision; an excerpt's own integrated loudness need
not equal the target. AAC is lossy and its delivered peak can differ from
precodec measurement. These tests do not claim that encoding preserves the
normalization ceiling exactly. Shifted audio, omitted captured gain/events,
blank visual plates and incorrect normalization levels are negative controls.

Desktop tests borrow the actual selected slice of a 100000-word alignment,
materialize one segment, exercise scoped 4096-cue paging and stale revisions,
and compare owned bytes before/after read-only inspection. Actual GUI evidence
and final required-check results are recorded in the OpenSpec verification report.
