## ADDED Requirements

### Requirement: Durable truthful speech alignment
Generated speech provenance SHALL optionally retain a closed alignment record with independent required sentence, word, and phoneme timed-text arrays, exact native/forced/estimated quality, nonblank alignment providerId, and required nullable modelId/modelVersion. Omitted nullable model fields and explicit null alignment MUST fail closed decoding; omitted alignment is the only absence form. Timings MUST be safe JSON integer asset-relative milliseconds (0 through 9007199254740991), nonnegative starts and strictly greater ends, ordered without overlap within each granularity and bounded by the generated asset's known positive duration. At least one granularity MUST be nonempty. Producer metadata SHALL identify the alignment producer independently of synthesis identity. Absent alignment MUST remain absent without inferred quality or timings.

#### Scenario: Preserve each alignment quality
- **WHEN** a valid generated speech result supplies native, forced, or estimated alignment with any nonempty independent granularity combination
- **THEN** commit, replacement, undo/redo, and reopen preserve the exact quality, producer metadata, text and timing arrays

#### Scenario: Preserve legacy unaligned synthesis
- **WHEN** speech has no alignment
- **THEN** commit and regeneration retain existing behavior and omit alignment without estimating timestamps

### Requirement: Canonical bounded atomic rejection
Core MUST enforce at most 100000 combined segments,4096 UTF-8 bytes per nonblank segment text,1 MiB combined timing text, and256 UTF-8 bytes per nonblank producer/model identifier. Unknown keys, invalid quality, fractional/negative timestamps and malformed shapes MUST fail typed decoding; semantic ordering, limits and duration failures MUST return existing non-retryable VALIDATION_FAILED. Invalid input, missing replacement item and revision conflict MUST leave authoritative state/history/resources unchanged through existing mutation ownership. No path, expression or network resource interpretation SHALL be added.

#### Scenario: Reject invalid alignment atomically
- **WHEN** a result has empty granularity arrays, overlapping/reversed/out-of-duration timing, invalid identity/text, or exceeds a bound
- **THEN** core rejects it before publishing the generated asset or replacing an item and preserves complete authoritative bytes and resources

#### Scenario: Retain exact revision and reference failures
- **WHEN** valid alignment accompanies a stale expectedRevision or missing replacement item
- **THEN** the existing REVISION_CONFLICT or missing-reference error remains unchanged and no partial asset or provenance is published

#### Scenario: Distinguish required nullable producer metadata and absence
- **WHEN** alignment modelId/modelVersion are explicitly null, omitted, or alignment itself is null
- **THEN** explicit null producer fields are accepted without inference, omissions/null alignment fail closed decoding, and omitted alignment preserves the complete unaligned predecessor
