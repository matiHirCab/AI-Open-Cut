## Why

PR #94 now routes persistence and render artifacts through private adapters, but project enumeration follows directory symlinks and the architecture analyzer still misses nested `cfg_attr` paths, standalone test items, legal crate-root aliases, and filesystem calls expressed as `Path` methods. These gaps can load or garbage-collect a project outside the configured root and can let forbidden owner responsibilities pass while CI remains green.

## What Changes

- Preserve project-root confinement by classifying directory entries without following symlinks, excluding linked project directories, and validating canonical project paths before lock, read, recovery, or garbage collection.
- Route the remaining project-asset canonicalization through the injected storage adapter.
- Recursively inspect production-relevant `cfg_attr` metadata and exclude only genuinely test-only items.
- Canonicalize leading `self`/`super` forms and aliases introduced by `extern crate`, including `extern crate self`.
- Detect direct filesystem access expressed through `Path` methods while keeping private persistence and artifact adapters distinguishable and permitted.
- Preserve public APIs, serialized data, normal error/warning behavior, FFmpeg behavior, and ADR 0003's dependency matrix.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `editor-core-architecture`: close project-root and parsed-Rust enforcement gaps without changing the allowed ownership graph.

## Impact

- Affected code: private persistence/store path handling and editor-core architecture-test internals; private adapter method names may change to make enforcement unambiguous.
- Affected tests: project listing/confinement, fake-storage behavior, and architecture syntax regressions.
- Compatibility: no public or persisted contract changes; paths escaping `projects_root` are rejected with the existing `PATH_NOT_ALLOWED` code.
- Dependencies: no new production or development dependency.
