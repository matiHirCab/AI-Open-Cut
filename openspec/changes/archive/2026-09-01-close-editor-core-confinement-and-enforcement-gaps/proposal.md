## Why

PR #94 still probes project marker files before proving that the requested directory remains inside the configured project root, and its architecture analyzer does not canonicalize block-local `extern crate` or type aliases or reject `Path::is_symlink`. These gaps violate the already-approved confinement and dependency-enforcement guarantees while the existing tests remain green.

## What Changes

- Validate a project's canonical directory before marker-file probes, locking, persisted reads, recovery, or garbage collection, while preserving `PROJECT_NOT_FOUND` for ordinary missing projects.
- Apply the existing alias-resolution rules to block-local `use`, `extern crate`, and path-based type aliases.
- Treat `Path::is_symlink` as direct filesystem inspection in owners required to delegate I/O through private adapters.
- Add deterministic regressions for empty external project targets, missing and valid projects, block-local aliases, test-only aliases, and method/UFCS filesystem calls.
- Preserve public APIs, persisted representations, normal error and warning behavior, FFmpeg behavior, and ADR 0003's dependency matrix.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `editor-core-architecture`: make project-root rejection precede project existence probes and apply canonical architecture enforcement to block-local aliases and the complete supported native `Path` inspection surface.

## Impact

- Affected code: private editor-core store confinement and the development-only architecture analyzer.
- Affected tests: fake-storage ordering and errors, Unix linked-directory behavior, and parsed-Rust architecture regressions.
- Compatibility: no public API, serialization, dependency-matrix, or valid-runtime behavior changes; invalid canonical escapes consistently use the existing `PATH_NOT_ALLOWED` code.
- Dependencies: no new production or development dependencies.

## Non-goals

- Implementing security/stress release gates, SVG ingestion, `EvaluatedScene`, rendering features, artifact retention, cancellation, or durable jobs assigned to other issues.
- Changing transaction recovery, asset-reference discovery, garbage-collection policy, ADR 0003, or the public injection surface.
