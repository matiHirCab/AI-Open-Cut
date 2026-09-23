## ADDED Requirements

### Requirement: Additive revisioned font-size editing
The existing `update_item` operation MUST accept optional integer `fontSize` in [1,1000] on text items in standalone, aliased batch and durable draft workflows. Omission MUST preserve size; a supplied value MUST change only the text item's stored font size and retain its ID, document/runs/spans, color, style, font binding, transform, timing and other authored fields. Core MUST reject null, fractions, values outside the range and non-text targets with non-retryable INVALID_ARGUMENT; missing items, locked tracks and stale revisions MUST retain their existing typed errors and rollback behavior. Successful edits MUST remain one revision/undo step, with exact undo/redo/reopen behavior. Old requests MUST remain valid. Canonical headless/MCP catalogs, Rust/TypeScript consumers, capability reporting and parity evidence MUST agree with the additive field; no persisted schema migration is required.

#### Scenario: Resize without replacing text
- **WHEN** a standalone or aliased batch edit updates `fontSize` on an existing styled text item
- **THEN** its ID, document, font binding and unrelated fields remain unchanged while evaluation uses the new size, including after undo, redo and reopen

#### Scenario: Reject invalid size and targets atomically
- **WHEN** `fontSize` is null, fractional, zero, greater than 1000, targets a non-text or missing item, or follows an earlier valid operation in a batch that later fails
- **THEN** structural or core validation returns the established typed error and project revision, history and authoritative files remain unchanged

#### Scenario: Preserve compatibility and stale-revision behavior
- **WHEN** an old update request omits `fontSize`, a durable draft previews size changes, or an edit uses a stale revision
- **THEN** old behavior remains valid, draft preview leaves authoritative state unchanged, and stale edits return retryable REVISION_CONFLICT without mutation
