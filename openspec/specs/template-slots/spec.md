# Template Slots Specification

## Purpose

Define typed component inputs, bounded values, stable local bindings and atomic slot replacement while preserving shared definitions and current rendering.

## Requirements

### Requirement: Closed typed slot definitions and values
Core MUST support component-local slots with `id`, `name`, `kind`, `required`, optional `defaultValue`, `binding` and kind-specific `constraints`. Values MUST be closed `{type,value}` records with matching types `text`, `rich_text`, `color`, `number`, `boolean`, `enum`, `duration`, or `asset`; null, coercion, unknown fields and mismatched tags MUST fail with non-retryable INVALID_ARGUMENT. Slot IDs MUST match `[A-Za-z0-9_-]{1,128}`, names MUST be nonblank and at most 256 UTF-8 bytes, and IDs MUST be unique within their definition. Core MUST validate every default even when an override exists. A required slot without a default MUST be legal on a definition, but every stored instance MUST supply a value for it. Override takes precedence over default; an absent optional value MUST leave the original target property unchanged. Unknown override IDs MUST fail with ITEM_NOT_FOUND. Overrides MUST NOT modify the shared definition or be materialized into its tracks.

#### Scenario: Round-trip all eight kinds
- **WHEN** definitions and nested instances contain valid values of every supported kind
- **THEN** reads, undo/redo and reopen preserve exact typed values and deterministic override/default precedence

#### Scenario: Validate absent and invalid values
- **WHEN** a default has the wrong tag, an override is null or unknown, a required effective value is absent, or an optional value is omitted
- **THEN** core rejects the invalid candidates with the specified error and accepts optional omission without changing the target

#### Scenario: Preserve special slot identifiers
- **WHEN** definitions and instances use __proto__, constructor or toString as legal slot IDs, including required slots without defaults and overridden defaults
- **THEN** native persistence and bridge reads preserve every own typed override exactly across undo, redo and reopen, overrides take precedence and no object prototype changes

#### Scenario: Validate special-key values without dropping entries
- **WHEN** an own special-key override is malformed, a required value is missing or the key does not name a declared slot
- **THEN** malformed input is rejected with INVALID_ARGUMENT and a key-prefixed bridge validation path, missing required values fail with INVALID_ARGUMENT, and structurally valid unknown IDs reach core and fail with ITEM_NOT_FOUND without mutation

Closed slot records MUST reject every unknown own enumerable field, including __proto__, constructor, toString and ordinary unknown names, before fields can be discarded. This applies to slot definitions, bindings, constraints, each typed value envelope, rich-text documents/runs and managed-asset references. Override-map keys MUST remain unrestricted by record-field closure and retain the existing slot-ID contract.

#### Scenario: Reject unknown fields throughout closed slot records
- **WHEN** a default or override contains an unknown own field at any applicable closed-record location, or a definition, binding or constraints record contains one
- **THEN** native and bridge structural validation reject it rather than silently removing it, bridge diagnostics identify the full containing-record path and offending keys, and no project or history mutation occurs

#### Scenario: Preserve valid records and special map identifiers
- **WHEN** all eight valid value kinds and legitimate special slot IDs pass through guarded record validation
- **THEN** exact parsed values, prototypes, required/default precedence, undo/redo/reopen and group-opacity behavior remain unchanged

### Requirement: Bounded slot content and constraints
Text MUST contain valid Unicode scalar sequences, with inclusive optional `minLength` and `maxLength` constraints measured in Unicode scalar values. Rich text MUST use `{runs:[{text,bold?,italic?,color?}]}` with Boolean style flags and optional six-digit hex colors; HTML, SVG, links, external fonts and unknown run fields MUST NOT be interpreted or accepted as document structure. Total text per value MUST be at most 4096 scalars and rich text MUST have 1–256 runs. Color MUST match `#[0-9A-Fa-f]{6}`. Number MUST be finite; duration MUST be a nonnegative integer millisecond value at most 9007199254740991. Number and duration MUST accept optional inclusive `min`/`max` constraints with ordered, type-valid endpoints. Enum MUST be a string selected from required `choices` of 1–128 unique nonempty strings, each at most 128 scalars. Boolean MUST be a JSON Boolean. Asset MUST be a project-scoped `{kind:"asset",scope:"project",id}` reference resolving to a managed asset and MAY be constrained by a nonempty unique `assetKinds` subset of image/video/audio. Boolean and color constraints MUST be empty. Asset IDs MUST follow existing managed-ID safety rules. Inapplicable constraint fields MUST fail. Slot counts MUST be at most 128 per definition and 4096 project-wide; aggregate default and override text MUST be at most 1048576 scalars per snapshot, including rich-text runs and enum values. Override count MUST be at most 128 per instance. Bounds MUST apply to hidden and unused definitions before resolving or expanding content.

