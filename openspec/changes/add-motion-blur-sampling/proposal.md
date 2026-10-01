## Why

Issue #44 requires deterministic shutter sampling of fully inherited visual transforms. Issue #43 provides the canonical evaluated scene and sampled raster foundation; disconnected renderer approximations would break parity.

## What Changes

- Add bounded optional per-leaf motionBlur settings through existing update_item and alias-aware batch operations.
- Average deterministic shutter samples in premultiplied linear light after each sample's crop, paint, ordered effects and complete inherited affine, before existing layer stacking.
- Govern finite shutter angle, sample count, timing, fallback and cumulative work in editor-core; migrate schema 27 current state and all retained history atomically to schema 28.
- Synchronize typed contracts, support reporting, fixtures, documentation and native visual/audio conformance evidence.

## Capabilities

### New Capabilities
- `motion-blur-sampling`: Authored shutter settings, inherited sampling, deterministic bounded rendering and lifecycle conformance.

### Modified Capabilities
None; the new capability retains existing lifecycle and rendering requirements.

## Impact

Canonical owner: crates/editor-core model, validation, mutation, migration, EvaluatedScene and raster preparation. Consumers: headless, agent bridge, desktop declarations, contracts and CODEOWNERS. Additive protocol-1 fields and capability motion_blur_sampling_v1; persisted schema 28 requires deterministic atomic adoption. No breaking public operations or error changes.

## Non-goals

No new animation channels, shutter curves, stochastic jitter, audio processing, group flattening, resources, FFmpeg expressions, network access or UI redesign. PR131 is an unmerged dependency; never merge or rewrite it. Do not publish this branch without separate user approval.
