## ADDED Requirements

### Requirement: Invalid persisted root layouts fail before output work

Frame preview, audiovisual range preview and final export SHALL reject invalid advanced root text layouts, including hidden text, through shared core validation with non-retryable INVALID_ARGUMENT before destination inspection, workspace allocation, writes, publication or renderer process execution. Existing geometry, fitting, diagnostics, encoding, numerical limits and valid legacy rendering MUST remain unchanged.

#### Scenario: R1 Invalid root layout preflight has no side effects
- **WHEN** a renderer request receives root text with invalid tracking, unusable padded bounds or missing fitting bounds, including when its destination would otherwise fail
- **THEN** INVALID_ARGUMENT occurs before destination inspection, artifact allocation, publication or FFmpeg execution and existing output bytes and artifact inventories remain unchanged
