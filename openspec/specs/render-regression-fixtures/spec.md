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
An explicit baseline capture MUST perform exactly one discarded warm-up and three measured renders, require deterministic conformance across the measured renders, and report the fixture and environment identity, stage-definition version, warm-up and sample counts, and per-intent observations for frame preview, audiovisual range preview, and final export. Ordinary golden conformance without report, recapture, or update mode MUST NOT execute benchmark-only decode, composition, or encoded-output probes. Every intent observation MUST contain finite non-negative direct measurements for scene evaluation, native graphics rasterization, filter-graph construction, media decoding, composition, encoding, and end-to-end elapsed time plus explicit work counts for decoded inputs, rasterized layers, composited layers, encoded video streams, and encoded audio streams. A stage with no work MUST be present with zero work and zero duration. Decode, composition, and encoding workloads MUST derive from the same immutable production `EvaluatedScene` and `RenderPlan`; independently timed workloads MUST be marked non-additive and MUST NOT be summed or subtracted to infer end-to-end latency. Capture MUST report maximum sampled resident memory for the test process tree using declared units and aggregation metadata, including recursively discovered FFmpeg and FFprobe descendants. Every aggregated report MUST pass the canonical strict validator before installation or publication. Required CI MUST resolve the report destination independently of Cargo's test working directory, create its parent directory before capture, and validate and upload that exact workspace artifact. Captured timing or memory values MUST remain report-only and MUST NOT become universal pass/fail budgets.

#### Scenario: Verify without benchmark capture
- **WHEN** configured native golden conformance runs without report, recapture, or update mode
- **THEN** deterministic render conformance runs without invoking benchmark-only decode, composition, or encoded-output probes

#### Scenario: Capture and validate stage observations
- **WHEN** explicit baseline capture completes one warm-up and three measured captures
- **THEN** every capture executes the three benchmark intents, the aggregated report passes strict typed and semantic validation, and memory is sampled only for measured captures

#### Scenario: Report a stage with no work
- **WHEN** the evaluated scene invokes no native graphics raster backend
- **THEN** every intent explicitly reports zero rasterized layers and zero rasterization duration rather than omitting, estimating, or reclassifying the stage

#### Scenario: Derive benchmark workloads from production semantics
- **WHEN** decode-only, composite-to-null, and encoded-output workloads are measured for an intent
- **THEN** they use the same immutable evaluated scene, ordered media inputs, source intervals, filter graph, stream mappings, and intent bounds as its production render plan and publish no benchmark-probe artifact

#### Scenario: Reject malformed observations
- **WHEN** a report omits or duplicates an intent or stage, contains an unknown schema or stage-definition version, non-finite or negative duration, inconsistent work count, additive timing claim, incomplete identity, or incorrect sampling, units, scope, or aggregation metadata
- **THEN** validation fails before the observation is accepted, installed, compared, or uploaded

#### Scenario: Publish the required Linux observation
- **WHEN** Linux CI captures a report to its absolute workspace destination
- **THEN** editor-core strictly reads and validates that same file before CI uploads it

#### Scenario: Compare unlike environments or definitions
- **WHEN** two reports have different operating-system, architecture, tool, font, fixture, sampling, scope, aggregation, stage-definition, or intent identity
- **THEN** they remain separate observations and MUST NOT be presented as a like-for-like regression comparison

### Requirement: Deliberate atomic golden updates
Golden verification MUST be the default, and replacing reviewed references MUST require an explicit update mode that stages and validates a complete bounded immutable generation before installation, durably persists every retained file and every directory ancestor through the generation root, and only then atomically commits it through a versioned generation pointer. A selected generation using the current fixture revision and report schema MUST receive full strict validation before update capture begins. Only the explicitly supported immediately preceding fixture revision and report schema MAY use migration-source validation; unsupported older or malformed current generations MUST fail before rendering, replacement, or cleanup. Every native golden invocation that reads, reconciles, renders, compares, captures, publishes, reports, or cleans the shared fixture container MUST first acquire the same blocking exclusive lock on a persistent coordination file and MUST retain that lock for its complete invocation. The coordination file MUST remain installed between invocations and MUST never be treated as cleanup residue. A pointer replacement MUST be considered committed once the atomic rename or replacement selects the new generation, even if a later pointer-directory durability sync fails. Failed content synchronization, generation installation, generation-directory synchronization, or other interrupted work confirmed before the pointer commit MUST leave the prior generation selected and byte-for-byte unchanged. A newly installed but unselected generation whose installation was confirmed successful MAY be removed with best effort or left as strictly recognizable inactive data for bounded reconciliation. When installation reports an error or otherwise has an ambiguous result, the invocation MUST NOT infer ownership merely because the digest path appeared and MUST leave that recognized path for a later locked reconciliation. A preexisting validated digest MUST never be removed because its resynchronization failed. When pointer durability is uncertain after a commit, both the prior and new complete durable generations MUST remain available so reopening can preserve whichever generation the strict pointer selects. Failure to clean an older generation after a successful commit MUST leave the new complete generation selected and retain the older generation for later bounded cleanup rather than reporting publication failure. Cleanup authority for inactive generations MUST use the same strict current-or-supported-legacy classification.

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

