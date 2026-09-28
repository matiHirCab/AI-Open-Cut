## Why

The schema-23 curve implementation accepts valid cubic Bézier and spring channels, but a native Bézier visual render fails on the retained FFmpeg 6.1.1 build while the same test passes on FFmpeg 8.1.2. The render-plan compiler also rounds curve constants to six decimal places; a valid near-critical spring then renders about 0.03369 away from the editor-core sample at normalized time 0.5. These defects break the approved deterministic curve and shared-render requirements and leave the required Linux FFmpeg render gate at risk.

## What Changes

- Serialize FFmpeg filter-graph evaluation only for scenes with a cubic Bézier curve, because FFmpeg 6 races on the curve expression's mutable registers. Retain exactly 40 bisection steps and equivalent results with FFmpeg 8.
- Preserve sufficient finite `f64` precision when writing parameterized curve constants and their segment values into FFmpeg expressions, without changing legacy curve output.
- Add independent scalar and decoded-media evidence for Bézier boundaries, all spring damping regimes, near-critical values, and matching draft, frame, range, and export results, including audio gain.

**Non-goals:** New curve types or parameters, changes to validation limits, legacy `set_keyframes` easings, schema migration, public request/response or MCP shapes, capability identifiers, FFmpeg 9 compatibility, or unrelated golden-reference refreshes. No breaking contract change is proposed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `animation-channels`: Make the existing `1e-9` fixed scalar tolerance explicit at the numeric-to-render expression boundary, including valid near-critical springs.
- `rendering-export`: Require supported FFmpeg 6 and 8 backends to render the same evaluated parameterized curves across all output intents, with decoded-media evidence rather than only plan equality or single-pixel presence.

## Impact

Affected implementation is confined to `crates/editor-core` render planning, process command construction, and focused render tests. Bézier scenes may render more slowly because their FFmpeg filter graph uses one thread; scenes without Bézier retain their current thread policy. The existing schema-23 persisted model, migrations, Rust/TypeScript/headless/MCP contracts, and stable errors stay unchanged. The protected Linux native render-parity step already runs `animation_channels` with FFmpeg; it will exercise the new regression evidence without changing the closed workflow. Changes to the canonical golden baseline, if genuinely necessary, require separate provenance and approval.
