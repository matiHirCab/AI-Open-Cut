## MODIFIED Requirements

### Requirement: Complete render preflight before destination inspection
All renderer entry points MUST finish read-only geometry preflight before export-collision inspection, temporary-name allocation, workspace creation, or writes. Missing references and invalid pure values MUST fail before resource probing. Measured text overflow MUST fail before metadata process calls. Bounded FFprobe metadata inspection is permitted after reference/value/text validation and before affine finalization; it is distinct from render execution. For static images this permission explicitly includes bounded first-frame decoding solely to inspect dimensions and orientation, without raster artifact production, workspace creation, or resource writes. Materialization MUST reuse finalized geometry and measured resources.

#### Scenario: Preserve overflow error precedence
- **WHEN** transformed text exceeds geometry bounds with either an absent or an existing export destination
- **THEN** rendering returns INVALID_ARGUMENT without collision inspection, temporary allocation, writes, or process calls

#### Scenario: Preserve valid export collisions
- **WHEN** a valid preflight targets an existing destination without overwrite permission
- **THEN** export returns EXPORT_EXISTS without workspace creation or writes

#### Scenario: Render oriented media across intents
- **WHEN** asymmetric rotated media uses identity or combined noncentral Transform2D in frame, range, draft, and export
- **THEN** orientation and complete source extent are preserved, SSIM is at least 0.99, audio RMS is at most 0.0001, and timing differs by at most one frame

#### Scenario: Preserve EXIF images across render intents
- **WHEN** quarter-turn and reflected static images use noncentral anchors and combined Transform2D in frame, range, materialized draft, and export
- **THEN** complete oriented source extent is preserved, visual SSIM is at least 0.99, timing differs by at most one output frame, and draft rendering leaves project and history unchanged
