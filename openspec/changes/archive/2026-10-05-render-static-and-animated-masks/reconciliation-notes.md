# Issue51 reconciliation against verified issue50

External preparation only: no promotion, approval or implementation. Verified source is 62c30eb224d36e0b0a73c88362e4423555a084f7 (draft PR143). Current expanded MCP SHA256 is `88b55ff7be147cb4aadc016dd92f92342c3dc366f49ac139b8116513d5854830`.

## Required schema33 catalog changes

All seven current projectSchemaVersion markers advance32→33. animation-presets, extended-visual-animation, inherited-animation-timing, motion-blur-sampling and initial-motion-preset-pack change only that marker. animation-channels additionally receives exactly the fifteen approved mask properties and typed mask-target/applicability/activation records; mask-models additionally receives only explicitly declared active-renderer annotation and executable expansion/paint-order metadata. Historical source fixtures remain unchanged. New mask-rendering-v1.json owns raster/channel-subset/budget/fixture semantics and references existing authoring/vector/Transform2D/global animation owners.

The seven immediate predecessor pins and exact JavaScript localeCompare/all-fields semantic codec are recorded in design.md and predecessor-catalog-digests.json. The two feature-bearing catalogs require an enumerated projection restoring their exact approved added/changed fields as well as the marker; marker-only projection is insufficient. Five others restore only top-level33→32. Six existing animation catalogs then compose the preserved32→31 proof. There is no invented pre50 mask-model catalog predecessor. MCP projection must separately enumerate all new mask enum/target storage members, two capabilities, and only the existing exact status/project schema literals before edits; preserve33→32→31 proof and reject unrelated drift.

## Resolved reconciliation findings

- Full contract-governance MODIFY now carries the complete issue50 current-marker requirement and scenario, advanced to seven schema33 catalogs rather than omitting the prior six-marker amendment.
- Project-persistence fully RENAMES/MODIFIES both actual issue50 requirements, avoiding contradictory current32 requirements. It preserves premature source1…31 mask rejection (including null/empty), adds source1…32 mask-channel guards, complete retained history/draft atomic adoption and unavailable-base structural/resource validation without invented owner-dependent replay.
- Stored masks use existing exported Mask/MaskSource/MaskChannel/MaskOperation types and reused VectorPath/Paint/Transform2D; empty-array omission and explicit-null rejection remain unchanged. IDs are literal item-local Unicode/@ identifiers, bounded by128 UTF-8 bytes, not creation aliases. Static4096 commands may contain more than4096 coordinate points; unchanged animated compound point cap remains4096.
- Channel-only mask edits must join existing staged draft/publication classifiers and pre-default introduced-field guards, including component payloads. Invalid final candidates cannot publish selector/resource rewrites or migrations.
- Root-selected owner coordinates are exact post-crop/clip raster top-left, logical W/density,H/density and pixel center((x+.5)/density,(y+.5)/density). Unexpanded analytic path bounds determine mask anchor; vector padded origin is never added again. design.md, mask-models and mask-rendering deltas now agree with implementation-interface.md.
- Memory accounting now separates current64P+16(W+H) pixel/line scratch, additive4K kernel taps,4P_owner accumulator and SUM64S over all concurrently retained contours PLUS checked fact headers, IDs, Paint/gradient-stop heap and Vec descriptor/capacity storage, including no-grid/S=0 facts and all stack/scene/shutter samples. Sequential raster processing does not free immutable stack contours. Interface and normative design/spec agree.
- Existing evaluated mask metadata identity tests from32 cannot claim active33 nonempty masks remain identity. Preserve exact no-mask regression and revision-scoped cache admission; implement new sampled mask facts in existing scene owners instead of inventing an existing mask-stripping helper.

## Ownership and consumers

Update contract-ownership-v1.json and .github/CODEOWNERS for new maskRendering canonical catalog/consumers and all new core files. Keep global headless/MCP capability owners: mask_animation_v1 is editor readiness independent of native dependencies, mask_rendering_v1 is fully ready renderer-only, mask_models_v1 remains stored-model support. Required public consumers include apps/headless/src/main.rs, tests/protocol.rs and render_worker.rs; apps/agent-bridge/src/schemas.ts/headless-contract.ts and governed contract/animation/mask fixtures, real MCP and packaged workflows. Current persisted schema owner model.rs and its migrations/store/drafts/persistence guards advance33. New private ownership reservations are specified in implementation-interface.md; promotion approval must precede source edits.

The following lists are the exact existing governed consumer inventories from the verified50 contract-ownership catalog. They identify parity/review obligations, not a claim that every listed file needs semantic edits.

### animationChannels: contracts/animation-channels-v1.json

