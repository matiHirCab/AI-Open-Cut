## MODIFIED Requirements

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
