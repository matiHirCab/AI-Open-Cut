## Why

Review of issue #33 reproduced three conformance defects: selector-key collisions overwrite newly resolved draft bindings, operation insertion loses retained bindings, and Unicode mandatory breaks render on one line.

## What Changes

- Retain bindings by unresolved scoped text identity and match draft operations by structure then font intent across updates.
- Preserve fonts across non-font edits; reject ambiguous matches atomically with INVALID_ARGUMENT.
- Honor mandatory Unicode separators with correct paragraph bidi context and original byte clusters.
- Explicitly approve a narrow rendering correction within opencut-text-v2 for separator-containing text.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `font-resolution`: stable draft matching and mandatory-break conformance.

## Impact

Core assets/store/shaping and their regression tests; text-layout documentation. No public field, persisted format, dependency or architecture edge changes. Schema 19 and draft/layout version 2 remain unchanged. The separator correction changes previously incorrect rendered output and is explicitly authorized by the user-approved plan.

## Non-goals

No operation-ID API, new layout profile, general draft redesign, font import service, or changes to unrelated active work.