- `crates/editor-core/src/model/animation_channels.rs`
- `crates/editor-core/src/animation.rs`
- `crates/editor-core/src/validation/animation_channels.rs`
- `crates/editor-core/src/evaluated_scene.rs`
- `crates/editor-core/src/render_plan.rs`
- `crates/editor-core/src/migrations.rs`
- `crates/editor-core/tests/animation_channels.rs`
- `apps/headless/src/main.rs`
- `apps/headless/tests/protocol.rs`
- `apps/agent-bridge/src/headless-contract.ts`
- `apps/agent-bridge/src/schemas.ts`
- `apps/agent-bridge/tests/animation-channels.test.ts`
- `apps/agent-bridge/tests/animation-edit-workflow.ts`
- `apps/agent-bridge/tests/pack-clock-migration-workflow.ts`
- `docs/animation-channels.md`
- `apps/agent-bridge/tests/contracts.test.ts`

### animationPresets: contracts/animation-presets-v1.json

- `crates/editor-core/src/model/animation_presets.rs`
- `crates/editor-core/src/model.rs`
- `crates/editor-core/src/validation/animation_presets.rs`
- `crates/editor-core/src/validation/animation_channels.rs`
- `crates/editor-core/src/timeline/animation_presets.rs`
- `crates/editor-core/src/timeline.rs`
- `crates/editor-core/src/migrations.rs`
- `crates/editor-core/src/store.rs`
- `crates/editor-core/src/drafts.rs`
- `crates/editor-core/tests/animation_presets.rs`
- `crates/editor-core/tests/animation_channels.rs`
- `apps/headless/src/main.rs`
- `apps/headless/tests/protocol.rs`
- `apps/agent-bridge/src/headless-contract.ts`
- `apps/agent-bridge/src/schemas.ts`
- `apps/agent-bridge/src/server/timeline.ts`
- `apps/agent-bridge/tests/animation-presets.test.ts`
- `apps/agent-bridge/tests/contracts.test.ts`
- `docs/animation-presets.md`
- `apps/agent-bridge/tests/preset-workflow.ts`
- `apps/agent-bridge/tests/smoke.test.ts`
- `apps/agent-bridge/tests/packaged-smoke.test.ts`

### extendedVisualAnimation: contracts/extended-visual-animation-v1.json

- `crates/editor-core/src/model.rs`
- `crates/editor-core/src/animation.rs`
- `crates/editor-core/src/evaluated_scene.rs`
- `crates/editor-core/src/render_plan.rs`
- `crates/editor-core/src/render_process.rs`
- `crates/editor-core/src/renderer.rs`
- `crates/editor-core/src/store.rs`
- `crates/editor-core/src/timeline.rs`
- `crates/editor-core/src/model/visual_effects.rs`
- `crates/editor-core/src/model/animation_channels.rs`
- `crates/editor-core/src/validation/extended_visual.rs`
- `crates/editor-core/src/evaluated_scene/extended_visual.rs`
- `crates/editor-core/src/evaluated_scene/extended_certification.rs`
- `crates/editor-core/src/evaluated_scene/text_bounds.rs`
- `crates/editor-core/src/render_artifact/extended_visual.rs`
- `crates/editor-core/src/migrations.rs`
- `crates/editor-core/tests/extended_visual_animation.rs`
- `crates/editor-core/tests/animation_channels.rs`
- `apps/headless/src/main.rs`
- `apps/headless/tests/protocol.rs`
- `apps/agent-bridge/src/schemas.ts`
- `apps/agent-bridge/src/headless-contract.ts`
- `apps/agent-bridge/tests/extended-visual-animation.test.ts`
- `apps/agent-bridge/tests/extended-visual-workflow.ts`
- `docs/animation-channels.md`
- `docs/extended-visual-animation.md`
- `docs/render-regression-fixtures.md`
- `apps/agent-bridge/tests/contracts.test.ts`

### inheritedAnimationTiming: contracts/inherited-animation-timing-v1.json

- `crates/editor-core/src/model.rs`
- `crates/editor-core/src/model/repeater.rs`
- `crates/editor-core/src/evaluated_scene.rs`
- `crates/editor-core/src/migrations.rs`
- `crates/editor-core/tests/inherited_animation_timing.rs`
- `apps/headless/src/main.rs`
- `apps/headless/tests/protocol.rs`
- `apps/agent-bridge/src/headless-contract.ts`
- `apps/agent-bridge/src/schemas.ts`
- `apps/agent-bridge/tests/contracts.test.ts`
- `docs/inherited-animation-timing.md`

### motionBlurSampling: contracts/motion-blur-sampling-v1.json

- `crates/editor-core/src/model/motion_blur.rs`
- `crates/editor-core/src/model.rs`
- `crates/editor-core/src/validation/extended_visual.rs`
- `crates/editor-core/src/timeline.rs`
- `crates/editor-core/src/migrations.rs`
- `crates/editor-core/src/store.rs`
- `crates/editor-core/src/evaluated_scene/extended_visual.rs`
- `crates/editor-core/src/evaluated_scene/extended_certification.rs`
- `crates/editor-core/src/render_artifact/extended_visual.rs`
- `crates/editor-core/tests/motion_blur_sampling.rs`
- `apps/headless/src/main.rs`
- `apps/agent-bridge/src/schemas.ts`
- `apps/agent-bridge/src/headless-contract.ts`
- `apps/agent-bridge/tests/motion-blur-sampling.test.ts`
- `docs/motion-blur-sampling.md`
- `crates/editor-core/src/evaluated_scene.rs`
- `crates/editor-core/src/render_plan.rs`
- `apps/headless/tests/protocol.rs`
- `apps/agent-bridge/tests/extended-visual-workflow.ts`
- `crates/editor-core/tests/animation_channels.rs`
- `apps/headless/tests/render_worker.rs`
- `apps/agent-bridge/tests/smoke.test.ts`
- `apps/agent-bridge/tests/contracts.test.ts`

