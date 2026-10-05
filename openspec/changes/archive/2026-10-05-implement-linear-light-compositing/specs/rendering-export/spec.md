## MODIFIED Requirements

### Requirement: Shared complete Transform2D rendering
Frame preview, audiovisual range preview, materialized draft preview, and final export MUST consume the same evaluated affine facts for every supported visual source. The adapter MUST preserve transformed source offsets and transparency, interpolate premultiplied alpha, clip to the composition, and apply Transform2D opacity once. It MUST preserve existing audio, geometry and timing semantics when transform2d is absent; all current visuals MUST use linear premultiplied scene composition, intentionally correcting encoded-space translucent blending. No backend SHALL approximate or omit unsupported transform components.

#### Scenario: Render all transformed visual kinds
- **WHEN** asymmetric media, text, solid, rectangle, and caption fixtures use anchor, both scales, both skews, rotation, units, and opacity
- **THEN** all output intents match the canonical coordinate oracle and preserve transparency and clipping

#### Scenario: Compare preview and export
- **WHEN** equivalent timestamps and ranges are rendered in all supported intents
- **THEN** semantic affine plans agree exactly, visual SSIM is at least 0.99, aligned float-PCM RMS error is at most 0.0001, and timing differs by at most one output frame

#### Scenario: Preserve old rendered fixtures
- **WHEN** migrated legacy fixtures with non-default transform, animation, captions, and transitions render without Transform2D
- **THEN** their geometry, clocks, Caption placement, transition activity and audio remain equivalent; unaffected opaque identity pixels remain equivalent and translucent colors/resampled edges match independent linear premultiplied expectations within existing output tolerance

## ADDED Requirements

### Requirement: Shared complete linear scene rendering
Frame preview, audiovisual range preview, materialized draft preview and final export MUST consume identical evaluated sample clocks, paint order and the full linear premultiplied stage semantics for every existing visual source. Equivalent decoded output MUST satisfy visual SSIM at least 0.99, float-PCM RMS error at most 0.0001 and timing alignment within one output frame. Render-intent routing MUST NOT choose encoded-space blending for ordinary or opaque sources. Existing public operations, authored request/project/error declarations and schema version MUST remain unchanged; only an additive readiness-gated capability MUST distinguish support. Render preparation MUST remain read-only over the immutable project revision; valid standalone/batch edits, alias resolution, stale revision errors, rollback, undo/redo and reopen MUST preserve their existing core semantics.

#### Scenario: Compare all production intents
- **WHEN** a mixed-source translucent scene is rendered at matched sample times through frame, range, draft and export paths
- **THEN** the same independently verified linear overlap colors and scene ordering are observed within documented visual/audio/timing tolerance

#### Scenario: Preserve editing and persistence behavior
- **WHEN** an existing project receives valid standalone or batch edits, failed missing-reference or stale-revision edits, undo/redo and reopen before repeated rendering
- **THEN** accepted states render deterministically, failed edits preserve state/history/revision and no new authored fields, migration or operation are required

### Requirement: Readiness-gated linear composition capability
Protocol-version-1 capability reporting MUST add the unique `linear_light_compositing_v1` capability only when the configured local renderer can execute the complete currently supported scene with the normative linear premultiplied pipeline. An absent or unusable renderer MUST omit that capability while preserving existing readiness errors. Canonical headless/MCP capability catalogs and every governed Rust/TypeScript consumer and parity test MUST agree; existing simple clients MUST remain valid and no authored or persisted contract shape MUST change.

#### Scenario: Detect ready corrected rendering
- **WHEN** a client queries protocol-v1 information with a conforming ready renderer
- **THEN** the capability list contains `linear_light_compositing_v1` and the canonical catalogs/consumer parity agree

#### Scenario: Omit unsupported capability
- **WHEN** the renderer executable, required local dependency or complete pipeline is unavailable
- **THEN** protocol information omits `linear_light_compositing_v1` and render requests retain their stable `DEPENDENCY_UNAVAILABLE` behavior without degraded fallback
