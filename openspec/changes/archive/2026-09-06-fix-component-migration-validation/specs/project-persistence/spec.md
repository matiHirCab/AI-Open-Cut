## MODIFIED Requirements

### Requirement: Atomic schema 13 instance activation
Core MUST migrate supported schemas 1–12 and all retained undo/redo snapshots to schema 13 under the project lock using the existing recoverable transaction. The 12-to-13 step MUST change only the version and preserve content, revisions, provenance, assets and rendering for formerly valid input. Input MUST validate under its source schema before upgrade; previously forbidden root instances in schema 12 MUST NOT become valid through relabeling. Malformed current/history data or unknown future versions MUST fail without replacing the authoritative generation. Schema 13 MUST permit validated root instances with required canonical slotValues; older requests MAY omit optional values under the documented edit defaults. No automatic downgrade SHALL be provided.

Core MUST reject non-default legacy transforms on nested instances in source schemas 11–12 before upgrading their version, including hidden and unused definitions, using non-retryable INVALID_ARGUMENT. Current and all retained snapshots MUST receive this check; rejection MUST preserve the caller's original documents and byte-identical authoritative project/history files. Default legacy transforms and supported transform2d values MUST remain migratable, and valid schema-13 non-default legacy transforms MUST remain accepted.

#### Scenario: Migrate retained history and reopen
- **WHEN** a schema-12 project has nonempty undo/redo stacks and populated definitions and slots
- **THEN** current and retained snapshots migrate together preserving all content, undo/redo and repeated reopen remain deterministic, and root output stays equivalent

#### Scenario: Reject invalid and interrupted migration
- **WHEN** a snapshot contains invalid old-schema content, an unknown future version, or migration publication is interrupted
- **THEN** rejection leaves the original files intact or crash recovery selects one complete authoritative generation, never mixed current/history schemas

#### Scenario: Reject forbidden source transforms across current and history
- **WHEN** a schema-11 or schema-12 current, undo or redo snapshot contains a nested instance with non-default legacy position, scale or opacity, including hidden or unused definition content
- **THEN** opening fails with INVALID_ARGUMENT before migration publication and current/history files and in-memory input documents remain unchanged

#### Scenario: Preserve valid transform migration and schema-13 behavior
- **WHEN** supported old snapshots use default legacy transforms with absent or valid transform2d, or schema-13 snapshots use valid non-default legacy transforms
- **THEN** opening succeeds, supported old versions migrate without content changes, schema-13 transforms remain accepted, and mixed history and repeated reopen remain deterministic
