## ADDED Requirements

### Requirement: Strict-lint-compatible native mask identity witnesses
The native mask-model metadata identity conformance test MUST preserve the existing independent visible-red RGB witness, nonzero decoded float32 PCM witness, exact decoded-length checks, thresholds and frame/range/draft/export pixel/audio/timing identity assertions. Fixed-width RGB and PCM iteration MUST use a supported typed complete-array API compatible with the pinned toolchain and strict CI Clippy, without warning suppression or broader assertion/behavior changes.

#### Scenario: Preserve independent witnesses under strict Clippy
- **WHEN** required native mask conformance executes with actual FFmpeg and FFprobe and strict workspace Clippy checks its test source
- **THEN** RGB complete triples and PCM complete four-byte samples retain identical ordered interpretation and thresholds, all native identity assertions pass, and no lint is suppressed
