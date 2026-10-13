## ADDED Requirements

### Requirement: Authoritative bounded instance slot inspection
Desktop SHALL inspect the selected component instance's authoritative definition slots in declared order, showing stable ID/name, kind, binding, required state, shared default and stored override with explicit source labels. Override, default and absent optional value MUST remain distinct; absence SHALL disclose that the bound source property is retained without inventing a default or evaluating a second scene model. Inspection MUST retain full scoped occurrence identity and remain bounded by existing slot limits. Component-local occurrences, rich_text, number, enum, duration and any unsupported binding MUST remain visibly read-only.

#### Scenario: S1 inspect independent occurrences and sources
- **WHEN** two root instances share a definition and have different overrides, defaults or optional omissions
- **THEN** the inspector identifies each occurrence and exact authoritative values/source labels without changing the definition, other instance or revision

#### Scenario: S2 inspect unsupported and scoped content
- **WHEN** a component-local occurrence or unsupported slot kind is selected, including repeated local IDs and a maximum-size slot definition
- **THEN** the same authoritative data remains bounded and visible read-only, with no mutation controls or conflated selection

### Requirement: Existing typed text color asset and visibility controls
Root instances SHALL expose text slots as literal Unicode text, color slots as exact typed hex6 strings, visibility slots as explicit Boolean visual.hidden values, and asset slots as managed project asset references. Controls MUST identify slot binding/units and preserve color casing and alpha semantics, text newlines and legal special IDs. Asset selection SHALL use the already managed project inventory without import, paths or network inputs; representation input MAY carry a literal managed ID so missing IDs receive the canonical core error. The UI MUST NOT implement semantic constraints, target validation, asset-kind admission or renderer evaluation in parallel. Input buffers SHALL be bounded while supporting an untruncated representation of up to4096 Unicode scalars; core SHALL continue to enforce both slot scalar/constraint limits and the resolved text1–4096 UTF-8-byte limit; malformed representation MUST produce parsing feedback without mutation.

#### Scenario: S3 represent all supported slot families faithfully
- **WHEN** text/newlines/Unicode, mixed-case hex6, managed asset IDs and hidden true/false are edited
- **THEN** the existing typed values preserve exact caller input and reach the authoritative core edit without flattening rich text, changing alpha or importing media

#### Scenario: S4 preserve special IDs and text boundaries
- **WHEN** slots use __proto__,constructor or toString and text reaches4096 supplementary Unicode scalars or exceeds core constraints
- **THEN** exact map keys and full input representations survive, values within the representation buffer that violate core constraints (including4096 supplementary scalars exceeding the resolved text byte limit) retain the canonical core rejection, and buffer exhaustion gives visible representation feedback without submitting a silently truncated value or dropping entries

### Requirement: Narrow revisioned Apply and explicit override reset
Apply SHALL clone the selected instance's current override map and replace only the selected supported slot through existing ComponentInstanceUpdate with the displayed expected revision, preserving component reference, timing, stagger and every unrelated override/property. Clearing an override SHALL remove only that literal map key through the same core operation and disclose shared-default or bound-source fallback. Draft Reset SHALL discard uncommitted input and show authoritative state without a revision. Removing a required value with no default MUST receive the existing core failure rather than fabricate fallback data. Success SHALL reload the authoritative project; no optimistic domain publication or new edit operation/schema is permitted.

#### Scenario: S5 apply one independent override
- **WHEN** a supported slot is applied on one of two differently customized root instances
- **THEN** one core transaction changes only its selected override, and shared sources, the other instance, timing, transforms, ordering, resources and unrelated overrides remain exact

#### Scenario: S6 distinguish stored and draft reset
- **WHEN** a stored override is cleared versus an unfinished draft is reset
- **THEN** clearing uses one revisioned core edit and restores existing default/base resolution while draft Reset leaves project/history bytes and revision unchanged

