## ADDED Requirements

### Requirement: All render intents consume common visual properties without output changes
Frame preview, audiovisual range preview, materialized draft preview, and final export MUST obtain transform and visibility semantics from the canonical common visual-properties value through the shared `EvaluatedScene` path. For content representable in schema version 6, migration to schema version 7 MUST NOT change evaluated instructions, active intervals, layer ordering, filter graphs, decoded pixels, audio samples, timing, or artifact publication behavior beyond existing documented tolerances.

#### Scenario: Compare pre-migration and migrated render intents
- **WHEN** the same schema-v6 fixture and its migrated schema-v7 state are rendered as a frame, audiovisual range, materialized draft preview, and final export
- **THEN** semantic plans are equal and visual, audio, and timing results satisfy the existing golden parity tolerances

#### Scenario: Keep new identity defaults non-operative
- **WHEN** a migrated caption or transition receives an identity transform solely because all variants now own common visual properties
- **THEN** evaluation and rendering preserve the schema-v6 caption or transition result and do not apply a new transform behavior

#### Scenario: Reject invalid common values before rendering
- **WHEN** common visual properties contain a non-finite, out-of-bounds, or otherwise invalid legacy transform
- **THEN** editor-core returns the existing typed validation failure before graphics preparation, filesystem path resolution, backend execution, or artifact publication
