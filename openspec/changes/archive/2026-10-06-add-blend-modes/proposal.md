## Why

Issue #53 requests normal, multiply, screen, overlay, add, darken and lighten as public persisted choices with common preview/export semantics. The verified #49 pipeline performs linear premultiplied source-over over an initially opaque black canvas. The verified predecessor is #52 commit65e89d637d882d56aa98735aae9374de6a0da9fe, with all11CI jobs passed in run37458611299. Blend selection must not introduce a new background interpretation or sampled destination dependency into matte providers.

## What Changes

- Add a closed BlendMode enum and default-normal VisualProperties.blendMode, edited through optional nonnullable update_item and existing component payloads.
- Apply non-normal W3C-style separable blending in premultiplied linear working color at final destination composition, preserving the exact current normal function and opaque black initial accumulator.
- Include every actual visual source leaf, including Caption; multiline text and caption glyph/background fragments assemble into one owning occurrence plane before one blend operation. Reject non-normal on audio/structural/transition items without adding group isolation.
- Preserve masks/effects/transforms/mattes/opacity and per-occurrence shutter averaging before destination blending, provider-plane independence from blend, exact normal/default output and unchanged audio/time semantics.
- Add checked blend work accounting within existing source/destination/live-memory preflight, canonical model/formula/eligibility/public fixtures and capability readiness.
- Migrate verified schema34→35 current/history/drafts atomically with source-envelope guards; synchronize headless/MCP fields, docs, ownership and narrowly reviewed digest/schema literal changes.

## Capabilities

### New Capabilities

- `blend-modes`: strict selection, eligible visual occurrences, seven blend equations, bounded shared destination composition and independent evidence.

### Modified Capabilities

- `project-persistence`: schema35 default-normal migration and raw/source-matched draft guards.
- `contract-governance`: canonical blend fixtures, strict additive fields, model/render capability and digest transition.
- `rendering-export`: shared evaluated blend selection for frame/range/draft/export and matte provider independence.
- `linear-light-compositing`: explicit destination blend formulas and unchanged normal/opaque-black default.
- `motion-graphics-architecture`: final destination-stage activation and applicability semantics.
- `repository-validation`: preserve complete mandatory track-matte CI conformance and add actual native blend core/default-headless/MCP execution with exact additive policy controls.

## Impact

Core model/timeline/validation/component/store/migration, evaluated_scene/render_plan/render_artifact/raster cache owners; typed headless/MCP/bridge contracts, capabilities/docs and canonical ownership consumers. No new operation, blend animation property, raw FFmpeg expression, HDR/additive-alpha rule, transparent project canvas or isolated-group compositing surface. Branch feat/issue-53-blend-modes-20261006 starts from that exact verified predecessor. Canonical pins are recorded in verified52-catalog-pins.json; approval and strict validation precede implementation.
