## ADDED Requirements

### Requirement: Shared advanced text layout and fit diagnostics
Frame preview, audiovisual range preview, materialized draft preview and final export MUST consume the same core evaluated advanced text layout after animation, component/slot substitution and repeater expansion. Backends MUST NOT independently select font size, wrap or align text. For layout-enabled text, render diagnostics MUST expose an additive `textLayouts` array in evaluated paint order, each entry containing `itemId`, `resolvedFontSize`, `lineCount`, `contentWidthPx`, `contentHeightPx`, `overflowX` and `overflowY`; itemId MUST be the existing effective evaluated identity, including instance/repeater identity. Dimensions MUST describe the measured line-box block excluding padding/effects. Legacy-only output MUST omit the new field. Equivalent requests MUST yield identical layout diagnostics/glyph plans, decoded visual SSIM at least 0.99, decoded float-PCM RMS error at most 0.0001 and timing alignment within one output frame. Half-open timing, keyframes, background/paint order, transforms and managed resource/path safety MUST remain effective. Resource failures MUST retain existing typed errors; no intent MUST silently substitute fonts or discard layout to complete a render.

#### Scenario: Verify layout across every render intent
- **WHEN** the same styled, animated, transformed text with components, slots and repeaters is rendered as frame, range, materialized draft and final export using each fit mode
- **THEN** shared glyph plans and diagnostics agree and decoded audiovisual results satisfy the existing tolerances

#### Scenario: Reopen without original fonts and fail safely
- **WHEN** original font files are removed before reopen or managed resources become missing/corrupt or layout exceeds limits
- **THEN** retained valid fonts reproduce the same layout and failures retain established errors without unsafe paths, output artifacts or renderer-specific fallbacks
