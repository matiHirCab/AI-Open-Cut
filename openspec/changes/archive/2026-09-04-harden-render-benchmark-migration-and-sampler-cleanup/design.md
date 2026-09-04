## Context

The only supported migration source is the exact revision-2 fixture previously checked into the repository. Its schema-2 report shape and validation rules are recoverable from the retained Git revision. Current migration code checks only the report version and reference hashes. Separately, `ProcessTreeSampler::finish` stops its worker on success, but dropping its `JoinHandle` during panic detaches a worker whose stop flag remains false.

## Goals / Non-Goals

**Goals:** fail closed for incomplete legacy evidence, keep cleanup authority narrow, and guarantee bounded sampler shutdown on all exits.

**Non-Goals:** support arbitrary historical schema-2 data, change current schema-3 reports, recapture goldens, add production telemetry, or change public/persisted contracts.

## Decisions

### Model the exact legacy report

Add private strict-deserialization types for the prior schema-2 report and phase timings. Validate its fixture identity, environment and font identity, units, one warm-up, three measured samples, process-tree scope, median/maximum aggregation, report-only comparison policy, and finite non-negative timings. Unknown or missing fields fail.

Factor shared manifest validation so current and legacy paths enforce identical canvas, duration, audio, timestamps, tolerances, environment, safe-path, hash, frame-set, and reference-count rules while selecting the appropriate report validator. Legacy recognition therefore authorizes update or cleanup only for a complete revision-2/schema-2 generation.

Alternative considered: compare only with the historical checked-in digest. Rejected because migration tests and valid environment-specific observations still need structural validation without broadening the accepted schema.

### Make sampler shutdown idempotent and RAII-safe

Store the worker handle in an `Option`. A shared shutdown routine sets the stop flag and takes and joins the handle once. Explicit `finish` reports a worker panic and returns the peak; `Drop` calls the same routine and suppresses join errors so unwinding never double-panics. The worker observes the stop flag after its current refresh, bounding shutdown by refresh completion plus at most one sampling interval.

Tests use an exit signal set by the worker to prove both explicit completion and panic unwinding join before control continues.

### Bind schema-3 work counts to fixture revision 3

Treat stage-work counts as fixture-revision invariants rather than accepting arbitrary positive values. The strict validator requires every intent to report one decoded input, zero native-rasterized layers, two composited visual layers, and one encoded video stream; frame preview reports zero encoded audio streams while audiovisual range preview and final export each report one. This preserves schema 3 and stage-definition version 1 while ensuring a hash-consistent report cannot claim work the canonical fixture did not perform.

Tests mutate every work-count field below and above its canonical value and require strict read-back rejection. Shared test observations use the same exact counts as the checked-in report.

### Complete schema-3 Git identity validation

Treat the `gitRevision` field as structurally required while allowing its value to be optional when Git metadata is unavailable: explicit JSON `null` is valid, while a string must contain at least one non-whitespace character. Use a field-level deserializer without a Serde default so omission fails before semantic validation while the in-memory type remains `Option<String>`. Do not constrain the value to a fixed hash width or encoding so the report remains compatible with repositories using different Git object formats.

Strict file read-back tests cover explicit null and nonblank identities as valid inputs and reject omission, empty, whitespace-only, and non-string values before publication or upload.

## Failure and Compatibility

Malformed or unsupported legacy fixtures remain selected/retained and fail before capture, replacement, or cleanup. Sampler shutdown cannot publish output or mutate fixtures. All changes remain inside test-only renderer infrastructure.

## Verification

Cover the complete prior schema, malformed report and manifest variants, exact schema-3 work counts, cleanup non-recognition, explicit sampler finish, and unwind shutdown. Run every repository-required Rust, native Linux, bridge, Python, and OpenSpec gate before archival.
