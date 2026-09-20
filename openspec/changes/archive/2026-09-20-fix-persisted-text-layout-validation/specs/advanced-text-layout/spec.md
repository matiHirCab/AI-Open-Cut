## ADDED Requirements

### Requirement: Persisted root layout validation across retained state

Core SHALL apply canonical text-style validation to layout-enabled root text, including hidden text, current state and retained undo/redo snapshots, before authoritative font or document publication. Invalid layouts MUST return non-retryable INVALID_ARGUMENT and preserve project, history, draft and font bytes and inventories. Shared validation MUST also cover direct renderer inputs. Existing document validation, component validation, locks, revision checks and transaction recovery MUST remain effective. Layout-absent root styles MUST retain their existing behavior.

#### Scenario: P1 Invalid current and hidden root layouts
- **WHEN** schema-21 current or hidden root text contains negative or excessive tracking, unusable padded bounds or missing fitting bounds
- **THEN** opening fails with non-retryable INVALID_ARGUMENT and authoritative files remain unchanged

#### Scenario: P2 Invalid retained history cannot be restored
- **WHEN** an undo or redo snapshot contains an invalid advanced root layout and opening or history restoration is requested
- **THEN** core rejects it with INVALID_ARGUMENT before publication and preserves all authoritative files

#### Scenario: P3 Valid and legacy state retains compatibility
- **WHEN** valid advanced text, layout-absent legacy text or valid stale drafts are reopened or migrated
- **THEN** existing state, font and draft semantics remain unchanged, including later revision conflicts without stale-draft replay
- **AND** existing old-schema layout restrictions, future-version rejection and unrelated error classifications remain effective

### Requirement: Precise persisted layout decoding errors

Structurally malformed TextStyle.layout values SHALL be rejected with non-retryable INVALID_ARGUMENT when decoded from supported persisted current state, retained history or durable draft text payloads, including component text. Explicit null, null optional nested values, wrong types, unknown fields and unsupported enums MUST remain invalid. Error classification MUST survive nested model decoding without exposing private classification markers. Unrelated persisted-data errors MUST retain their existing classification. Rejection MUST precede authoritative migration or font publication and MUST preserve all authoritative files.

#### Scenario: E1 Malformed current and retained layouts
- **WHEN** a supported persisted current or history text layout contains null, invalid nested structure, wrong types, unknown fields or unsupported enums
- **THEN** opening fails with non-retryable INVALID_ARGUMENT rather than INTERNAL_ERROR and preserves authoritative bytes and inventory

#### Scenario: E2 Malformed retained draft layouts
- **WHEN** retained text creation/update or component creation/update payloads contain structurally invalid layouts during schema-20 migration or schema-21 reopen
- **THEN** opening returns non-retryable INVALID_ARGUMENT before publication without replaying the draft or changing authoritative files

#### Scenario: E3 Native and MCP classification agrees
- **WHEN** native headless and MCP clients encounter malformed persisted layouts or unrelated malformed persisted data
- **THEN** layout failures expose INVALID_ARGUMENT with retryable false, unrelated errors retain their existing codes, and no private classification marker appears in public messages
