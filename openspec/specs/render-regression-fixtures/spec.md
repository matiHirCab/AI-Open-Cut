# Render Regression Fixtures Specification

## Purpose

Define deterministic, reviewable visual, audiovisual, filter-graph, timing, and memory baseline capture and conformance behavior for production rendering.

## Requirements

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

### Requirement: Explicit deterministic dependency gate
Required golden conformance MUST run only with explicit FFmpeg, FFprobe, and deterministic font configuration, and once the native golden gate is selected it MUST fail rather than skip when a configured dependency is missing or unusable, FFmpeg has insufficient filter support, or the deterministic font identity is inconsistent with the manifest. Tool identities MUST be retained in environment-tagged observations rather than requiring byte-identical FFmpeg builds across supported reference environments.

#### Scenario: Run required Linux conformance
- **WHEN** required CI configures the canonical rendering executables and font
- **THEN** the golden suite executes the native renders and reports success or a concrete failure instead of returning early as skipped

#### Scenario: Reject an unusable dependency
- **WHEN** a configured executable cannot run, a required filter is absent, or the deterministic font cannot be read or identified
- **THEN** conformance fails before accepting or updating golden references

### Requirement: Report-only timing and memory baseline
An explicit baseline capture MUST perform exactly one discarded warm-up and three measured renders, require deterministic conformance across the measured renders, and report the fixture and environment identity, warm-up and sample counts, median scene-evaluation time, median filter-graph construction time, median frame rendering time, median audiovisual range rendering time, median export time, median total elapsed time, and maximum sampled resident memory for the test process tree using finite non-negative values and declared units and aggregation metadata. Process-tree memory MUST include recursively discovered FFmpeg and FFprobe descendants. Required CI MUST resolve the report destination independently of Cargo's test working directory, create its parent directory before capture, and use that same workspace artifact for schema validation and upload. This change MUST NOT treat captured timing or memory values as universal pass/fail budgets.

#### Scenario: Capture comparable observations
- **WHEN** a caller runs baseline capture in a fully declared environment
- **THEN** it emits a machine-readable report declaring one warm-up, three measured samples, median timing aggregation, maximum memory aggregation, process-tree memory scope, and all required fixture, tool, font, and platform metadata

#### Scenario: Publish the required Linux observation
- **WHEN** Linux CI invokes golden conformance through Cargo from the repository workspace
- **THEN** capture writes schema 2 to an absolute workspace destination whose parent exists, and the subsequent validation and artifact upload consume that same file

#### Scenario: Compare unlike environments
- **WHEN** two reports have different operating-system, architecture, tool, font, fixture, sampling, scope, or aggregation identity
- **THEN** they remain separate observations and MUST NOT be presented as a like-for-like regression comparison

### Requirement: Deliberate atomic golden updates
Golden verification MUST be the default, and replacing reviewed references MUST require an explicit update mode that stages and validates a complete bounded immutable generation before installation, durably persists every retained file and every directory ancestor through the generation root, and only then atomically commits it through a versioned generation pointer. Every native golden invocation that reads, reconciles, renders, compares, captures, publishes, reports, or cleans the shared fixture container MUST first acquire the same blocking exclusive lock on a persistent coordination file and MUST retain that lock for its complete invocation. The coordination file MUST remain installed between invocations and MUST never be treated as cleanup residue. A pointer replacement MUST be considered committed once the atomic rename or replacement selects the new generation, even if a later pointer-directory durability sync fails. Failed content synchronization, generation installation, generation-directory synchronization, or other interrupted work confirmed before the pointer commit MUST leave the prior generation selected and byte-for-byte unchanged. A newly installed but unselected generation whose installation was confirmed successful MAY be removed with best effort or left as strictly recognizable inactive data for bounded reconciliation. When installation reports an error or otherwise has an ambiguous result, the invocation MUST NOT infer ownership merely because the digest path appeared and MUST leave that recognized path for a later locked reconciliation. A preexisting validated digest MUST never be removed because its resynchronization failed. When pointer durability is uncertain after a commit, both the prior and new complete durable generations MUST remain available so reopening can preserve whichever generation the strict pointer selects. Failure to clean an older generation after a successful commit MUST leave the new complete generation selected and retain the older generation for later bounded cleanup rather than reporting publication failure.

#### Scenario: Verify without rewriting selected evidence
- **WHEN** the golden command runs without explicit update mode
- **THEN** it performs conformance checks against the selected immutable generation and does not rewrite any checked-in reference or pointer, while it may remove only strictly recognized orphan data while holding the exclusive coordination lock

