## Why

#213 requires reviewing existing generated range artifacts without leaving desktop. The current panel only reveals an MP4 for external playback. Dependencies are ready; #208 narration/audition and #225 decoder diagnostics are separate owners and are not included.

## What Changes

- Embed play/pause, bounded seek, elapsed/captured-project time and loop of an existing completed range.
- Support Linux with an optional installed libmpv 2 client/software-render API. Missing backend and other platforms explicitly retain the existing external-file fallback.
- Retain immutable captured metadata/stale labels, bound presentation buffers and release workers, native handles and GPUI image resources on replacement/close.
- Add deterministic controller tests, real native video/audio/seek/loop tests and actual GUI evidence on the supported backend.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `desktop-review-surfaces`: embedded presentation-only playback of owned range review artifacts.

## Impact

Only desktop review state, preview panel, a private nested playback module and dedicated tests/fixtures/documentation. No core renderer, speech/narration, shell/session ownership, canonical MCP/headless catalogs, persisted project, dependency manifest/lockfile, security or deployment changes. No arbitrary path or URL entry. Playback is optional runtime readiness, not a new renderer or timeline playback engine.

## Approval

Approved under the delegated maintainer authority before executable edits. #213 and comments/PR state were refreshed; no hard dependencies or competing implementation were found. Capability probes built the unchanged desktop, captured its real GUI under X11/GPUI/Vulkan software rendering and captured 88,200 bytes of actual libmpv audio with nonzero signal through an ALSA file sink. These are environment probes, not implementation acceptance. Only issue-scoped changes below are authorized.
