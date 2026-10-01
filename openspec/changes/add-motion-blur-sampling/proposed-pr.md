# Proposed PR title

[MG-M3-07] Add inherited motion-blur sampling

# Proposed PR body

An animated visual leaf can now author `motionBlur: { shutterAngleDeg, sampleCount }`
through existing standalone and alias-aware batch edits. Centered deterministic
midpoint samples evaluate crop, paint, effects, complete inherited transforms and
activity on canonical clocks, then average premultiplied linear-light canvas rasters
before the existing stacking pass. Preview frames, audiovisual ranges, drafts and
export share that preparation; audio processing stays unchanged.

Editor-core owns finite numeric bounds, 16-sample and cumulative work limits,
unsupported-target rejection, schema28 adoption of current state and retained
history, and atomic failures. Additive bridge/headless catalogs advertise
`motion_blur_sampling_v1`; omitted/disabled settings preserve instantaneous behavior.
See `docs/motion-blur-sampling.md` and the approved OpenSpec change for the exact
coordinate, timing, ordering, clipping and fallback policy.

Depends on PR131. This local implementation is stacked on pinned issue43 head
`e1010bb97174d44a5d19c61a90514bf2336845ca`; PR131 is not merged or rewritten.
References #44. Publication is not approved yet.

Validation: pinned Rust format/strict Clippy and 779 workspace tests pass; bridge
427 unit, 360 contract, 14 integration and 8 packaged smoke tests pass. Real
FFmpeg 7.1 passes all 55 animation fixtures, reviewed native goldens, native
cache/worker reuse and geometry/font/report checks. Packaged smoke is mocked.

**Not merge-ready:** official Ubuntu FFmpeg 6.1 reproduces the base's three parity
failures plus two issue44 conformance failures. Rust 1.98 has an inherited lint
failure. Required rules-screen coverage is incomplete, Moon's GHCR download is
blocked, and CODEOWNER review/archive are pending. Full commands, failures,
performance observations and follow-up steps are in this change's `verification.md`.
Do not close #44 until those requirements are satisfied.
