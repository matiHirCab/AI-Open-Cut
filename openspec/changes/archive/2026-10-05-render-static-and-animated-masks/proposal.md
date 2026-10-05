## Why

Issue #51 explicitly rasterizes static masks and animates path, feather, expansion and transform. Verified #50 schema 32 painted path masks supply typed storage while rendering identity. Closed prerequisites #38/#39 provide typed channel values, source clocks, curves and deterministic interpolation. Activate those models through the same evaluated sampling and local premultiplied pipeline in every production intent.

## What Changes

- Rasterize canonical filled path coverage, expand/contract geometric support with a bounded alpha-aware grid signed-distance algorithm, apply original-coordinate paint and alpha/luma extraction, Gaussian feather, mask affine transform/opacity, inversion and declared scalar combinations before existing effects.
- Add closed mask-targeted path, solid color/gradient-stop, feather/expansion and all Transform2D scalar channels through existing set_animation_channels/edit/batch/draft surfaces; preserve static fallback, inherited fractional clocks, integer endpoint selection, curves and loops.
- Advance schema 32 to 33 for new channel/target variants and explicit static-mask rendering activation; atomically validate/migrate current state, retained history and drafts. Existing schema 32 masked visuals intentionally acquire their authored mask appearance; no-mask output remains unchanged.
- Add editor mask_animation_v1 and readiness-gated renderer mask_rendering_v1 with contracts/mask-rendering-v1.json and synchronized canonical/native/TypeScript/MCP evidence.
- Synchronize the six existing active animation catalogs and mask-models-v1 current markers32→33 with governed consumers; pin exact verified #50 semantic predecessors and preserve the existing six-catalog32→31 proof through composed projections.
- Amend #50 identity-only/current-future statements rather than leaving contradictory living requirements; document renderer work/memory/precision rejection and deliberate activation compatibility.

## Capabilities

### New Capabilities

- mask-rendering: Deterministic painted path coverage, bounded expansion/feather, shared sampling/composition and readiness.

### Modified Capabilities

- mask-models: Replace inactive-rendering boundary with explicit schema 33 activation and refine deferred expansion/paint raster order.
- animation-channels: Add closed mask targets/properties and exact bounded sampling rules without widening existing value/curve/count contracts.
- project-persistence: Fully modify/rename #50 current migration/draft requirements for atomic schema33 adoption, preserve before32 mask guards, add before33 channel guards and staged channel-only requests.
- rendering-export: Replace metadata-identity requirement with shared mask semantics while preserving no-mask output.
- linear-light-compositing: Activate the previously identity mask stage, preserving unrelated source-over/effects/transform/matte/blend math and bounds.
- motion-graphics-architecture: Update supported/current pipeline distinction without activating mattes or non-normal blends.
- contract-governance: Add separate authoring/render readiness capabilities, model activation annotations and exact reviewed additive digest transitions.

## Non-goals

No source-layer masks, mattes/DAG, new blend/effect kinds, raw SVG/path/expression resources, mask topology animation, path trim, inversion/channel/operation/unit animation, gradient geometry animation, desktop inspector or provider changes. No unbounded neighborhood-distance search or unbudgeted Gaussian radius loops. Existing #50 model limits, existing scalar/compound tags, 64-channel/1000-keyframe limits, root/component item addressing and protocol major 1 remain.

## Impact

New property/target identifiers and capabilities are additive on existing operations/protocol major 1; schema 33 is an explicit persisted transition. Strict older clients/binaries pinned to32 require upgrade. Schema 32 mask output was identity and intentionally changes upon activation; older valid metadata that exceeds render precision/work/bounds can now fail atomic adoption with stable INVALID_ARGUMENT and unchanged prior bytes, rather than be clipped/repaired. Unmasked source/effects/geometry/clock/audio output and budgets remain unchanged. Core owns validation, sampling, rasterization and migration; transports only expose typed input/results. Explicit delegated parent/reviewer specification approval precedes implementation under the user’s authorization. Prepare complete contract evidence for designated @matiHirCab CODEOWNER review in the draft PR; human review remains pending and is not represented as delegated approval or a completed merge gate. This external draft is reconciled to verified #50 commit 62c30eb224d36e0b0a73c88362e4423555a084f7 (draft PR143) and its schema32 current-marker amendment. Its expanded MCP predecessor is 88b55ff7be147cb4aadc016dd92f92342c3dc366f49ac139b8116513d5854830. Promotion/review/approval remain parent-owned; #50 is not claimed merged.
