## ADDED Requirements

### Requirement: Independent linear composition evidence
The native render suite MUST include independent analytic transfer-function, alpha, source-over, transparent-edge, and stage-order oracles plus production mixed-source preview/range/draft/export coverage. The oracles MUST NOT calculate expected values using production compositing helpers. Existing golden generation/hash/provenance/atomic-update policy MUST remain enforced; any changed reference MUST be reviewed against the intentional linear compositing correction and all unchanged audio/timing/semantic requirements. Missing configured native dependencies MUST fail selected conformance rather than skip.

#### Scenario: Detect shared encoded-space drift
- **WHEN** every render intent produces the same encoded-space midpoint or introduces hidden transparent RGB into an edge
- **THEN** independent linear/alpha reference assertions fail even when preview and export agree

#### Scenario: Review corrected goldens
- **WHEN** adopting the normative pipeline changes a reviewed frame or filter graph
- **THEN** deliberate atomic regeneration retains validated provenance and an independently justified color/alpha correction, and every required native conformance check passes against the reviewed generation

## MODIFIED Requirements

### Requirement: Report-only timing and memory baseline
An explicit baseline capture MUST perform exactly one discarded warm-up and three measured renders, require deterministic conformance across the measured renders, and report the fixture and environment identity, stage-definition version, warm-up and sample counts, and per-intent observations for frame preview, audiovisual range preview, and final export. Ordinary golden conformance without report, recapture, or update mode MUST NOT execute benchmark-only decode, composition, or encoded-output probes. Stage-definition version 2 for canonical fixture revision 4 MUST explicitly define `filter_graph_construction` as the directly timed aggregate of complete production resource preparation, local source rasterization/decode, floating-point scene materialization and lossless prepared-scene stream generation, plus filter-script construction. Its `decoding` and `compositing` observations MUST describe independently timed final prepared-input decoding and final opaque prepared-scene/backend routing probes; they MUST NOT be described as isolated CPU leaf-composition measurements. Every intent observation MUST contain finite non-negative direct measurements for scene evaluation, native graphics rasterization, filter-graph construction, media decoding, composition, encoding, and end-to-end elapsed time plus explicit work counts for decoded inputs, rasterized layers, composited layers, encoded video streams, and encoded audio streams. A stage with no work MUST be present with zero work and zero duration. Decode, composition, and encoding workloads MUST derive from the same immutable production `EvaluatedScene` and `RenderPlan`; independently timed workloads MUST be marked non-additive and MUST NOT be summed or subtracted to infer end-to-end latency. Capture MUST report maximum sampled resident memory for the test process tree using declared units and aggregation metadata, including recursively discovered FFmpeg and FFprobe descendants. Every aggregated report MUST pass the canonical strict validator before installation or publication. Required CI MUST resolve the report destination independently of Cargo's test working directory, create its parent directory before capture, and validate and upload that exact workspace artifact. Captured timing or memory values MUST remain report-only and MUST NOT become universal pass/fail budgets.

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

#### Scenario: Attribute linear materialization honestly
- **WHEN** the canonical linear compositor prepares a complete scene before FFmpeg routing
- **THEN** stage-definition version 2 reports the complete directly observed production preparation aggregate separately from the non-additive final prepared-input decode and opaque routing probes, with no inferred CPU-only timing or like-for-like version-1 comparison


### Requirement: Exact schema-3 fixture work counts
Every schema-3 performance report for canonical fixture revision 4 and stage-definition version 2 MUST declare the exact stage-work counts performed by that fixture. Frame preview MUST report two decoded inputs (the authored audio and prepared scene), zero native Rust source-rasterized layers, one final backend-composited prepared visual layer, one encoded video stream, and zero encoded audio streams. Audiovisual range preview and final export MUST report the same counts except for one encoded audio stream. The canonical strict validator MUST reject zero, inflated, or otherwise different counts before installation, publication, comparison, or upload.

#### Scenario: Accept canonical work counts
- **WHEN** a schema-3 report contains the exact work-count matrix for all three canonical fixture intents
- **THEN** work-count validation succeeds and the remaining strict report checks continue

#### Scenario: Reject incorrect work counts
- **WHEN** any decoded-input, rasterized-layer, composited-layer, encoded-video, or encoded-audio count differs from the canonical value for its intent
- **THEN** strict typed read-back rejects the report before it can be installed, published, compared, or uploaded

#### Scenario: Preserve incompatible stage definitions
- **WHEN** a revision-3/stage-definition-1 observation is compared to a revision-4/stage-definition-2 observation
- **THEN** strict comparison rejects them as unlike workloads rather than subtracting the synthesized prepared input or claiming comparable CPU composition timing


### Requirement: Exact legacy fixture migration
The golden harness MUST accept migration only from the exact immediately preceding revision-3 fixture, strict schema-3 performance-report format and stage-definition version 1. Legacy schema-3 work validation MUST enforce exactly one decoded input, zero rasterized layers, two composited layers, one encoded video stream and intent-dependent audio counts, while current revision-4/stage-definition-2 reports MUST enforce their distinct complete current matrix. Revision-2/schema-2 fixtures and all other older or unknown versions MUST be rejected. Legacy validation MUST enforce the canonical canvas, duration, audio, timestamps, tolerances, environment and font identity, safe reference paths, reference hashes, frame timestamp set, reference counts, report identity, units, sampling, aggregation, memory scope, comparison policy, and finite non-negative timing values. Missing or unknown fields and unsupported revision/report pairings MUST fail before capture, replacement, or cleanup. The same complete validation MUST govern selected migration sources and inactive-generation cleanup recognition.

#### Scenario: Migrate a complete legacy generation
- **WHEN** update mode opens a hash-consistent revision-3 generation containing the complete prior schema-3 report
- **THEN** the harness accepts it as a bounded migration source and may recapture the current generation

#### Scenario: Reject incomplete legacy evidence
- **WHEN** legacy manifest metadata or report fields are missing, unknown, non-finite, inconsistent, or paired with the wrong revision or report schema
- **THEN** the harness fails before rendering or replacement and does not classify the generation as removable cleanup data
