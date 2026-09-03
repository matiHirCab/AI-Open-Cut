## MODIFIED Requirements

### Requirement: Canonical render-regression fixture
The project MUST maintain one versioned, checked-in, synthetic render-regression fixture whose fixed canvas, frame rate, duration, sample timestamps, generated local media, deterministic font identity, visual layering, text, animation, and audio are evaluated and rendered through production editor-core semantics. The fixture manifest MUST identify and hash every retained reference, and MUST contain only safe fixture-relative paths and finite values subject to explicit bounds. A path whose prefix matches RFC 3986 scheme syntax MUST be rejected as a URI before filesystem interpretation.

#### Scenario: Load the reviewed fixture
- **WHEN** the golden conformance suite loads the canonical generation, manifest, and all referenced data
- **THEN** the generation pointer, digest, every hash, timestamp, media parameter, font identity, relative path, and fixture version validate before scene evaluation or renderer execution

#### Scenario: Reject invalid golden metadata
- **WHEN** the pointer or manifest has an unknown version, malformed generation digest, missing or duplicate reference, mismatched hash, non-finite or out-of-range value, absolute path, traversal, RFC 3986-style URI scheme, or fixture-root escape
- **THEN** conformance fails before scene evaluation, renderer execution, reference replacement, or artifact publication

### Requirement: Golden visual, audiovisual, and graph conformance
The golden suite MUST compare production frame preview, audiovisual range preview, and final export from the same immutable fixture against reviewed first, middle, and final visual references, bidirectionally aligned decoded float-PCM audio, probed timing, the exact normalized semantic plan, and the exact narrowly path-normalized filter graph. Visual SSIM MUST be at least `0.99`, the minimum decoded float-PCM RMS error across every eligible offset within one output video frame MUST be at most `0.0001`, and timing deviation MUST be no greater than one output video frame. An alignment candidate MUST retain at least the shorter stream length minus the maximum permitted offset.

#### Scenario: Verify the canonical output
- **WHEN** the fixture is rendered with its declared FFmpeg, FFprobe, font, dimensions, frame rate, interval, and sample timestamps
- **THEN** preview, range, and export meet every visual, bidirectionally aligned audio, timing, semantic-plan, and normalized-filter-graph reference without comparing encoded container bytes

#### Scenario: Detect coordinated renderer drift
- **WHEN** preview, range, and export change together but differ from a reviewed frame, aligned audio sample, timing value, semantic instruction, or non-path filter argument
- **THEN** golden conformance fails even if the three newly rendered outputs still agree with one another

#### Scenario: Repeat the render
- **WHEN** the same immutable fixture is evaluated and rendered repeatedly in the same declared environment
- **THEN** its semantic plan and normalized filter graph are exactly equal and decoded output remains within the same golden tolerances after bounded alignment

### Requirement: Report-only timing and memory baseline
An explicit baseline capture MUST perform exactly one discarded warm-up and three measured renders, require deterministic conformance across the measured renders, and report the fixture and environment identity, warm-up and sample counts, median scene-evaluation time, median filter-graph construction time, median frame rendering time, median audiovisual range rendering time, median export time, median total elapsed time, and maximum sampled resident memory for the test process tree using finite non-negative values and declared units and aggregation metadata. Process-tree memory MUST include recursively discovered FFmpeg and FFprobe descendants. This change MUST NOT treat captured timing or memory values as universal pass/fail budgets.

#### Scenario: Capture comparable observations
- **WHEN** a caller runs baseline capture in a fully declared environment
- **THEN** it emits a machine-readable report declaring one warm-up, three measured samples, median timing aggregation, maximum memory aggregation, process-tree memory scope, and all required fixture, tool, font, and platform metadata

#### Scenario: Compare unlike environments
- **WHEN** two reports have different operating-system, architecture, tool, font, fixture, sampling, scope, or aggregation identity
- **THEN** they remain separate observations and MUST NOT be presented as a like-for-like regression comparison

### Requirement: Deliberate atomic golden updates
Golden verification MUST be the default, and replacing reviewed references MUST require an explicit update mode that stages and validates a complete bounded immutable generation before installation and atomically commits it through a versioned generation pointer. Failed or interrupted work before the pointer commit MUST leave the prior generation selected and byte-for-byte unchanged. Failure to clean an older generation after a successful commit MUST leave the new complete generation selected and retain the older generation for later bounded cleanup rather than reporting publication failure.

#### Scenario: Verify without rewriting
- **WHEN** the golden command runs without explicit update mode
- **THEN** it performs conformance checks against the selected immutable generation and does not modify any checked-in reference or pointer

#### Scenario: Fail before pointer commit
- **WHEN** update mode cannot render, decode, hash, validate, install a complete generation, or atomically replace the generation pointer
- **THEN** it removes recognized temporary output, leaves the complete prior generation selected, and never exposes a missing or partial canonical set

#### Scenario: Interrupt after pointer commit
- **WHEN** the generation pointer was committed but old-generation cleanup fails or the process is interrupted
- **THEN** the new complete generation remains selected and the recognized inactive generation remains available for safe cleanup by a later invocation

#### Scenario: Clean recognized orphan data
- **WHEN** a later invocation finds stale staging data or an inactive generation with the harness's strict recognized naming and validated layout
- **THEN** it removes only those recognized inactive entries and never removes the selected generation or an unknown path

### Requirement: Documented fixture provenance and review
The repository MUST document the fixture semantics, immutable-generation and pointer formats, reference formats, normalization tokens, tolerance and alignment calculations, reference-platform dependencies, capture and verification commands, performance sampling and process-tree scope, atomic commit and cleanup behavior, and reviewer checks required before accepting a golden update.

#### Scenario: Review an intended baseline change
- **WHEN** an implementation intentionally changes rendering output or the selected golden generation
- **THEN** a reviewer can reproduce the old and proposed results, inspect visual/audio/graph differences, confirm the pointer, generation digest, manifest hashes, environment identity, sampling metadata, and process-tree memory scope, and distinguish a conformance update from a platform-specific performance observation
