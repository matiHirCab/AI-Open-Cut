## ADDED Requirements

### Requirement: Canonical render-regression fixture
The project MUST maintain one versioned, checked-in, synthetic render-regression fixture whose fixed canvas, frame rate, duration, sample timestamps, generated local media, deterministic font identity, visual layering, text, animation, and audio are evaluated and rendered through production editor-core semantics. The fixture manifest MUST identify and hash every retained reference, and MUST contain only safe fixture-relative paths and finite values subject to explicit bounds.

#### Scenario: Load the reviewed fixture
- **WHEN** the golden conformance suite loads the canonical manifest and all referenced data
- **THEN** every hash, timestamp, media parameter, font identity, relative path, and fixture version validates before scene evaluation or renderer execution

#### Scenario: Reject invalid golden metadata
- **WHEN** the manifest has an unknown version, missing or duplicate reference, mismatched hash, non-finite or out-of-range value, absolute path, traversal, URI, or fixture-root escape
- **THEN** conformance fails before scene evaluation, renderer execution, reference replacement, or artifact publication

### Requirement: Golden visual, audiovisual, and graph conformance
The golden suite MUST compare production frame preview, audiovisual range preview, and final export from the same immutable fixture against reviewed first, middle, and final visual references, aligned decoded float-PCM audio, probed timing, the exact normalized semantic plan, and the exact narrowly path-normalized filter graph. Visual SSIM MUST be at least `0.99`, aligned decoded float-PCM RMS error MUST be at most `0.0001`, and timing deviation MUST be no greater than one output video frame.

#### Scenario: Verify the canonical output
- **WHEN** the fixture is rendered with its declared FFmpeg, FFprobe, font, dimensions, frame rate, interval, and sample timestamps
- **THEN** preview, range, and export meet every visual, audio, timing, semantic-plan, and normalized-filter-graph reference without comparing encoded container bytes

#### Scenario: Detect coordinated renderer drift
- **WHEN** preview, range, and export change together but differ from a reviewed frame, audio sample, timing value, semantic instruction, or non-path filter argument
- **THEN** golden conformance fails even if the three newly rendered outputs still agree with one another

#### Scenario: Repeat the render
- **WHEN** the same immutable fixture is evaluated and rendered repeatedly in the same declared environment
- **THEN** its semantic plan and normalized filter graph are exactly equal and decoded output remains within the same golden tolerances

### Requirement: Explicit deterministic dependency gate
Required golden conformance MUST run only with explicit FFmpeg, FFprobe, and deterministic font configuration, and once the native golden gate is selected it MUST fail rather than skip when a configured dependency is missing or unusable, FFmpeg has insufficient filter support, or the deterministic font identity is inconsistent with the manifest. Tool identities MUST be retained in environment-tagged observations rather than requiring byte-identical FFmpeg builds across supported reference environments.

#### Scenario: Run required Linux conformance
- **WHEN** required CI configures the canonical rendering executables and font
- **THEN** the golden suite executes the native renders and reports success or a concrete failure instead of returning early as skipped

#### Scenario: Reject an unusable dependency
- **WHEN** a configured executable cannot run, a required filter is absent, or the deterministic font cannot be read or identified
- **THEN** conformance fails before accepting or updating golden references

### Requirement: Report-only timing and memory baseline
An explicit baseline capture MUST report the fixture and environment identity, warm-up and sample counts, scene-evaluation time, filter-graph construction time, frame rendering time, audiovisual range rendering time, export time, total elapsed time, and peak resident working-set memory using finite non-negative values and declared units. This change MUST NOT treat captured timing or memory values as universal pass/fail budgets.

#### Scenario: Capture comparable observations
- **WHEN** a caller runs baseline capture in a fully declared environment
- **THEN** it emits a machine-readable report with all required phase, total, memory, fixture, tool, font, platform, and sampling metadata

#### Scenario: Compare unlike environments
- **WHEN** two reports have different operating-system, architecture, tool, font, fixture, or sampling identity
- **THEN** they remain separate observations and MUST NOT be presented as a like-for-like regression comparison

### Requirement: Deliberate atomic golden updates
Golden verification MUST be the default, and replacing reviewed references MUST require an explicit update mode that stages a complete bounded fixture set, validates its manifest and hashes, and publishes the set atomically only after every capture succeeds. Failed or interrupted capture MUST leave the prior golden set unchanged.

#### Scenario: Verify without rewriting
- **WHEN** the golden command runs without explicit update mode
- **THEN** it performs conformance checks and does not modify any checked-in reference

#### Scenario: Fail partway through capture
- **WHEN** update mode cannot render, decode, hash, validate, or stage any required reference
- **THEN** it removes temporary capture output and leaves the complete prior golden set byte-for-byte unchanged

### Requirement: Render failure and lifecycle integrity evidence
The regression suite MUST prove that invalid timing, missing asset references, and stale expected revisions retain their existing stable typed errors before render side effects; that successful rendering does not mutate the project revision, current state, drafts, or retained history; and that edit, undo, redo, and reopen preserve the canonical evaluated result for the corresponding retained state.

#### Scenario: Reject invalid fixture work without side effects
- **WHEN** a fixture variant has invalid timing, a missing asset reference, or a stale expected revision
- **THEN** the existing canonical error is returned, no renderer process or reference update occurs, no partial artifact is published, and project state plus history remain unchanged

#### Scenario: Undo, redo, and reopen a rendered edit
- **WHEN** a valid fixture edit is rendered, undone, redone, closed, and reopened
- **THEN** each retained state evaluates to its corresponding deterministic semantic result, the reopened redone state matches its reviewed result, and no migration or history rewrite occurs

### Requirement: Documented fixture provenance and review
The repository MUST document the fixture semantics, reference formats, normalization tokens, tolerance calculations, reference-platform dependencies, capture and verification commands, performance-report scope, and reviewer checks required before accepting a golden update.

#### Scenario: Review an intended baseline change
- **WHEN** an implementation intentionally changes rendering output
- **THEN** a reviewer can reproduce the old and proposed results, inspect visual/audio/graph differences, confirm the manifest hashes and environment identity, and distinguish a conformance update from a platform-specific performance observation
