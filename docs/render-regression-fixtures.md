# Render regression fixtures

OpenCut's milestone-zero render baseline is the editor-core fixture `flat-scene-av-v1` under `crates/editor-core/tests/fixtures/render-golden`. It is a 160x90, 10-fps, one-second scene containing a layered solid, deterministic DejaVu Sans text with scale and opacity animation, and a generated 48 kHz mono PCM tone. The typed project builder lives beside the renderer tests and feeds the production `EvaluatedScene`, resource preparation, render planner, frame preview, audiovisual range preview, and export paths.

The fixture is regression evidence, not a public or persisted contract. It adds no project field, migration, headless request, MCP tool, capability, stable error, or renderer input syntax.

## Checked-in evidence

`CURRENT` is a closed version-1 pointer to one immutable directory under `generations/<digest>`. The digest is the lowercase SHA-256 of that generation's exact `manifest.json` bytes. The revision-2 manifest fixes the canvas, frame rate, duration, sample timestamps, audio representation, tolerances, reference environment, font hash, and SHA-256 hash of every retained file. Unknown fields and versions, malformed generation digests, incomplete or duplicate references, mismatched hashes, non-finite or out-of-range tolerances, and unsafe paths fail before rendering.

Reference paths must contain only normal fixture-relative components. Before filesystem interpretation, the validator rejects any RFC 3986-style scheme prefix: an ASCII letter followed by letters, digits, `+`, `-`, or `.`, then `:`. This rejects opaque forms such as `file:frame.rgb` and `data:text/plain,...` as well as URL forms. Absolute paths, traversal, and canonical or symlink escapes are also rejected.

The retained files are:

- `frames/0000.rgb`, `frames/0500.rgb`, and `frames/0900.rgb`: packed RGB24 frames, row-major, 160x90, captured from the exported fixture.
- `audio/reference.f32le`: decoded 48 kHz mono IEEE float PCM for the shared one-second interval.
- `semantic-plan.txt`: the exact renderer-neutral evaluated scene snapshot.
- `filter-graph.txt`: the exact FFmpeg graph after narrow path normalization.
- `performance-baseline.json`: a schema-3 report-only observation tagged with fixture, Git, OS, architecture, FFmpeg, FFprobe, font, units, sampling, aggregation, stage-definition, intent, and memory-scope identity.

Encoded PNG and MP4 bytes are deliberately not goldens. Container metadata and codec output can vary across supported FFmpeg builds; the test decodes real preview/range/export artifacts and compares their meaning.

## Normalization and tolerances

Normalization replaces only the configured font with `<FONT>` and the generated `textfile` workspace directory with `<WORKSPACE>`. It does not reorder filters, rewrite numeric expressions, normalize semantic values, or remove renderer arguments. Tests prove request-ID changes normalize while a changed `x` expression remains visible.

For all three timestamps, frame preview, range preview, and export must achieve SSIM >= 0.99 against the reviewed RGB reference. Range/export float PCM searches integer offsets in both directions through one output frame and uses the minimum RMS from candidates retaining at least the shorter stream length minus that maximum offset; the result must be <= 0.0001. Probed range/export duration may differ from 1,000 ms by at most one output frame (100 ms). The semantic plan and normalized filter graph compare exactly. Encoded-container bytes are never compared.

## Verify

Set all three native dependencies together and run the exact test:

```sh
OPENCUT_FFMPEG_PATH=ffmpeg \
OPENCUT_FFPROBE_PATH=ffprobe \
OPENCUT_TEST_FONT_PATH=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf \
OPENCUT_GOLDEN_REQUIRED=1 \
cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact
```

With none of the variables set, ordinary cross-platform unit runs skip native conformance. A partial configuration fails. `OPENCUT_GOLDEN_REQUIRED=1` also turns an absent configuration into a failure, so required CI cannot pass vacuously. Configured executables must run and expose `overlay`, `drawtext`, and `amix`; the font must be readable and have the reviewed SHA-256 identity.

Linux CI uses the explicit DejaVu path above and writes `target/render-baseline-linux.json`, uploaded as the `render-baseline-linux-v3` artifact. Performance reports from different tool, font, fixture, OS, architecture, sampling, scope, aggregation, stage-definition, or intent identities are unlike observations and must not be described as regressions against one another.

## Capture a report-only observation

Add `OPENCUT_GOLDEN_REPORT_PATH` to the verification command. Report, recapture, and update modes run one discarded warm-up followed by three measured captures and require their deterministic output to agree. Ordinary verification without one of those explicit modes performs conformance rendering only and does not execute benchmark probes. The schema-3 JSON contains separate `framePreview`, `audiovisualRangePreview`, and `finalExport` observations. Each reports median scene-evaluation, native-rasterization, filter-construction, decode-only, composite-to-null, encoded-output, and end-to-end durations plus work counts.

Stage timings are direct, non-additive workloads. Decode opens the production plan's ordered media inputs over their declared intervals without filters or encoding. Composite executes the production filter graph to the null muxer. Encoding executes the normal production command. Decode and composite therefore repeat work that is also present in encoding; never add or subtract these values to infer latency. `endToEnd` directly times evaluation, plan preparation, and the encoded production workload. The current renderer has no native graphics raster backend, so `rasterization` and `rasterizedLayers` are explicitly zero; FFmpeg `drawtext` work is part of composition. Omitting that zero-work stage is invalid.

