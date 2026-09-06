## Why

Review of issue #24 reproduced a schema-12 project with a nested instance using legacy scale 2 being accepted and rewritten as schema 13. That transform was forbidden in the source schema, violating the existing requirement to reject invalid old-schema content without publication.

## What Changes

- Validate source-version component constraints before migration relabels current or retained snapshots.
- Reject non-default nested legacy transforms in schemas 11–12 with INVALID_ARGUMENT, including hidden and unused definitions.
- Preserve valid legacy migrations, schema-13 transforms and atomic failure behavior, with regression coverage.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-persistence`: Make source-schema transform rejection and preservation scenarios explicit under atomic schema-13 activation.

## Impact

Core validation, migration and persistence tests only. This restores an existing documented constraint; no public API, persisted shape, schema version, capability, error code or retryability changes. No cross-language declaration changes or new dependencies are needed.

## Non-goals

No rendering, editing, provider or UI changes; no broader migration redesign. Preserve the original archived change and unrelated working-tree work.

## Approval

The user explicitly approved these proposal, design, delta specification and task artifacts with "Approve" in this task on 2026-09-06. Implementation and verified archival are authorized by the requested plan.
