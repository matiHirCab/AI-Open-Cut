# Proposed PR title

[MG-M3-07] Add motion-blur sampling over inherited transforms

# Proposed PR body

Animated visual leaves can author `motionBlur: { shutterAngleDeg, sampleCount }`
through existing standalone `update_item` and alias-aware batch edits. Centered
midpoint samples use authored project FPS and canonical inherited clocks, then
average premultiplied linear-light canvas rasters after crop/clip/paint/effects and
full affine/transition evaluation, before existing stacking. Frame, audiovisual
range, draft and export consume the same preparation; audio stays unchanged.

Editor-core owns finite angle bounds, the 16-sample and 268435456 weighted-pixel
limits, unsupported-target errors, optimistic revisions, atomic batch rollback,
undo/redo and schema 28 adoption of current state plus all retained history/drafts.
Additive protocol 1 catalogs advertise `motion_blur_sampling_v1`. Omitted, zero-angle
and single-sample controls retain deterministic instantaneous output. The inherited
instantaneous fallback reuses the private expression-thread guard for graphs that
emit mutable FFmpeg registers. No raw expressions, paths or resources are exposed.

Based on main `b2be8f219743639db106330f5839a2efcf77c053`, containing the merged
issue43 implementation and corrected head `81a6ec6` (all 11 exact-head CI jobs
passed; 49.03 minutes within the unchanged 120-minute budget). Original issue44 commits are
preserved. References #44.

Validation passes: Rust format, workspace tests and strict Clippy 1.97/1.98;
bridge type/lint, 185 Rust + 360 TS contract fixtures, 427 unit, 14 MCP integration,
8 packaged smoke and 15 hermetic Python cases; actual FFmpeg 6/7 animation 55 each,
cache core/worker/bridge 13/3/1, native lifecycle and geometry/font 29. Additional
actual-headless 22-request reproduction has independent pixel MSE 0, SSIM >0.9997,
unchanged PCM RMS 0, exact durations, equal draft pixels and byte preservation.
The unchanged seven-corpus golden run and strict captured-report validator pass.
The loaded integration timeout is retained alongside the passing complete idle
rerun; timeout 60 seconds, tolerances, golden references and CI policy are unchanged.
Packaged smoke mocks FFmpeg and supplies packaging evidence only.

The complete unchanged PR rules-screen matrix passes locally: 25 / 25 / 6 actual
renders at 960 / 1280 / 1920, with all five lifecycle states at each resolution.
This is Linux evidence; issue44 remote CI has not been run.

**Pending before merge readiness:** designated `@matiHirCab`
implementation review, OpenSpec synchronization/archival and the protected final
gate remain required. This is local proposed text; publication is separately
unapproved. See `docs/motion-blur-sampling.md` for exact timing, coordinate, ordering
and fallback policy, and the change's `verification.md` for commands, failures,
performance observations, limits and complete evidence.

## Superseding archival status

The proposed text above is historical. PR #132 is published in draft; explicit human CODEOWNER implementation approval is recorded in approval.md. OpenSpec is synchronized and archived with 21/21 tasks; final local protected checks pass as recorded in verification.md. Remote CI for the archival head is a separate observation.