### initialMotionPresetPack: contracts/initial-motion-preset-pack-v1.json

- `crates/editor-core/src/model/animation_presets.rs`
- `crates/editor-core/src/model.rs`
- `crates/editor-core/src/validation/animation_presets.rs`
- `crates/editor-core/src/validation/animation_channels.rs`
- `crates/editor-core/src/timeline/animation_presets.rs`
- `crates/editor-core/src/timeline.rs`
- `crates/editor-core/src/migrations.rs`
- `crates/editor-core/src/store.rs`
- `crates/editor-core/src/drafts.rs`
- `crates/editor-core/tests/animation_presets.rs`
- `crates/editor-core/tests/animation_channels.rs`
- `apps/headless/src/main.rs`
- `apps/headless/tests/protocol.rs`
- `apps/agent-bridge/src/headless-contract.ts`
- `apps/agent-bridge/src/schemas.ts`
- `apps/agent-bridge/src/server/timeline.ts`
- `apps/agent-bridge/tests/animation-presets.test.ts`
- `apps/agent-bridge/tests/contracts.test.ts`
- `docs/animation-presets.md`
- `apps/agent-bridge/tests/preset-workflow.ts`
- `apps/agent-bridge/tests/smoke.test.ts`
- `apps/agent-bridge/tests/packaged-smoke.test.ts`

### maskModels: contracts/mask-models-v1.json

- `crates/editor-core/src/model/mask.rs`
- `crates/editor-core/src/validation/mask.rs`
- `crates/editor-core/src/model.rs`
- `crates/editor-core/src/model/buffered.rs`
- `crates/editor-core/src/timeline.rs`
- `crates/editor-core/src/migrations.rs`
- `crates/editor-core/src/store.rs`
- `crates/editor-core/tests/mask_models.rs`
- `apps/headless/src/main.rs`
- `apps/headless/tests/protocol.rs`
- `apps/agent-bridge/src/schemas.ts`
- `apps/agent-bridge/src/headless-contract.ts`
- `apps/agent-bridge/tests/contracts.test.ts`
- `apps/agent-bridge/tests/mask-models.test.ts`
- `apps/agent-bridge/tests/mask-model-workflow.ts`
- `docs/mask-models.md`
- `apps/agent-bridge/tests/fixtures/mask-mcp-projection.ts`
- `apps/agent-bridge/tests/smoke.test.ts`
- `crates/editor-core/src/validation.rs`
- `crates/editor-core/src/render_plan.rs`
- `crates/editor-core/src/renderer/tests/raster_caching.rs`
- `apps/agent-bridge/tests/packaged-smoke.test.ts`

### headlessProtocol: contracts/headless-protocol-v1.json

- `apps/headless/src/main.rs`
- `apps/headless/tests/protocol.rs`
- `apps/agent-bridge/src/headless-contract.ts`
- `apps/agent-bridge/src/schemas.ts`
- `apps/agent-bridge/tests/contracts.test.ts`

### mcpSurface: contracts/mcp-surface-v1.json

- `apps/agent-bridge/src/server`
- `apps/agent-bridge/src/schemas.ts`
- `apps/agent-bridge/tests/contracts.test.ts`

### persistedProject: crates/editor-core/src/model.rs

- `crates/editor-core/src/store.rs`
- `apps/headless/src/main.rs`
- `apps/agent-bridge/src/schemas.ts`

## External correction inventory

proposal.md/design.md/tasks.md/README.md now identify verified50 commit/digest/current-marker amendment; design adds required risks, atomic migration/backup rollback and no-unresolved-normative-question sections, coordinate/private interface and seven semantic pins. specs/contract-governance/spec.md preserves full current marker requirement/scenario; specs/project-persistence/spec.md fully modifies actual50 requirements; specs/mask-models/spec.md and specs/mask-rendering/spec.md clarify logical coordinates and additive retained-fact/kernel accounting. Added predecessor-catalog-digests.json and these reconciliation notes. implementation-interface.md is main-owned exact plan. No checkout implementation or approval is performed by this handoff.

## Exact activation annotation amendment

Model semantics.timing must advance from static_metadata_no_animation_targets to static_and_mask_targeted_animation. It is the fifth exact mask-model predecessor-projection path, alongside projectSchemaVersion/status/rendererStage/semantics.order; verify all exact33 values before restoring32. No unrelated field/fixture change is authorized. This makes existing approved mask-channel activation observable without widening runtime scope.
