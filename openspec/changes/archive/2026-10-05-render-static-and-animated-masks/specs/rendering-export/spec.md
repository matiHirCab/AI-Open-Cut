## RENAMED Requirements

- FROM: `### Requirement: Mask metadata preserves current evaluated rendering`
- TO: `### Requirement: Shared evaluated mask rendering`

## MODIFIED Requirements

### Requirement: Shared evaluated mask rendering
Schema33 static/animated masks MUST execute mask-rendering semantics after source crop/clip before existing local effects and owner/ancestor transforms through the same evaluated scene in frame, audiovisual range, draft preview and export. Render samples MUST preserve existing source/owner clocks, requested origins, motion-blur/shutter averaging, effect support, layer ordering, inherited opacity and audio behavior. Model/animation capability SHALL NOT imply rendering readiness; complete renderer readiness MUST govern mask_rendering_v1. Existing schema 32 nonempty masks MUST intentionally change from identity to approved mask coverage after atomic adoption; absent/empty masks MUST retain exact prior geometry/clock/effect/resource/normalized-graph/lossless pixel/audio/timing output and existing budgets/failures. Persisted/snapshot/revision-cache fingerprints MAY change with normal metadata/channel edits; existing revision-scoped cache invalidation MUST remain. Public lifecycle comparisons MUST normalize only explicitly named revision/snapshot admission/output/temp identities, not authored render semantics or resource integrity.

#### Scenario: Compare every production intent with animated masks
- **WHEN** the same static/animated asymmetric painted masks are sampled through frame/range/draft/export at matching inherited/fractional/shutter clocks
- **THEN** evaluated mask plans are identical, independent coverage/pixel oracles hold and existing SSIM≥0.99, PCM RMS≤0.0001 and timing≤one-frame tolerances remain

#### Scenario: Preserve no-mask output and failures
- **WHEN** otherwise identical prior scenes omit/have empty masks or encounter existing media/dependency/process failures
- **THEN** no-mask output remains exact under existing identity normalization and stable failure/cleanup guarantees remain, without partial artifact/state publication