#### Scenario: Enforce inclusive limits
- **WHEN** values and collections are exactly at each named bound versus one above it, contain non-finite numbers, or violate ordered ranges or enum uniqueness
- **THEN** core accepts valid boundaries and rejects every overflow or malformed constraint with INVALID_ARGUMENT

#### Scenario: Count Unicode consistently
- **WHEN** text includes supplementary-plane characters, combining marks, or malformed Unicode
- **THEN** Rust and TypeScript agree on scalar counts and reject malformed Unicode without silent replacement

#### Scenario: Confine managed assets
- **WHEN** an asset value references a missing managed ID, an incompatible asset kind, a path, a URL or executable resource content
- **THEN** core returns ASSET_NOT_FOUND for an absent safe ID and INVALID_ARGUMENT for invalid kinds or unsafe resource forms without mutation

### Requirement: Stable local binding targets
A binding MUST be exactly `{targetLayerId,property}` and resolve to an item in the owning component's local tracks. Property MUST be a closed identifier from the following mapping: `text.document` accepts text or rich_text on text items; `text.color` accepts color on text items; `visual.opacity` accepts number in [0,1] on visual items; `visual.hidden` accepts boolean on visual items; `text.alignment` accepts enum choices drawn from left/center/right on text items; `item.durationMs` accepts duration on items with explicit duration; `media.asset` accepts asset on media items. Text and rich-text defaults/overrides MUST remain typed, without flattening rich text. Missing targets MUST fail with ITEM_NOT_FOUND; cross-scope or arbitrary property paths, incompatible item/property/kind combinations and duplicate target/property writers MUST fail with INVALID_ARGUMENT. Scope MUST be derived from the owning definition, never a caller-selected root or foreign composition. Binding identities MUST remain stable across track order changes. Every resolved default or instance override MUST also satisfy the target's domain rules, including positive duration, containing-component bounds, source trim/time-scale bounds and existing media compatibility. Core MUST validate a derived candidate for effective values without mutating stored base tracks; the no-default required slot case MUST defer value checks until an instance supplies its value.

#### Scenario: Resolve local identity and compatible properties
- **WHEN** equal item IDs exist in different compositions or tracks are reordered
- **THEN** each binding resolves only in its owner and retains the same property target

#### Scenario: Reject invalid bindings and effective values
- **WHEN** a target is removed, two slots write the same property, a rich-text slot targets opacity, a duration overruns the composition, or an asset violates media rules
- **THEN** core rejects the complete candidate with the specified typed error before publication

#### Scenario: Apply group opacity without requiring explicit Transform2D
- **WHEN** a group with absent or existing Transform2D receives default or instance-override opacity 0, 0.5 or 1
- **THEN** the effective candidate uses that Transform2D opacity, preserves all other transform fields, passes complete core validation and leaves stored base tracks unchanged

#### Scenario: Preserve group opacity failure atomicity
- **WHEN** a group opacity default or override is outside [0,1], a changed slot targets a locked group track, a revision is stale or a later batch operation fails
- **THEN** existing INVALID_ARGUMENT, TRACK_LOCKED, retryable REVISION_CONFLICT or later-operation errors preserve the prior revision and byte-identical project/history files

### Requirement: Atomic slot definition replacement
Core MUST expose `component_define_slots {componentId,slots}` replacing the complete slot list, as a standalone edit and an existing 1–100 operation batch variant. It MUST validate every incoming instance against the candidate slots without silently deleting stale overrides. Earlier component creation aliases MUST resolve in componentId; local slot/item IDs MUST remain literal and the operation MUST NOT produce resultAlias. One successful batch MUST commit once and occupy one undo step. A stale revision MUST fail with retryable REVISION_CONFLICT, a missing component with ITEM_NOT_FOUND, and any added/removed/changed slot targeting a locked local track with TRACK_LOCKED. Unchanged slots on locked tracks MUST remain legal. Later batch failure MUST preserve the complete prior revision and byte-identical project/history files. Existing alias-envelope errors MUST remain unchanged. Successful results MUST report the affected component ID deterministically.

#### Scenario: Create and define slots by alias
- **WHEN** a batch creates a definition then defines its slots using the creation alias
- **THEN** the batch commits once and undo/redo/reopen restores exact slots, defaults and instance values

#### Scenario: Reject stale, locked and partially valid edits
- **WHEN** replacement invalidates an incoming instance, alters a locked binding, uses a stale revision or missing component, or precedes a failing batch operation
- **THEN** the specified error preserves all committed state and retained history