#### Scenario: S7 reject absent required fallback
- **WHEN** a required slot without a definition default has its stored value cleared
- **THEN** existing INVALID_ARGUMENT and unchanged complete project/history/resource bytes are preserved with no manufactured value

### Requirement: Canonical safety and stale draft identity
Core SHALL remain the authority for bounds, malformed colors/text, missing or incompatible managed assets, references, locks and revision conflicts. Desktop MUST display unchanged canonical code/message/retryability for semantic failures and publish no partial state. Draft identity SHALL include full scoped selection, authoritative revision, component reference, literal slot ID and kind/binding. Field/selection/tab changes, Reset, refresh, successful edit/history and reopening MUST discard stale drafts; asynchronous completion MUST not revive drafts from another context. Busy and needs_refresh MUST guard all slot mutation actions. A conflict SHALL require explicit refresh and replanning, without automatic retry.

#### Scenario: S8 retain invalid input and lock failures
- **WHEN** core rejects malformed color, constrained text, missing/incompatible asset, missing reference or locked occurrence
- **THEN** the existing failure is visible and complete prior revision/project/history/resources remain unchanged

#### Scenario: S9 require refresh after external revision conflict
- **WHEN** an external edit advances the project while a slot draft uses the displayed older revision
- **THEN** retryable REVISION_CONFLICT leaves that edit unapplied, blocks further mutations until explicit refresh and discards obsolete input before replanning

#### Scenario: S10 discard drafts across context and pending work
- **WHEN** slot/selection/tab/history/refresh changes or asynchronous work completes after another occurrence is selected
- **THEN** stale input cannot target a different slot, component, occurrence or revision, and busy/conflict guards preserve existing state

### Requirement: Retained editable lifecycle and compatibility
Undo, redo and reopening SHALL use existing core/session operations and reconstruct exact committed slot values, source defaults and independent instances. Desktop-only changes MUST retain every existing hierarchy/vector/text/animation/compositing/narration/review control, public contract/catalog/pin, persisted version, core validation/rendering behavior and protected gate. Current-main and exact pending199/200 compatibility MUST be reviewed before normal publication without importing an unmerged domain or weakening checks.

#### Scenario: S11 restore instance edits through history and reopen
- **WHEN** Apply or clearing an override is undone, redone and reopened
- **THEN** exact authoritative values/precedence and both independent instances restore without persisted UI draft state

#### Scenario: S12 preserve existing domains and pending owners
- **WHEN** complete repository and exact overlapping-head evidence is evaluated
- **THEN** prior commands/contracts/pins/negative controls and existing desktop behavior remain enforced, and independent resolution records the compatible composition

### Requirement: Actual isolated desktop interaction evidence
The affected desktop MUST build and demonstrate actual GPUI interactions against a disposable deterministic project with two shared-definition instances and all supported/unsupported slot families. Evidence SHALL record source/build/environment, actual controls, screenshots and authoritative persisted transitions for Apply, clear-override, draft Reset, invalid/lock/conflict feedback, selection/tab changes, history and reopen. Session/model tests MUST supplement actual GUI interactions. Baseline empty windows, nonrendering captures, fake transport checks or screenshots alone MUST NOT substitute for feature acceptance. Required repository/native/specification and all18 exact-published-head hosted jobs with the unchanged120-minute audit MUST pass. No installed/private production desktop, media, configuration or renders SHALL be accessed or modified.

#### Scenario: S13 exercise supported feature controls in the built app
- **WHEN** the built source app is driven through real input events on the isolated display and disposable project
- **THEN** all four supported families, source labels, two-instance isolation, resets, failures and lifecycle are observed with recorded project revisions and screenshots

#### Scenario: S14 reject incomplete evidence or unavailable GUI
- **WHEN** required interaction/runtime evidence is missing, failed or only an empty/nonrendering baseline is available
- **THEN** acceptance remains explicitly blocked, with complete failure evidence retained and no substitute success or portable-performance claim
