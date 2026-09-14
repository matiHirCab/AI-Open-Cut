## ADDED Requirements

### Requirement: Centralized managed font ownership
Core MUST extend its existing ownership discovery and managed-content integrity/collection policy to font catalogs and bindings in current state, all definitions including unused content, retained undo/redo, slot-bearing text and durable drafts. Font content MUST be addressed by its recorded hash and confined to project-managed storage; font records MUST NOT masquerade as timeline audio/video assets. Reachable content MUST NOT be collected. Missing records, missing bytes, hash mismatch and invalid retained face references MUST fail with ASSET_INTEGRITY_FAILED and the owning reference classification without ambient substitution. Lexical and canonical path escapes MUST preserve existing path errors. Removing/replacing text MUST retain fonts needed by another owner or history. Unreachable cleanup failures MUST retain ASSET_GC_FAILED and MUST NOT corrupt committed state.

#### Scenario: Retain fonts through every owner
- **WHEN** a font is reachable only from an unused component, slot-bearing text, a retained draft or an undo/redo snapshot
- **THEN** collection preserves it and the retained owner reopens and renders with the same hash

#### Scenario: Reject damaged or unsafe pinned content
- **WHEN** a bound font record or file is absent, modified, contains an invalid face reference or escapes managed storage
- **THEN** open/integrity/render preflight fails with the specified stable error and no fallback or output publication

#### Scenario: Collect only unreachable font bytes
- **WHEN** the last current, draft and history owner is removed and garbage collection succeeds or encounters a deletion failure
- **THEN** only unreachable managed font files are removed and cleanup failure reports ASSET_GC_FAILED while preserving the committed generation
