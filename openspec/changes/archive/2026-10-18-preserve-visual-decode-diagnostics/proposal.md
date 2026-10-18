## Why

Issue #177 identifies a reproducible source-level defect: visual decode read/no-frame errors return before collected child status and stderr are attached. Preserve that evidence so native failures can be investigated without exposing private paths.

## What Changes

- Distinguish spawn, missing pipe, read, incomplete RGBA frame, empty output, wait and child-exit failures with fixed safe explanatory messages.
- Retain the bounded stderr fragment on collection errors and retry interrupted reads. Give failed-output EOF a bounded 100ms exit-status grace, then terminate/reap any still-live producer.
- Reap each successfully spawned decoder and retain available exit status and bounded sanitized stderr on failures, including empty and incomplete output.
- Preserve successful last-complete-frame selection, bounded memory and failure publication behavior.
- Add deterministic reader/process-adapter regression tests and a synthetic native 720p30 boundary control.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `rendering-export`: preserve actionable compatible diagnostics and lifecycle safety for visual decoding failures.

## Impact

Only editor-core render-process internals, regression fixtures/tests and diagnostic documentation. Existing `FFMPEG_FAILED`, nonretryability, `visual_decode` stage, exit-code and stderr fields remain the public contract. Messages become more explanatory; no new fields, codes, operations, persisted schema, dependencies, migrations or canonical catalog changes. No changes to speech, narration or #208 files/scope.

## Non-goals

Do not assert or fix the unknown cause of the private Windows #177 recording, access private media, alter decoder seek/filter semantics, change render budgets, implement #213 playback, merge or deploy. The original issue stays open for affected-runtime reproduction even if the diagnostics slice passes.

## Approval

The delegated user instruction explicitly authorizes approving issue specifications and implementing independent work. Approved within that delegated authority after reviewing proposal, delta, design and tasks, before implementation. This approval covers only the diagnostic slice above.
