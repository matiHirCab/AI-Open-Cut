## Context

Editor-core already owns `EvaluatedScene`, render-plan construction, path-safe resource preparation, and the frame, audiovisual range, draft, and export entry points. A native test currently creates a synthetic 160x90, 10-fps, one-second project and proves those entry points agree with each other when FFmpeg, FFprobe, and a deterministic font are configured. That relative parity check can let all outputs drift together, does not retain a reviewed filter graph, and does not produce a reproducible timing or memory report.

Issue #14 is test and benchmark infrastructure for the completed milestone-zero evaluator/routing seam. It must not introduce a second scene definition, loosen resource safety, or promote test-only motion-graphics contracts into runtime or transport surfaces. Encoded MP4 bytes, wall-clock duration, and peak memory are environment-sensitive, while normalized render plans and decoded samples can be reviewed and compared portably within the existing tolerance contract.

## Goals / Non-Goals

**Goals:**

- Establish one canonical, small, synthetic fixture that covers visual layering, text, animation, and audio through the production evaluator and renderer.
- Retain reviewed first, middle, and final still references, short decoded audiovisual reference data, and an exact normalized filter-graph snapshot.
- Make the conformance command deterministic and fail closed when its explicit native dependencies or golden data are unavailable or invalid.
- Capture report-only render duration and peak resident-memory baselines with enough environment identity to compare like with like later.
- Prove repeated render, invalid input, missing reference, stale revision, undo/redo, and deterministic reopen behavior where those behaviors intersect rendering.
- Document a deliberate update and review workflow for baseline changes.

**Non-Goals:**

- No new project fields, operations, schema migration, history semantics, stable errors, capabilities, MCP tools, headless requests, or provider contracts.
- No byte-for-byte MP4 comparison and no universal timing or memory budget.
- No raw FFmpeg expressions, executable vector content, network fetches, arbitrary paths, or golden fixture behavior in production code.
- No coverage claim for future shapes, masks, effects, groups, components, or other motion-graphics features that are not active runtime semantics yet.

## Decisions

### Use one editor-core-owned canonical project builder

The fixture project and its generated tone source will live in editor-core test support and will be consumed only through production evaluation and render entry points. Its canvas, frame rate, duration, timestamps, colors, text, keyframes, audio frequency/sample rate, and deterministic font identity will be fixed. This keeps canonical timeline, validation, ordering, and animation rules in editor-core.

Alternatives considered: storing a hand-written project JSON would exercise deserialization but make the fixture vulnerable to incidental serialization churn; building separate FFmpeg commands would duplicate the behavior under test. A typed project builder is smaller and remains checked by the owning Rust types, while persistence/lifecycle scenarios separately exercise reopen behavior.

### Store decoded references and normalized semantic evidence, not encoded-byte hashes

The checked-in golden set will contain a versioned manifest, lossless still-frame references, decoded audio reference data for the shared interval, and a normalized filter-graph snapshot. The generated range preview and export remain real short A/V files, but tests compare decoded frames, aligned float PCM, probed stream timing, and semantic plans rather than container bytes. Filter-graph normalization will replace only fixture-root paths and generated workspace names with declared tokens; it will not reorder filters, rewrite numeric expressions, or remove semantic arguments.

Alternatives considered: checking in and hashing one MP4 is simple but brittle across FFmpeg builds and codec metadata; comparing only preview with export misses coordinated drift; broad textual normalization can hide a renderer regression. Decoded lossless references plus narrow path normalization preserve reviewable meaning and use the tolerances already specified by rendering-export.

### Separate deterministic conformance from performance capture

The normal native golden test will enforce exact manifest/schema and normalized graph equality plus the existing SSIM, PCM RMS, and one-frame timing limits. A dedicated opt-in capture mode will write a JSON report containing fixture revision, git revision when available, OS/architecture, FFmpeg/FFprobe/font identities, warm-up/sample counts, per-phase and total elapsed times, and peak resident working set. Performance values are observations grouped by environment identity, not pass/fail thresholds in this change.

Alternatives considered: enforcing the first measured numbers would create noisy CI and unsupported cross-platform promises; omitting performance data would not meet the milestone-zero baseline purpose. Separating the two keeps conformance stable while retaining evidence from canonical Linux CI and comparable local runs.

### Make golden updates explicit and bounded

The capture command will default to verification. Updating references will require an explicit update flag and an output directory rooted in the canonical fixture directory. It will generate into a temporary sibling, validate the complete manifest and all referenced files, and replace the baseline set only after successful capture. The manifest will include hashes for every reference file and reject unknown schema versions, missing entries, duplicate timestamps, non-finite metrics, unsafe relative paths, and paths escaping the fixture root.

Alternatives considered: implicit snapshot rewrites make accidental drift easy to bless; direct in-place writes can leave a mixed baseline after failure. Explicit atomic replacement makes the review boundary visible and prevents partial updates.

### Reuse existing lifecycle and failure surfaces

The golden suite will call the production `Renderer` and, for revision/history scenarios, `EditorCore`/headless behavior already owned by the repository. A stale expected revision must fail before rendering; invalid timing and missing assets must return their existing stable codes without files or state changes. A successful edit followed by undo, redo, close, and reopen will be compared using the canonical evaluated-plan digest and a selected golden frame. No test-only operation or transport request will be added.

Alternatives considered: a new baseline transport would make automation convenient but create an unnecessary compatibility surface; reproducing revisions in a test helper would duplicate core rules. Direct use of existing typed APIs keeps the evidence attached to canonical behavior.

### Run one hermetic platform in required CI

Linux CI will install the existing FFmpeg and DejaVu packages, set explicit executable and font variables, and run the golden conformance test. Once those variables are present, missing or unusable dependencies are failures rather than skipped tests. Other platforms may capture reports, but their performance numbers are not comparable unless environment identity matches.

Alternatives considered: allowing silent skips preserves developer convenience but can make the required gate vacuous; checking all platform outputs immediately expands scope without established runners and font packages. An explicit Linux reference platform gives the first reproducible baseline without claiming cross-platform byte identity.

## Risks / Trade-offs

- [Distribution FFmpeg updates can cause small decoded differences] -> Use the documented SSIM/RMS/timing tolerances, retain tool identity in reports, and require review for golden changes.
- [Narrow filter-graph normalization can miss a newly variable path] -> Fail exact snapshot comparison and add only a documented path token when the variability is proven non-semantic.
- [Broad filter-graph normalization can conceal behavior drift] -> Limit normalization to known fixture/workspace prefixes and test that semantic argument mutations still fail.
- [Performance sampling is noisy or platform-specific] -> Warm up, report multiple samples and environment identity, and do not enforce budgets in this milestone.
- [Binary fixture growth makes reviews harder] -> Keep the scene at 160x90 for one second, sample only defined timestamps, hash every file, and document commands for rendering visual diffs.
- [Update mode could overwrite reviewed evidence] -> Require an explicit flag, constrain paths, stage atomically, and keep updates visible in version control.

## Migration Plan

No runtime or persisted-data migration is required. Land the fixture harness, initial captured references, documentation, and CI gate together. Rollback consists of removing that additive test infrastructure and CI step; projects, history, contracts, and runtime outputs require no conversion or recovery.

## Open Questions

None. Release performance budgets and additional platform baselines are intentionally deferred until repeated comparable measurements exist.
