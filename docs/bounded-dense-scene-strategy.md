# Bounded dense-scene strategy evaluation

Issue [187](https://github.com/matiHirCab/AI-Open-Cut/issues/187) evaluates the completed original fixtures on the issue181 reference environment. [ADR0006](adr/0006-bounded-native-composition-strategy.md) selects one narrowly scoped **direct-rendering optimization candidate**; it does not claim that the candidate already meets the targets. Static source-preserving precomposition is not selected for this bounded slice.

The unchanged decision rule is the maximum of three fresh-Renderer samples in each of fourteen cells:5000ms for frame500ms and8000ms for range[0,100)ms (three frames). All cells must pass to declare current direct rendering meets the reference targets. Thirteen cells pass; row4 frame reaches5414.02ms. Repeated/cached results cannot rescue that fresh failure.

| Original workload | Expanded leaves | Fresh frame maximum (ms) | Fresh three-frame range maximum (ms) |
| --- | ---: | ---: | ---: |
| Issue181 baseline | 7 | 3475.30 | 4261.55 |
| Native hero, both defaults | 14 | 2213.60 | 3333.12 |
| Motif gallery, all ten instances | 16 | 1689.89 | 2451.23 |
| Team row, one member | 9 | 2429.72 | 3049.55 |
| Team row, four members | 36 | **5414.02** | 4661.64 |
| Team grid, one member | 9 | 1772.85 | 2287.52 |
| Team grid, four members | 36 | 3353.92 | 4672.18 |

[Raw observations](verification/bounded-dense-scene-strategy/reference-observation.json) retain all140 requests: per cell,3fresh+1excludedwarmup+3identicalrepeat+3raster-warm/encoded-cold controls. [Validation and comparison receipt](verification/bounded-dense-scene-strategy/reference-validation.json) records the original capture and later validation separately. The Debian13/Xeon8370C reference environment, five visible logical CPUs/four-CPU quota,16GiB cgroup limit, release Rust1.97, FFmpeg/FFprobe7.1.5 and pinned font match issue181. This is a source-native engineering runtime. Fresh means a new Renderer; OS caches are not flushed. The process-tree sampler adds observable instrumentation overhead compared with181, so these results do not establish a renderer regression or a portable timing guarantee.

All outputs passed independently authored colors/geometry/temporal states, original glyph regions and actual label-omission negatives, dimensions/three-frame counts and exact decoded cold/warm parity. Every render preserved complete authoritative/preexisting-file bytes and permitted only its returned new preview file. Original artwork is managed64x64 synthetic white/red imagery, not private content or a flattened native composition. The96-leaf/six-instance/3000ms roadmap envelope remains unmeasured.

## Source raster reuse and measured work

Counters distinguish three conditions rather than inferring a stage from timing:

- Fresh: actual successful source rasterization; equivalent occurrences already share eligible keys. A four-member badge has32cache-eligible text/shape occurrences,14misses and18hits on its first render.
- Identical repeat:1encoded hit, zero raster misses and32raster hits for four members. **Native visual preparation still runs** before the encoded-cache lookup; only final backend execution is absent.
- Raster-warm/encoded-cold: a Renderer clone shares raster bytes but has an independently empty encoded cache. It records actual raster hits, zero misses, native preparation and final backend execution, with identical decoded output.

For the failed row4 frame sample, facade elapsed is5414.02ms; directly observed source-miss closure time706.67ms/14calls, native visual preparation2307.17ms, final backend execution1755.95ms. These are directly measured, inclusive/non-additive observations. They are not summed/subtracted to infer an unmeasured stage. The largest measured preparation phases recur even with raster hits.

The existing preparation path parses byte rasters into linear premultiplied float pixels. Its `Raster::rgba` computes the same u8 sRGB transfer function repeatedly using `powf` for each channel/pixel, including bytes obtained from the raster cache. This inner hot-loop contribution is an **inference from inspected source**, not a separately measured cost or demonstrated optimization gain. The selected candidate is a256-entry **f64** transfer lookup, preserving alpha multiplication and the subsequent f32 cast. A separate issue must prove every256color×256alpha pair bit-for-bit and demonstrate the reference targets before claiming sufficiency.

Maximum sampled process-tree resident memory is680824832bytes, including the parent and recursively discovered native descendants at5ms sampling. It includes allocator/cache retention and can miss shorter peaks; it is not isolated per-request heap or the core's certified memory. Original owner-call maxima are317373696conservative composition bytes versus1073741824limit, and74649600charged pixel-samples versus268435456limit. These are maxima at original admission owners, not a second scene estimator or complete RSS certificate. Every fixed request was admitted; no existing guard was raised or labeled defective. Portable CI may report null quota values when the host exposes no cgroup files; null means unavailable, never unbounded/reference-equivalent. The reference comparison requires actual matching quota values.

## Reproduction and provenance

Use the pinned toolchain, no concurrent repository checks and the reviewed font:

```sh
OPENCUT_GOLDEN_REQUIRED=1 OPENCUT_FFMPEG_PATH=ffmpeg OPENCUT_FFPROBE_PATH=ffprobe \
OPENCUT_TEST_FONT_PATH="$(pwd)/crates/editor-core/tests/fixtures/fonts/DejaVuSans.ttf" \
OPENCUT_DENSE_REPORT_PATH=/tmp/bounded-dense-scene.json \
OPENCUT_DENSE_ARTIFACT_DIR=/tmp/bounded-dense-scene-artifacts \
cargo test --release -p opencut-editor-core --lib \
  renderer::golden::dense_scene_strategy::native_bounded_dense_scene_strategy -- --exact --nocapture
```

The existing required native golden entry also runs the complete matrix. Missing configured tools/font fail closed. Runtime current checkout must match compiled CARGO_MANIFEST_DIR before measurements. Driver, input, binary, source/diff, tool/font and environment hashes remain attributable; timing stays report-only in portable CI.

The original complete record is source head7043c449 with dirty approved187 inputs and driverdfa3f4cc305eaf48b974f7edbd8cde8d58dae312af8578def6e10e72fa215ff8. Its whole command initially returned101 after a mistaken validator assumption that an encoded hit skipped native preparation. All140native-output/state assertions had passed before that final classification failure. Corrected validation replay passed against the unchanged raw JSON (SHA25601194abacd1fdb644d9a6dda0d0922e6f751b1b9d2b16a6072dc317b9326bd7c), with validatorbff38f8bd1c4c1007ac9f6c5166b6f7f4523d25c84a4f7a67d9f96f158018731. Replay is not another native render or a passing original command.

An earlier incomplete run had an incorrect per-leaf-alpha overlap oracle; its logs/incoming captures remain excluded and were not turned into references. No captured pixel, tolerance, target, original pin, public catalog, persisted schema or production renderer was changed. Fresh final required native/repository checks and exact-head hosted acceptance remain mandatory and are recorded in the issue's verification artifact.

Conditional188/189 disposition and the separately scoped optimization issue are recorded in ADR0006 after independent decision review. Release stays blocked until selected follow-up implementation and the remaining illustrated-workflow acceptance pass. No installed desktop readiness, arbitrary fitting, portable performance, merge or deployment is claimed.
