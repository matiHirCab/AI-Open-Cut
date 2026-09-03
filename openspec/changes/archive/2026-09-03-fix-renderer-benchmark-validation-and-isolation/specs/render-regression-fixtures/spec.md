## MODIFIED Requirements

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