#### Scenario: Update from the current generation
- **WHEN** update mode opens a selected current revision-3/schema-3 generation
- **THEN** the complete manifest and performance report pass strict validation before capture or replacement

#### Scenario: Migrate the supported predecessor
- **WHEN** update mode opens a complete revision-2 generation with its schema-2 report
- **THEN** bounded migration-source validation permits recapture into the current format

#### Scenario: Reject unsupported or malformed generation data
- **WHEN** a current report is malformed or a selected or inactive generation uses an unsupported revision/schema pairing
- **THEN** the harness rejects it before capture or replacement and does not delete it as recognized cleanup data

### Requirement: Render failure and lifecycle integrity evidence
The regression suite MUST prove that invalid timing, missing asset references, and stale expected revisions retain their existing stable typed errors before render side effects or downstream stage observations; that successful rendering and benchmark capture do not mutate the project revision, current state, drafts, or retained history; and that edit, undo, redo, and reopen preserve the canonical evaluated result for the corresponding retained state. Benchmark-only decode and composition probes MUST publish no artifact and MUST use the same bounded workspace cleanup as the production plan from which they derive. Encoding, decode, and composition failures MUST each be independently injectable in tests and MUST stop subsequent benchmark stages.

#### Scenario: Fail during each measured renderer process
- **WHEN** encoding, decoding, or composition fails during benchmark capture
- **THEN** no observation is returned, no later process runs, temporary output and workspace residue are removed, project state remains byte-for-byte unchanged, and the selected golden generation is unchanged

#### Scenario: Reject invalid fixture work without side effects
- **WHEN** a fixture variant has invalid timing, a missing asset reference, or a stale expected revision
- **THEN** the existing canonical error is returned, no downstream renderer stage or process is observed, no reference update or partial artifact is published, and project state plus history remain unchanged

#### Scenario: Undo, redo, and reopen a rendered edit
- **WHEN** a valid fixture edit is rendered and observed, undone, redone, closed, and reopened
- **THEN** each retained state evaluates to its corresponding deterministic semantic result, the reopened redone state matches its reviewed result, and neither observation nor rendering performs a project migration or history rewrite

### Requirement: Documented fixture provenance and review
The repository MUST document the fixture semantics, immutable-generation and pointer formats, reference formats, normalization tokens, tolerance and alignment calculations, reference-platform dependencies, capture and verification commands, performance sampling and process-tree scope, atomic commit and cleanup behavior, and reviewer checks required before accepting a golden update.

#### Scenario: Review an intended baseline change
- **WHEN** an implementation intentionally changes rendering output or the selected golden generation
- **THEN** a reviewer can reproduce the old and proposed results, inspect visual/audio/graph differences, confirm the pointer, generation digest, manifest hashes, environment identity, sampling metadata, and process-tree memory scope, and distinguish a conformance update from a platform-specific performance observation

### Requirement: Complete schema-3 Git identity
Every schema-3 performance report MUST contain `gitRevision`, encoded as JSON null when Git metadata is unavailable or as a non-empty, non-whitespace string when present. The canonical strict validator MUST reject an omitted field and empty, whitespace-only, or non-string Git identities before installation, publication, comparison, or upload. Validation MUST NOT require a fixed hash length or encoding.

#### Scenario: Accept optional or present Git identity
- **WHEN** a schema-3 report contains either a null `gitRevision` or a nonblank string
- **THEN** Git identity validation succeeds and the remaining strict report checks continue

#### Scenario: Reject malformed Git identity
- **WHEN** a schema-3 report omits `gitRevision` or contains an empty string, a whitespace-only string, or a non-string non-null value
- **THEN** strict typed read-back rejects the report before it can be installed, published, compared, or uploaded

### Requirement: Exact schema-3 fixture work counts
Every schema-3 performance report for canonical fixture revision 3 MUST declare the exact stage-work counts performed by that fixture. Frame preview MUST report one decoded input, zero rasterized layers, two composited layers, one encoded video stream, and zero encoded audio streams. Audiovisual range preview and final export MUST report the same counts except for one encoded audio stream. The canonical strict validator MUST reject zero, inflated, or otherwise different counts before installation, publication, comparison, or upload.

#### Scenario: Accept canonical work counts
- **WHEN** a schema-3 report contains the exact work-count matrix for all three canonical fixture intents
- **THEN** work-count validation succeeds and the remaining strict report checks continue

#### Scenario: Reject incorrect work counts
- **WHEN** any decoded-input, rasterized-layer, composited-layer, encoded-video, or encoded-audio count differs from the canonical value for its intent
- **THEN** strict typed read-back rejects the report before it can be installed, published, compared, or uploaded

