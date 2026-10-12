## MODIFIED Requirements

### Requirement: Stable local binding targets
A binding MUST be exactly `{targetLayerId,property}` and resolve to an item in the owning component's local tracks. Property MUST be a closed identifier from the following mapping: `text.document` accepts text or rich_text on text items; `text.color` accepts color on text items; `visual.opacity` accepts number in [0,1] on visual items; `visual.hidden` accepts boolean on visual items; `text.alignment` accepts enum choices drawn from left/center/right on text items; `item.durationMs` accepts duration on items with explicit duration; `media.asset` accepts asset on media items; `shape.fill.color`, `shape.stroke.color` and `grid.dot.paint.color` accept color on their existing solid local shape fill, shape stroke paint and dot-grid paint respectively under the exact RGB/alpha, compatibility and schema rules of solid-graphic-color-slots. Text and rich-text defaults/overrides MUST remain typed, without flattening rich text. Missing targets MUST fail with ITEM_NOT_FOUND; explicit caller-selected scope fields or arbitrary property paths, incompatible item/property/kind combinations and duplicate target/property writers MUST fail with INVALID_ARGUMENT. Scope MUST be derived from the owning definition, never a caller-selected root or foreign composition. Binding identities MUST remain stable across track order changes. Every resolved default or instance override MUST also satisfy the target's domain rules, including positive duration, containing-component bounds, source trim/time-scale bounds and existing media compatibility. Core MUST validate a derived candidate for effective values without mutating stored base tracks; the no-default required slot case MUST defer value checks until an instance supplies its value.

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

