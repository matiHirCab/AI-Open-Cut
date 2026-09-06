## ADDED Requirements

### Requirement: Atomic schema 13 instance activation
Core MUST migrate supported schemas 1–12 and all retained undo/redo snapshots to schema 13 under the project lock using the existing recoverable transaction. The 12-to-13 step MUST change only the version and preserve content, revisions, provenance, assets and rendering for formerly valid input. Input MUST validate under its source schema before upgrade; previously forbidden root instances in schema 12 MUST NOT become valid through relabeling. Malformed current/history data or unknown future versions MUST fail without replacing the authoritative generation. Schema 13 MUST permit validated root instances with required canonical slotValues; older requests MAY omit optional values under the documented edit defaults. No automatic downgrade SHALL be provided.

#### Scenario: Migrate retained history and reopen
- **WHEN** a schema-12 project has nonempty undo/redo stacks and populated definitions and slots
- **THEN** current and retained snapshots migrate together preserving all content, undo/redo and repeated reopen remain deterministic, and root output stays equivalent

#### Scenario: Reject invalid and interrupted migration
- **WHEN** a snapshot contains invalid old-schema content, an unknown future version, or migration publication is interrupted
- **THEN** rejection leaves the original files intact or crash recovery selects one complete authoritative generation, never mixed current/history schemas