A development-only sampler refreshes every five milliseconds during measured benchmark captures and records the maximum aggregate resident memory of the test process and all recursively discovered FFmpeg/FFprobe descendants. The sampler owns its worker thread: normal completion signals and joins it exactly once, while error and panic unwinding perform the same join without replacing the original failure. Shutdown waits only for the current process refresh and at most one sampling interval. The discarded warm-up executes the same benchmark workloads without memory sampling. Generated reports are strictly validated by editor-core before publication, and Linux CI reads the exact workspace report through that validator again before upload. These measurements are observations only; issue #15 adds no timing or memory pass/fail budget.

## Update reviewed references

Golden verification is the default. An intentional renderer change requires the explicit `OPENCUT_UPDATE_GOLDENS=1` flag with all native dependency variables configured. After validating those tools, every native golden invocation waits for an exclusive `fs2` lock on the persistent, Git-ignored `.golden.lock` coordination file. The invocation retains that lock through selection, reconciliation, rendering, comparison, staging, publication, reporting, and cleanup. Ordinary conformance is locked too, so a generation being read cannot be removed by another process. The coordination file remains between runs and is never cleanup residue.

The test captures and validates a complete temporary generation, synchronizes the manifest and every declared file, makes the generation's directory entries durable, and only then atomically replaces `CURRENT`. Unix walks from every retained file's parent through the generation root, deduplicates those directories, synchronizes them deepest-first before rename, and synchronizes both affected parents after installation. Windows installs the directory with write-through semantics after synchronizing its files. A validated digest that already exists is revalidated and resynchronized before reuse.

Pointer replacement is the commit point. A content-sync, generation-install, or generation-directory-sync failure occurs before that point, so it leaves the previous pointer and generation byte-for-byte intact. A generation whose installation was confirmed and then failed a later pre-commit durability step is removed with best effort; if rollback also fails, its strictly recognized inactive directory remains available for bounded cleanup on reopen. An installation error is treated as unconfirmed even if its digest path appears afterward: that path is preserved for the next locked reconciliation instead of being deleted based only on observation. A preexisting digest is never removed because resynchronization failed. Once pointer replacement selects the new generation, a later pointer-directory-sync failure is reported as pending durability rather than publication failure; both the previous and new complete durable generations are retained so reopening is safe whether the old or new pointer state persisted.

While holding the coordination lock, every native golden invocation validates `CURRENT` and its selected generation before capture, then attempts bounded reconciliation. Current revision-3 generations always receive full manifest and schema-3 report validation, including in update mode. The only migration source is the exact immediately preceding revision-2 fixture with its complete schema-2 report. Legacy validation rejects unknown or missing report fields and applies the same canonical media metadata, timestamp, tolerance, environment/font, safe-path, hash, frame-set, and reference-count invariants as the current fixture; it additionally verifies every legacy report identity, unit, sample count, aggregation rule, memory scope, comparison policy, and finite non-negative phase timing. Cleanup removes only inactive generations that satisfy that same complete current-or-supported-legacy classification, `.stage-<uuid>` directories, and `.CURRENT.tmp-<uuid>` files. An initial update without `CURRENT` cleans only recognized temporaries because no generation can yet be identified as inactive. Unsupported revisions, incomplete or malformed legacy/current reports, unknown paths, `.golden.lock`, and the selected generation are never removed, and ordinary conformance never rewrites selected evidence or the pointer. Startup and post-commit cleanup failures are non-fatal pending work. Cleanup is skipped after an uncertain pointer-durability result and retried after the next validated reopen. Concurrent invocations block before fixture inspection, preventing cleanup from touching another process's live stage or selected generation.

For a non-publishing reproducibility check, set `OPENCUT_CAPTURE_GOLDENS_TO` to a new, nonexistent directory. The harness writes and validates a complete candidate there and requires every deterministic reference hash (frames, audio, semantic plan, and graph) to match the reviewed set. Its performance observation is intentionally excluded from byte equality.

Before accepting an update:

1. Run verification before the change and retain its performance report.
2. Run explicit update mode with the declared DejaVu font.
3. Run verification again without update mode.
4. Resolve `CURRENT`, then render RGB files inside its generation for visual inspection when needed, for example `ffmpeg -f rawvideo -pixel_format rgb24 -video_size 160x90 -i generations/<digest>/frames/0500.rgb frame.png`.
5. Inspect the semantic-plan and filter-graph diff; only documented path tokens may disappear through normalization.
6. Confirm manifest hashes, environment identity, audio/timing results, failure-path tests, and lifecycle tests.
7. Confirm schema 3 reports stage-definition version 1, non-additive timings, all three intents and six stages, coherent work counts, one warm-up, three measured samples, median timing aggregation, maximum memory aggregation, and `process_tree` memory scope.
8. Treat performance changes as comparable only when all environment, sampling, scope, and aggregation identity fields match; establish a budget in a separately approved change.

Invalid timing and missing assets are tested with executables that must never run and must leave the project and filesystem unchanged. The native headless lifecycle test separately proves stale-revision rejection before output plus successful render, undo, redo, draft isolation, and deterministic process-per-request reopen behavior.