#### Scenario: Serialize concurrent golden invocations
- **WHEN** one golden invocation holds the coordination lock while using a selected generation or live stage and another invocation targets the same fixture container
- **THEN** the second invocation blocks before reading or reconciling the container and can clean recognized residue only after the first invocation releases the lock

#### Scenario: Release coordination after failure
- **WHEN** a golden invocation returns an error or unwinds through a panic after acquiring the coordination lock
- **THEN** RAII releases the lock and a later invocation can acquire it without unlinking or recreating the coordination file

#### Scenario: Synchronize a deeply nested reference
- **WHEN** a retained reference is nested below directories such as `frames/nested/deeper/0000.rgb`
- **THEN** the harness synchronizes `deeper`, `nested`, `frames`, and the generation root in deepest-first order before installing the generation

#### Scenario: Preserve an unconfirmed install destination
- **WHEN** generation installation reports an error and the destination digest path is observable afterward
- **THEN** the harness leaves `CURRENT` unchanged and preserves the recognizable destination for a later locked reconciliation instead of deleting it based only on observation

#### Scenario: Fail before pointer commit
- **WHEN** update mode cannot render, decode, hash, validate, synchronize every retained file and ancestor directory, durably install the complete generation, synchronize its required directory entries, or atomically replace the generation pointer before commit
- **THEN** it leaves the complete prior generation selected, removes only confirmed-owned new temporary or inactive output with best effort, and never exposes a missing or partial canonical set

#### Scenario: Fail first publication before pointer commit
- **WHEN** generation content or installation durability fails during an update with no existing `CURRENT`
- **THEN** the harness does not create `CURRENT` and leaves no selected canonical generation

#### Scenario: Reuse an installed digest
- **WHEN** the validated generation digest already exists before update mode attempts to select it
- **THEN** the harness revalidates and resynchronizes that generation before pointer commit and never removes the preexisting generation when synchronization fails

#### Scenario: Fail durability sync after pointer commit
- **WHEN** atomic pointer replacement selects the new complete durable generation but a later pointer-directory durability sync fails
- **THEN** publication remains committed with a non-fatal durability warning and retains both the prior and new generations

#### Scenario: Reopen after uncertain pointer durability
- **WHEN** the harness reopens after an uncertain durability result and `CURRENT` validly selects either the prior or new generation
- **THEN** it preserves that selected complete generation and may clean only the other strictly recognized inactive generation while holding the exclusive coordination lock

#### Scenario: Reconcile recognized orphan data
- **WHEN** any later golden invocation finds stale staging data, a pointer temporary, or an inactive generation with the harness's strict recognized naming and validated layout
- **THEN** it attempts to remove only those recognized inactive entries after acquiring the exclusive coordination lock and never removes the selected generation, persistent coordination file, or an unknown path

#### Scenario: Defer failed cleanup
- **WHEN** startup or post-commit cleanup of recognized inactive data fails
- **THEN** the selected complete generation remains usable and the invocation reports cleanup pending without treating publication or conformance as failed

### Requirement: Render failure and lifecycle integrity evidence
The regression suite MUST prove that invalid timing, missing asset references, and stale expected revisions retain their existing stable typed errors before render side effects; that successful rendering does not mutate the project revision, current state, drafts, or retained history; and that edit, undo, redo, and reopen preserve the canonical evaluated result for the corresponding retained state.

#### Scenario: Reject invalid fixture work without side effects
- **WHEN** a fixture variant has invalid timing, a missing asset reference, or a stale expected revision
- **THEN** the existing canonical error is returned, no renderer process or reference update occurs, no partial artifact is published, and project state plus history remain unchanged

#### Scenario: Undo, redo, and reopen a rendered edit
- **WHEN** a valid fixture edit is rendered, undone, redone, closed, and reopened
- **THEN** each retained state evaluates to its corresponding deterministic semantic result, the reopened redone state matches its reviewed result, and no migration or history rewrite occurs

### Requirement: Documented fixture provenance and review
The repository MUST document the fixture semantics, immutable-generation and pointer formats, reference formats, normalization tokens, tolerance and alignment calculations, reference-platform dependencies, capture and verification commands, performance sampling and process-tree scope, atomic commit and cleanup behavior, and reviewer checks required before accepting a golden update.

#### Scenario: Review an intended baseline change
- **WHEN** an implementation intentionally changes rendering output or the selected golden generation
- **THEN** a reviewer can reproduce the old and proposed results, inspect visual/audio/graph differences, confirm the pointer, generation digest, manifest hashes, environment identity, sampling metadata, and process-tree memory scope, and distinguish a conformance update from a platform-specific performance observation
