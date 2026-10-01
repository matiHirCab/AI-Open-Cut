## ADDED Requirements

### Requirement: Portable existing extended visual conformance
The existing issue43 implementation MUST pass strict workspace Clippy on supported CI platforms without warning suppression and preserve fixed-width RGBA decoding. Identity effects and zero rotation MUST retain verified legacy placement and colors. Nested inherited rotation and compound path, gradient, crop and ordered effects MUST retain shared evaluated semantics across frame, audiovisual range, draft and export on supported FFmpeg backends. Existing SSIM >= 0.99, aligned PCM RMS <= 0.0001 and timing <= one frame MUST remain unchanged. No public, schema27, audio, revision, rollback, path, sanitization or resource limits MUST change.

#### Scenario: Decode validated RGBA without lint failure
- **WHEN** a correctly sized RGBA buffer is decoded under strict supported Clippy
- **THEN** fixed-width channel groups preserve exact input bytes and linear-premultiplied semantics without warnings

#### Scenario: Reproduce and correct Linux identity and sampled parity
- **WHEN** native identity, nested sample200 and compound sample0 fixtures execute with explicit supported FFmpeg/FFprobe and reviewed dependencies
- **THEN** independent legacy controls and expected semantics pass unchanged thresholds across every required intent

### Requirement: Isolated reviewed corrective delivery
Corrections MUST remain separate from issue44 and MUST use the latest verified PR131 head. Required formatting, Clippy, workspace, affected contracts, real render, cache, protected spec and integration checks MUST be recorded accurately before publication. Failed, skipped and unavailable checks MUST remain distinguishable. Native fixture mocks MUST NOT count as parity evidence. Publication MUST use an ordinary non-force update only to PR131's existing branch, preserve concurrent work and await terminal latest-head CI. No merge, auto-merge, deployment, security change, tolerance relaxation, golden recapture or CI bypass MUST occur.

#### Scenario: Deliver only reviewed corrections
- **WHEN** corrective commits are ready and the remote base is rechecked
- **THEN** only issue43 fixes update PR131 and issue44 remains independently preserved and unpublished
