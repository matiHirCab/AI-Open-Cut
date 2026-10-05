## ADDED Requirements

### Requirement: Structurally scoped introduced-field validation
Historical source-version validation MUST distinguish actual persisted model fields from legal dynamic component slot identifiers. The introduction guards for `animationChannels` below22, item `startTime` below24, actual inherited `staggerMs` and repeater `timeOffsetMs` below26, item `crop`/`effects` and actual extended channel properties below27, and item `motionBlur` below28 MUST inspect their owning root/component track items or channel/descriptor locations, not arbitrary nested dictionary keys. A legal slotValues entry whose slot ID equals any such name MUST retain its typed value and migrate to current schema31 without being interpreted as animation metadata. Actual premature fields MUST still fail closed by presence, including explicit zero/null/empty records. Supported current state and every retained undo/redo snapshot MUST use the same source validation under the existing lock and atomic complete-generation protocol. Existing structural clocks/Scalar/Pack provenance guards, future-schema errors, finite/resource safety and recovery semantics MUST remain unchanged.

#### Scenario: Migrate legal colliding slot identifiers
- **WHEN** a supported historical root or nested component instance contains a legal Number slotValues entry named animationChannels, startTime, staggerMs, timeOffsetMs, crop, effects or motionBlur at a source version before the similarly named model field was introduced
- **THEN** migration validates the actual slot binding/value and preserves that slot ID/value, components, animation, assets/fonts, revisions and timestamps through schema31 without inferring introduced fields or recompiling provenance

#### Scenario: Migrate every retained legal slot generation atomically
- **WHEN** a supported current project or schema31 current project retains undo/redo snapshots containing those legal historical slot identifiers
- **THEN** current state and all retained snapshots migrate and publish as one validated generation, reopen idempotently, and Undo/Redo restore the exact intended slot values

#### Scenario: Reject actual premature model fields and preserve safety
- **WHEN** an actual root/component item, channel or repeater descriptor contains a field before its introduction, including zero/null/empty presence, or any retained source has malformed clocks/provenance, invalid slot values or an unknown future schema
- **THEN** validation retains its established failure and leaves current project/history/managed resources unchanged; legal dictionary names do not bypass the genuine source-version or semantic checks

#### Scenario: Preserve failure and recovery atomicity
- **WHEN** a malformed committed journal or pre-commit publication failure occurs while validating/migrating legal colliding slot generations
- **THEN** existing recovery/fault semantics apply, malformed sources are rejected before replay/publication, and no partial current/history migration or rewritten authoritative bundle is exposed
