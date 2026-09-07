## ADDED Requirements

### Requirement: Fractional grid render conformance
Native conformance MUST include independent fractional-edge pixel expectations alongside frame/range/export comparisons and materialized-draft equivalence. Existing SSIM, PCM RMS and timing thresholds MUST remain unchanged. Rendering failures from clipping complexity MUST precede output preparation, and existing non-grid native golden guarantees MUST remain satisfied.

#### Scenario: Render fractional edges through shared evaluation
- **WHEN** a fractional grid fixture renders through frame, range, draft and export at equivalent selections
- **THEN** independent edge pixels retain correct coverage, semantic plans agree and decoded outputs meet existing tolerances

#### Scenario: Preserve failure and legacy guarantees
- **WHEN** clipping exceeds a budget or the existing non-grid golden fixtures execute
- **THEN** excessive work fails before side effects and legacy output remains within its established guarantees
