# Illustrated scene benchmark v1

Issue [#181](https://github.com/matiHirCab/AI-Open-Cut/issues/181) defines the first small benchmark for epic #180. It uses existing core authoring and rendering. It does not install a reusable pack or implement later features.

The reviewed [recipe](../crates/editor-core/tests/fixtures/illustrated-scene/recipe.json) is original synthetic content: five solid native rectangle shapes, one native text item and one managed 64×64 two-color image. Role IDs map to actual native item IDs during authoring. Native geometry remains editable; the artwork is media whose internal pixels are not native child shapes. There is no external geometry script, private project, third-party art or flattened background plate.

| Parameter | Initial executable slice |
| --- | --- |
| Canvas / frame rate | 1920×1080 / 30 fps |
| Duration / sample timestamps | 1000 ms / 0, 500, 966 ms |
| Measured frame / range | 500 ms / [0,100) ms, three frames |
| Native roles | background, device_frame, screen_panel, label_panel, accent, artwork, label |
| Visual leaf / instance caps | 16 / 1; actual slice has seven root leaves and zero instances |
| Text | plain Unicode, 1–64 scalars; default `NATIVE HERO`, 48 px; 440×100 supported label box |
| Artwork | one opaque managed image, at most 64×64; default analytic left white/right red PPM, placed at 620,360 with scale 4 |
| Geometry / animation caps | 128 authored path commands; no effects, masks, repeaters or animation channels in this slice |
| Graphic colors | authored solid paints; existing color slots bind text, not shape/grid paint |

The inventory names supported bounded inputs, rather than advertising a new fixture customization API. Arbitrary text/artwork replacements must still pass existing core validation and the recipe's layout/input envelope; the independent pixel oracle is for the fixed original fixture.

Five shapes form a device frame and label panel, not a rasterized illustration substitute. Existing `add_shape`, `add_text`, managed import, `add_media` and `update_item` operations produce an ordinary persisted project. Reopening and read-only rendering preserve the same project/revision and stable role mapping. #182 owns the reusable component and existing text/artwork slot workflow; this benchmark does not pre-implement it.

The recipe's independent samples describe known interior and exterior colors before rendering. Verification checks those colors in actual PNG and decoded range frames (RGB channel tolerance 2 for PNG, 8 for encoded range), dimensions, exactly three range frames, text size/line count/bounded extent, seven native leaves and unchanged authoritative state. Eight widely separated samples cover every role except glyph interiors; text layout diagnostics and at least 100 visible white pixels in the independent label box verify the label without pinning antialiased glyph pixels. Negative controls reject inventory/cap/input drift and demonstrably fail black output. These checks are narrower than full-image goldens and do not prove dense animated pack readiness.

## Three final targets

All targets share 1920×1080 and 30 fps; duration is capped at 3000 ms and later temporal samples are 0, 1500 and 2966 ms. The layout envelopes and semantic roles are in the reviewed inventory. They are acceptance targets, **not implemented or measured compositions**.

| Target | Named roles | Inclusive complexity caps |
| --- | --- | --- |
| Illustrated device intro | device frame, screen, artwork, label, accent, backdrop | 24 visual leaves, 1 instance, 128 text scalars, 1 artwork ≤1024², 256 path commands, 8 channels |
| Sticker motif intro | authored contour, artwork, dot backdrop, sun rays, speed lines, label | 48 leaves, 2 instances, 128 scalars, 1 artwork ≤1024², 1024 commands, 16 channels |
| Role badge team intro | team parent, role badges, member artwork, member labels, title, backdrop | 96 leaves, 6 instances, 512 scalars, 6 artwork images ≤1024² each, 2048 commands, 32 channels |

Those counts are target inventories; existing core resource admission remains authoritative. Grid/repeater expansion must fit the target leaf cap. No assertion that all capped combinations render within existing guards is made. #184 owns authored sticker/motif geometry, #185 the team layout and #187 the eventual measured rendering strategy. No automatic silhouette tracing is specified.

## Reproduction and measurement semantics

Use the pinned project toolchain and reviewed DejaVuSans font:

```sh
OPENCUT_FFMPEG_PATH=ffmpeg OPENCUT_FFPROBE_PATH=ffprobe \
OPENCUT_TEST_FONT_PATH="$(pwd)/crates/editor-core/tests/fixtures/fonts/DejaVuSans.ttf" \
OPENCUT_GOLDEN_REQUIRED=1 OPENCUT_ILLUSTRATED_REPORT_PATH=/tmp/illustrated-scene.json \
cargo test --release -p opencut-editor-core --lib \
  renderer::golden::illustrated_scene::native_illustrated_scene_benchmark -- --exact --nocapture
```

The existing protected native golden test also invokes this fixture in Linux render parity, at the exact PR head. Existing protected commands, goldens, reports and resource limits are preserved. Missing required configured tools or the reviewed font fail that entry point. Set `OPENCUT_ILLUSTRATED_ARTIFACT_DIR` to an owned local output directory to retain the verified `frame500.png` and `range0-100.mp4`; these disposable captures are not authoritative project content. Their hashes are recorded in the report. A separate `illustrated_scene_observation=` JSON record in its log contains the fixture report; it does not modify the canonical schema-3 golden report.

For each intent, three fresh requests each create a new Renderer. One warm-up and three repeated requests share a Renderer and immutable project/request. Fresh does not flush OS/filesystem caches. Repeated requests include eligible raster and encoded-preview cache reuse; they do not measure an isolated rasterizer speedup. Timers cover facade call through artifact return, excluding independent decode/probe verification. Observations retain exact Git revision, tracked diff hash, fixture/driver hashes, executing test binary hash, build mode, host CPU/memory/OS, FFmpeg/FFprobe versions and font hash. An uncommitted test driver is identified by its explicit hash, rather than falsely described as unchanged main.

Existing guards remain in force: 268,435,456 sampled output-frame pixel work, 16,777,216 pixels per surface and 1,073,741,824 composition bytes, alongside core layer/geometry limits. The small workload's admission is evidence for this fixture only. A larger target's typed failure must be recorded; it cannot justify silently raising a limit.

The [reference observation](verification/illustrated-scene-benchmark/reference-observation.json) was captured on 2026-10-11 in the saved Debian 13 x86_64 engineering container: Xeon Platinum 8370C, five visible logical CPUs with a four-CPU cgroup quota, 16 GiB cgroup memory limit, Rust 1.97.0 release test build, FFmpeg/FFprobe 7.1.5 and the hash-pinned repository font. No other repository checks ran during this capture. Production source is main `431a5e38`; the new test-only driver/recipe and executing binary are identified by exact hashes in the record. Its tracked diff hash identifies the two-line golden-suite hook. This is a compiled native test runtime, not verification of a separately installed desktop build.

Three samples per cell follow the declared procedure; warm-up is excluded:

| Intent / condition | Median | Observed range | Reference target |
| --- | --- | --- | --- |
| frame / fresh Renderer | 2231.80 ms | 2149.61–2356.41 ms | 5000 ms |
| frame / repeated request | 1111.03 ms | 1095.77–1258.36 ms | 5000 ms |
| range / fresh Renderer | 3700.23 ms | 3625.25–3715.14 ms | 8000 ms |
| range / repeated request | 2823.41 ms | 2645.55–2965.53 ms | 8000 ms |

Targets were set after capture: for each intent, twice its largest measured fresh/repeated sample, rounded up to the next whole second (5 seconds for frame, 8 seconds for the three-frame range). These are bounded comparison goals for this named host/build/workload, with explicit headroom; they are not new renderer guards or portable CI timing assertions. No distribution, tail-latency or dense-scene guarantee follows from three samples. Existing guards admitted every fixed workload. Peak process-tree memory was not measured for this separate fixture; the host memory fields are environment metadata. The existing canonical golden memory report is unchanged.

The retained frame/range hashes identify the final repeated-request artifacts; their independently verified local captures are `/tmp/issue181-artifacts/frame500.png` and `/tmp/issue181-artifacts/range0-100.mp4`. They remain disposable engineering evidence, not imported background plates. CI emits new environment-tagged observations; unlike hosts/builds are not regression comparisons.

## Later feature boundaries

#182 owns one reusable native hero card and slots; #183 graphic color slots; #184 motifs; #185 role/team layouts; #186 resource diagnostics; #187 the measured strategy decision; #188/#189 only a justified source-preserving precomposition branch; #190 desktop controls; #191 full acceptance. #177–#179 own separate startup/progress work. #85 restart semantics remain independent. The recorded native observation preceded its external merge; publication rebases onto main `07d2c687`, whose #85 change does not alter Rust or rendering inputs. No detailed renderer redesign or product palette API is specified here.
