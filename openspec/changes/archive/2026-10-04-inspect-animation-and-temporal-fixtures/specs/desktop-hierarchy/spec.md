## ADDED Requirements

### Requirement: Bounded authored animation inspection
Desktop MUST inspect animation from the authoritative selected item with its full scoped identity, using bounded channel/key selection rather than eagerly expanding every channel/key. Inspection MUST expose active typed channel kind, target identity/scope, selected source key time/value/curve, loop, retained clock offset/source duration, preset attribution and legacy animation. Source times MUST be explicitly distinguished from visible local time. Component-local occurrences, entire compound channels (including values, source times, curve parameters and loop metadata), legacy tracks and retained clock metadata MUST remain read-only. Root audio-only media MUST expose applicable scalar gain inspection/editing without exposing unrelated visual controls.

#### Scenario: A1 Inspect scoped and retained animations
- **WHEN** selection moves among root visual/audio items, compound or legacy animation and repeated component occurrences
- **THEN** displayed authored values, attribution and source clocks match the authoritative scoped item, supported root fields alone are editable, and the inspector remains bounded for maximum valid channel/key counts

### Requirement: Revisioned existing animation controls
Desktop MUST edit existing scalar-channel key values/source times, existing scalar-channel curve parameters, scalar-channel loop mode and finite/infinite iterations, root group/component stagger and existing repeater signed time offset through existing typed core operations with the displayed expected revision. New loops MUST initially use finite count1; removing a loop MUST preserve other channel data. Channel replacement MUST preserve every unedited key, target, channel and explicit retained clock and MUST disclose the existing clearing of preset attribution before Apply. Desktop MUST NOT reattach trusted provenance or perform parallel domain validation, channel/key creation/deletion, curve variant changes or raw-expression authoring. Parse failures MUST avoid mutation; core failures MUST retain code/message/retryability and no speculative state. Success MUST reload, and reset/selection/refresh/history MUST discard stale drafts.

#### Scenario: A2 Apply supported authored edits
- **WHEN** a supported scalar-channel value/source-time/curve/loop field, stagger or repeater offset is applied
- **THEN** one existing core transaction commits only intended authored fields, preserves all unrelated channel/source-clock/controller fields, applies existing provenance clearing for channel replacement and reloads the committed revision

#### Scenario: A3 Reject invalid or stale animation input
- **WHEN** representation parsing fails or core rejects invalid curve/key/loop/controller bounds, missing item, locked track or stale revision
- **THEN** no partial mutation commits, the applicable feedback remains visible and conflicts require explicit refresh without automatic retry

#### Scenario: A4 Reset and restore animation history
- **WHEN** an animation draft is reset, selection/refreshed revision changes, or an edit is undone, redone and reopened
- **THEN** stale input cannot target a different identity/revision, reset performs no mutation and displayed values/clocks/provenance match each authoritative retained state

### Requirement: Demonstrated desktop animation workflow
The animation inspector change MUST include a successful desktop build and genuine manual GUI workflow evidence covering inspection, valid Apply, failure feedback, reset, selection changes, history and refresh. Session tests MUST supplement rather than replace that evidence; unavailable GUI execution MUST remain an explicit acceptance blocker. Documentation MUST identify the placeholder preview and production render fixture commands accurately.

#### Scenario: A5 Exercise the actual desktop workflow
- **WHEN** the affected desktop is built and used against the generated animation project
- **THEN** the observed controls and authoritative transitions are recorded with environment/commands and limitations, and unavailable or failed required observations are not reported as passing
