## Why

PR229 head97db1db3 fixes ambient FFprobe usage but introduces macOS owned-header rejection. Controlled ancestor-alias reproduction confirms a mismatch between unresolved producer paths and canonical probe identities.

## What Changes

Canonicalize newly created private test roots before paths/hash keys in backend regression setup and combined workflow. Add cross-platform alias coverage; preserve strict probe and separate hermetic/genuine proof.

## Capabilities / Impact

Modify durable-production-requests hermetic acceptance: add an owned-root ancestor-alias scenario. Only two private TypeScript test files and specs/evidence change; no public/persisted/runtime contract or migration.

## Non-goals

No production/native lifecycle/disposal, shared probe/backend, dependencies, workflows, profiles, timeouts, skips or weaker negatives. Preserve archives/failed CI. Windows disposal remains restricted. No PR approval, merge/deploy, readiness change or denied description/comments.