### Requirement: Exact legacy fixture migration
The golden harness MUST accept migration only from the exact immediately preceding revision-2 fixture and strict schema-2 performance-report format. Legacy validation MUST enforce the canonical canvas, duration, audio, timestamps, tolerances, environment and font identity, safe reference paths, reference hashes, frame timestamp set, reference counts, report identity, units, sampling, aggregation, memory scope, comparison policy, and finite non-negative timing values. Missing or unknown fields and unsupported revision/report pairings MUST fail before capture, replacement, or cleanup. The same complete validation MUST govern selected migration sources and inactive-generation cleanup recognition.

#### Scenario: Migrate a complete legacy generation
- **WHEN** update mode opens a hash-consistent revision-2 generation containing the complete prior schema-2 report
- **THEN** the harness accepts it as a bounded migration source and may recapture the current generation

#### Scenario: Reject incomplete legacy evidence
- **WHEN** legacy manifest metadata or report fields are missing, unknown, non-finite, inconsistent, or paired with the wrong revision or report schema
- **THEN** the harness fails before rendering or replacement and does not classify the generation as removable cleanup data

### Requirement: Bounded process-tree sampler lifecycle
Every started benchmark process-tree sampler MUST signal and join its worker exactly once on explicit completion and on every early return or panic after startup. Explicit completion MUST surface a worker panic; cleanup during unwinding MUST never panic. Sampler shutdown MUST complete after the worker's current refresh and at most one declared sampling interval, and MUST NOT leave a detached worker that can consume resources or perturb later benchmark observations.

#### Scenario: Finish a measured capture
- **WHEN** benchmark capture completes normally
- **THEN** explicit sampler completion stops and joins the worker before returning the maximum observed process-tree resident memory

#### Scenario: Unwind after benchmark failure
- **WHEN** encoding, decode, composition, or later conformance work fails after the sampler starts
- **THEN** RAII cleanup stops and joins the worker before unwinding continues without replacing the original failure

### Requirement: Independently visible render-parity gate
Continuous integration MUST publish a dedicated required Linux render-parity status that configures explicit FFmpeg, FFprobe, deterministic font, required-gate, and absolute report-path dependencies; executes production preview, audiovisual range, export, and lifecycle conformance with fail-closed critical steps against the selected immutable golden generation; strictly validates the report captured at the declared absolute workspace path; and only then uploads that exact validated observation. The gate MUST contain only its exact reviewed checkout, dependency, toolchain, native-conformance, report-validation, and upload steps in that order. Workflow-level and render-job-level environment maps MUST be absent; required CI MUST use only exact approved step environments, reject inherited execution defaults and job containers, and reject `OPENCUT_UPDATE_GOLDENS` or `OPENCUT_CAPTURE_GOLDENS_TO` from every effective configuration path so reviewed references remain immutable. Critical conformance, validation, and publication steps MUST NOT ignore failures or contain incompatible command alterations.

#### Scenario: Accept reviewed deterministic output
- **WHEN** production preview, audiovisual range preview, final export, and edit/undo/redo/reopen lifecycle behavior match the reviewed fixture under the exact declared Linux sequence and step-scoped environment
- **THEN** the dedicated render-parity status succeeds and publishes the strictly validated report-only observation

#### Scenario: Reject coordinated output drift
- **WHEN** preview, range, and export drift together from a reviewed frame, decoded audio reference, timing value, semantic plan, or normalized filter graph
- **THEN** the dedicated render-parity status fails even when the newly rendered outputs agree with one another

#### Scenario: Reject a weakened deterministic environment
- **WHEN** inherited workflow or render-job environment is declared, or a required executable, filter, font identity, selected generation, reference, report field, report destination, approved step environment key, step property, or execution setting is missing, additional, invalid, or inconsistent
- **THEN** repository policy validation fails before the dedicated render-parity status can accept or publish the observation

#### Scenario: Reject an environment-persisting step
- **WHEN** an added or replaced step attempts to alter later steps through `GITHUB_ENV` or another unreviewed command
- **THEN** repository policy validation fails because the render leaf sequence is not exact

#### Scenario: Reject inherited golden mutation mode
- **WHEN** golden update or alternate-capture mode is declared at workflow, render-job, native-step, validation-step, or job-container scope
- **THEN** repository policy validation fails before required CI can replace or bypass comparison with the reviewed references

#### Scenario: Reject any inherited process control
- **WHEN** workflow or render-job `env` is present with an empty map, a literal value, or an expression value
- **THEN** repository policy validation fails before inherited configuration can change process startup or command resolution

#### Scenario: Reject neutralized or reordered render evidence
- **WHEN** a render step is added, duplicated, replaced, or reordered; a critical step uses a custom shell or ignores failure; execution defaults wrap the command; or report publication moves before strict validation
- **THEN** repository policy validation fails before the weakened render gate can be accepted

#### Scenario: Preserve renderer semantics and report-only budgets
- **WHEN** the dedicated gate's closed verification-only sequence and isolated environment are enforced
- **THEN** golden references, render semantics, conformance tolerances, local deliberate update workflows, and application output remain unchanged and timing or memory observations do not become universal pass/fail budgets
