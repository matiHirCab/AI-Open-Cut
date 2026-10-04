## Why

PR141 at c030f4f failed its required native CI: all six requested-origin audit tests rejected configured `ffmpeg` and `ffprobe` command names because their helper used filesystem `is_file()` instead of command readiness. The protected workflow explicitly selects PATH commands;120 other native animation tests passed.

## What Changes

- Make the requested-origin test dependency gate accept usable configured command names and executable paths through the existing canonical Renderer readiness check, preserving readable-file font validation and required-mode failure.
- Exercise the actual full native animation suite with bare commands and both required flags; add focused unusable-command/font controls without changing process environment globally.
- Preserve failed CI evidence, reverify the amended exact published head and maintain honest approval and input-bound check reuse records.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: clarify usable explicit executable configuration in the requested-origin audit gate.

## Impact

Only `crates/editor-core/tests/epic6_requested_origin.rs` executable test helper/controls and specification/delivery documentation. Compatibility is a test-harness correction: production code, public API/error identities, schema31, persistence/migrations, reference data, budgets, codecs and output equations remain unchanged. No new dependency.

## Non-goals

No renderer or workflow change, automatic dependency discovery, fallback executable/font substitution, required-test skipping, timeout/tolerance widening, golden update, merge, deployment or issue49 implementation.
