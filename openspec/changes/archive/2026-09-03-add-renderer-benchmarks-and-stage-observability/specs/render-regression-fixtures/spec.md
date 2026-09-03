## MODIFIED Requirements

### Requirement: Report-only timing and memory baseline
An explicit baseline capture MUST perform exactly one discarded warm-up and three measured renders, require deterministic conformance across the measured renders, and report the fixture and environment identity, stage-definition version, warm-up and sample counts, and per-intent observations for frame preview, audiovisual range preview, and final export. Every intent observation MUST contain finite non-negative direct measurements for scene evaluation, native graphics rasterization, filter-graph construction, media decoding, composition, encoding, and end-to-end elapsed time plus explicit work counts for decoded inputs, rasterized layers, composited layers, encoded video streams, and encoded audio streams. A stage with no work MUST be present with zero work and zero duration. Decode, composition, and encoding workloads MUST derive from the same immutable production `EvaluatedScene` and `RenderPlan`; independently timed workloads MUST be marked non-additive and MUST NOT be summed or subtracted to infer end-to-end latency. Capture MUST report maximum sampled resident memory for the test process tree using declared units and aggregation metadata, including recursively discovered FFmpeg and FFprobe descendants. Required CI MUST resolve the report destination independently of Cargo's test working directory, create its parent directory before capture, and use that same workspace artifact for strict schema validation and upload. Captured timing or memory values MUST remain report-only and MUST NOT become universal pass/fail budgets.

#### Scenario: Capture stage-separated observations
- **WHEN** a caller runs baseline capture in a fully declared environment
- **THEN** it emits one strict machine-readable observation for each frame, range, and export intent, declares one warm-up and three measured samples, median timing aggregation, maximum memory aggregation, process-tree memory scope, non-additive timing semantics, all six required renderer stages, end-to-end duration, work counts, and complete fixture, stage-definition, tool, font, and platform identity

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
- **WHEN** Linux CI invokes golden conformance through Cargo from the repository workspace
- **THEN** capture writes the current strict report schema to an absolute workspace destination whose parent exists, and subsequent validation and artifact upload consume that same file

#### Scenario: Compare unlike environments or definitions
- **WHEN** two reports have different operating-system, architecture, tool, font, fixture, sampling, scope, aggregation, stage-definition, or intent identity
- **THEN** they remain separate observations and MUST NOT be presented as a like-for-like regression comparison

### Requirement: Render failure and lifecycle integrity evidence
The regression suite MUST prove that invalid timing, missing asset references, and stale expected revisions retain their existing stable typed errors before render side effects or downstream stage observations; that successful rendering and benchmark capture do not mutate the project revision, current state, drafts, or retained history; and that edit, undo, redo, and reopen preserve the canonical evaluated result for the corresponding retained state. Benchmark-only decode and composition probes MUST publish no artifact and MUST use the same bounded workspace cleanup as the production plan from which they derive.

#### Scenario: Reject invalid fixture work without side effects
- **WHEN** a fixture variant has invalid timing, a missing asset reference, or a stale expected revision
- **THEN** the existing canonical error is returned, no downstream renderer stage or process is observed, no reference update or partial artifact is published, and project state plus history remain unchanged

#### Scenario: Fail during a measured renderer stage
- **WHEN** a decode, composition, or encoding benchmark process fails
- **THEN** capture rejects the complete observation, removes bounded temporary outputs, publishes no benchmark-probe artifact, and leaves the selected golden generation plus project state and history unchanged

#### Scenario: Undo, redo, and reopen a rendered edit
- **WHEN** a valid fixture edit is rendered and observed, undone, redone, closed, and reopened
- **THEN** each retained state evaluates to its corresponding deterministic semantic result, the reopened redone state matches its reviewed result, and neither observation nor rendering performs a project migration or history rewrite
