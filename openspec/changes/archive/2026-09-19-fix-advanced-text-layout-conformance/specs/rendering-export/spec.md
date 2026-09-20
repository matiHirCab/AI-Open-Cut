## ADDED Requirements

### Requirement: Corrected layout geometry and preflight parity

Frame, range, draft and export rendering SHALL consume the same core logical layout geometry, width diagnostics and shared candidate-glyph accounting. Work-limit failure MUST precede destination inspection, artifact allocation and publication. Existing visual, audio and timing tolerances, encoding settings and layout-absent behavior SHALL remain unchanged.

#### Scenario: R1 Corrected geometry across rendering intents
- **WHEN** an advanced layout with fractional bounds and negative paint margins is rendered through frame, range, draft and export paths
- **THEN** corresponding text-layout diagnostics agree exactly and decoded results agree at SSIM at least 0.99, audio RMS difference at most 0.0001 and timing difference at most one frame where applicable
- **AND** exact legacy regression expectations remain unchanged

#### Scenario: R2 Budget rejection has no output side effects
- **WHEN** expanded layout preflight exceeds its shared candidate-glyph budget for an output request
- **THEN** the existing work-limit failure occurs before destination inspection, artifact allocation or publication, including when the destination would otherwise fail validation
- **AND** existing output files and artifact inventories remain unchanged
