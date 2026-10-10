## Why

#79 exact-head standard CI reports the Clippy1.99 chunks_exact_to_as_chunks lint, which rejects constant-sized chunks_exact in the private PCM oracle despite local Rust1.97 passing. Required correctness must pass without suppressing warnings.

## What Changes

Use fixed-array as_chunks iteration in the private native PCM comparison, retaining exact byte-length, finite, alignment and RMS assertions. No production behavior, media fixtures, thresholds, workflow or toolchain pin changes.

## Capabilities

### Modified Capabilities
- `motion-release-gates`: Preserve native PCM oracle semantics while conforming to strict supported CI lint.

## Impact

Only private native test source and OpenSpec/evidence. Standing issue-scoped authorization approves this bounded correction before implementation. No contract/schema/migration/renderer change.
