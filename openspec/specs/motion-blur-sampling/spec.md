# motion-blur-sampling Specification

## Purpose

Deterministic bounded per-leaf shutter sampling of inherited visual transforms, shared rendering, and atomic schema adoption.

## Requirements

### Requirement: Typed bounded shutter settings
Core MUST accept optional motionBlur on eligible visual leaf items through existing update_item edits, standalone and alias-aware timeline_batch_edit. The closed record MUST contain finite shutterAngleDeg in [0,360] and integer sampleCount in [1,16]. Omission on edits MUST preserve the value; omission on persisted items MUST mean disabled; shutter zero or count one MUST preserve existing instantaneous output. Media, text, solid, rectangle, shape, SVG and grid leaves MUST be eligible; controllers, audio, captions and transitions MUST reject the field. Malformed fields and unsupported targets MUST return non-retryable INVALID_ARGUMENT; missing items and stale revisions MUST preserve existing stable errors. No paths, resources, expressions or scripts MUST be introduced.

#### Scenario: Author and restore shutter settings
- **WHEN** an eligible leaf is created under a batch alias and receives valid settings in that batch or a standalone edit
- **THEN** one revision atomically stores the settings and undo, redo and reopen reproduce them

#### Scenario: Reject without publication
- **WHEN** a setting is nonfinite, out of bounds, unknown, null, unsupported, references a missing item, or is submitted with a stale revision, including after valid earlier batch operations
- **THEN** the existing stable error is returned without changes to revision, history, resources or authoritative bytes

#### Scenario: Deterministic disabled inherited fallback
- **WHEN** omitted, zero-angle or single-sample settings repeatedly render a leaf under inherited animated affine transforms on supported backends
- **THEN** the instantaneous fallback is deterministic and the disabled controls retain identical decoded pixels without changing timing, audio, thresholds or reference fixtures

### Requirement: Canonical deterministic shutter interval
Core MUST calculate midpoint samples on a centered exposure of width shutterAngleDeg / 360 times 1000 / project fps milliseconds. For N enabled samples, sample i MUST use root time t plus width times ((i+0.5)/N - 0.5), in ascending i order. Root times MUST clamp to the project interval [0,durationMs) before evaluation; integer-root sampling MUST floor once after interval construction and retain all inherited fractional mappings thereafter. Disabled settings MUST sample exactly t. Midpoint offsets MUST floor the exact parsed binary64 shutter angle ratio shutterAngleDeg * 1000 * (2*i+1-N) / (720 * projectFPS * N) without floating cancellation or epsilon snapping, including subnormal angles. Sample computation MUST remain deterministic, use no seed or random jitter, and reject unsafe arithmetic. Every sample MUST evaluate visibility, clipping, transitions, animated crop, paint, ordered effects, local transform, all parent/group/component transforms and opacity, loops, stagger and signed repeaters on their canonical clocks.

#### Scenario: Sample inherited boundaries
- **WHEN** shutter intervals cross keyframes, repeat seams, ping-pong turns, finite exhaustion, nested fractional clocks, stagger or signed repeater offsets
- **THEN** sample times, inherited matrices, opacity and half-open activity match an independent oracle

#### Scenario: Clamp the timeline boundaries
- **WHEN** a centered exposure overlaps project start or exclusive end
- **THEN** clamped samples retain their equal weights and deterministic order without accessing out-of-range resources

#### Scenario: Floor exact midpoint boundaries for every allowed count
- **WHEN** valid exposures with any count 1..16 meet integer keyframe boundaries, fractional frame periods, representable neighboring angles, subnormal angles, high integer roots or timeline clipping
- **THEN** exact ratio flooring, ascending equally weighted samples and inherited held-keyframe coverage match independent arithmetic and native frame/range/draft/export oracles without changing tolerance or audio

### Requirement: Shared bounded temporal composition
Each enabled leaf MUST average equally weighted premultiplied linear-light canvas rasters after each sample's crop, local clip, paint, ordered effects, complete inherited affine and transition opacity; existing layer stacking MUST then compose the averaged leaf. Invisible samples MUST contribute transparent pixels. Core MUST reuse the issue43 pipeline for frame, audiovisual range, materialized draft and export. Independent native expected pixels MUST demonstrate blur rather than merely comparing identical consumers. Audio source sampling, gain and synchronization MUST remain unchanged. Native parity MUST meet SSIM >= 0.99, aligned decoded PCM RMS error <= 0.0001 and timing error at most one frame. Disabled/zero shutter and single samples MUST preserve previous output. Cumulative sample-weighted pixel work MUST not exceed 268435456 units per output scene frame, alongside stricter existing effect, geometry, occurrence and surface limits; all arithmetic MUST be checked. Hidden, unused, retained and expanded content MUST undergo canonical validation. Excess or unavailable rendering support MUST fail before destination inspection, workspace creation or persistence publication.

#### Scenario: Preserve full inherited motion and ordering
- **WHEN** asymmetric nested leaves animate under rotating parents with noncentral anchors, clipping, crop and differently ordered effects
- **THEN** independent midpoint and linear-light composition expectations match frame, range, draft and export within the documented tolerance

#### Scenario: Bound work and preserve output
- **WHEN** shutter multiplication exceeds a sample, occurrence, geometry, effect or pixel-work limit or an encoder fails
- **THEN** core returns its existing sanitized typed failure before publication and preserves destinations, project/history bytes and resources

#### Scenario: Preserve audio and compatibility
- **WHEN** enabled blur is rendered alongside audio and compared with zero shutter, single sample and omitted settings controls
- **THEN** audio remains synchronized and unchanged and compatibility controls retain prior visuals

### Requirement: Atomic schema adoption and governed contracts
Schema 27 and all previously supported project versions MUST migrate current state and all retained undo/redo snapshots atomically under the project lock to schema 28 with blur disabled, preserving revisions, IDs, provenance and integrity. Premature motionBlur fields in older schema state/history/drafts and unknown future versions MUST fail closed without rewriting authoritative bytes. Valid schema28 reopen MUST not rewrite bytes. Protocol1 consumers MUST retain existing requests and advertise additive motion_blur_sampling_v1 support. Canonical fixtures, ownership, native declarations, structural consumers, aliases, documentation and parity evidence MUST agree on numeric bounds, timing, coordinate, stacking and fallback semantics. CODEOWNER review MUST be recorded before archival.

#### Scenario: Migrate complete retained state
- **WHEN** legacy current state, retained history and compatible drafts are reopened
- **THEN** atomic adoption preserves lifecycle behavior and deterministic second reopen without enabling blur

#### Scenario: Reject malformed migration
- **WHEN** a retained generation contains premature or invalid settings, or a future schema version
- **THEN** migration fails with the existing typed error and preserves every authoritative file

#### Scenario: Discover and exercise support
- **WHEN** an agent queries status then submits the canonical standalone and batch fixtures
- **THEN** capability/version reporting and every governed Rust, TypeScript, headless and MCP consumer agree on acceptance, rejection and alias behavior
