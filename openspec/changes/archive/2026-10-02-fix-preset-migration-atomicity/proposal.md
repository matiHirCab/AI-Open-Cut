## Why

A rejected preset application on a schema-28 project currently publishes schema-29 current/history migration before validating the operation. This contradicts the approved transactional preset guarantees and can make a failed edit prevent reopening with an older build.

## What Changes

- Restore the existing rejection guarantee for standalone presets and any ordered batch containing a preset, including migration, retained history, draft upgrades, and managed resource additions.
- Prepare documents and the entire candidate under the existing exclusive lock before their publication, with verified asset copies staged speculatively and rolled back on any rejection to preserve existing font resolution. Commit accepted edits and migration in one existing journal transaction.
- Add legacy-project rejection, successful migration, undo/redo/reopen, transport, and transaction-failure regression evidence.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `animation-presets`: restate and cover existing transactional scenarios without new semantics. This correction implements the existing transactional requirement. Its delta repeats that approved requirement with regression scenarios for migration; it adds no request, error, persisted schema, artistic, or renderer semantics.

## Non-goals

No issue-46 implementation, changes to preset parameter semantics, additional preset catalog entries, general transaction redesign, or new storage backend. Ordinary reads and non-preset edit behavior remain on their existing migration path.

## Impact

Core store preparation and asset migration publication, core/headless regression tests. Public catalogs, schema 29, compiler version 1, error classifications, aliases, and lock/journal ownership remain unchanged. No breaking contract change or new dependency edge is intended. The user authorized correction after the finding; independent review must approve these concrete artifacts before code.
