## Why

Independent review of issue #35 reproduced invalid advanced layouts reopening and rendering from root text and retained history, and malformed retained draft layouts returning INTERNAL_ERROR instead of INVALID_ARGUMENT. The previous two archives do not implement these remaining corrections.

## What Changes

- Validate layout-enabled root text in the shared scope validator, including hidden text, current state, retained undo/redo and direct renderer inputs.
- Classify structural layout decoding errors with a private reserved marker and return non-retryable INVALID_ARGUMENT without changing unrelated persisted-data error classification.
- Add persisted-file, preflight and headless/MCP regression coverage for numeric and structural failures and compatibility controls.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `advanced-text-layout`: Complete persisted root/history validation and precise structural error classification across retained text locations.
- `rendering-export`: Reject malformed root layouts before output inspection, allocation or execution.

## Impact

Core model deserialization, error translation and shared validation, with persistence/renderer/native transport and bridge tests. No public fields, operations, capabilities, dependencies, schema version or encoding settings change. This restores the existing INVALID_ARGUMENT contract; valid persisted data and requests remain compatible. Preserve both existing issue-35 archives and unrelated working-tree changes.

## Non-goals

No layout-absent legacy validation expansion, stale-draft replay, fitting or geometry changes, numerical-limit changes, schema migration, generic persisted-error reclassification, renderer fallback or new public configuration.

## Approval

The user requested implementation of the plan on 2026-09-20 and explicitly approved these concrete proposal, design, delta specification and task artifacts with "yes". Implementation is authorized through tasks.md.
