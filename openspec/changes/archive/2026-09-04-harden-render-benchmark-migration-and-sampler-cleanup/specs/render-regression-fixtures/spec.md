## ADDED Requirements

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
