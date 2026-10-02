## Why

Issue [#45](https://github.com/matiHirCab/AI-Open-Cut/issues/45) needs a canonical, versioned preset compiler whose saved output survives later catalog changes. Animation channels and deterministic curves already exist; presets must compile to those editable primitives rather than introduce another evaluator.

## What Changes

- Add an editor-core-owned, pure compiler and an additive `apply_animation_preset` edit, exposed as `timeline_apply_animation_preset` and in `timeline_batch_edit`.
- Start with one technical seed, `scalar_tween@1`, on the six existing targetless scalar channels. Require explicit endpoints, item-local integer timing, a supported curve, and an explicit preset version. The named creative pack remains issue #46.
- Merge the resolved channel by exact identity: default `collisionPolicy: "reject"`; explicit `"replace"` replaces the whole colliding typed channel. Never silently remove overlapping legacy keyframes. Preserve unrelated channels and static properties.
- Persist descriptive per-channel provenance alongside authoritative `animationChannels`. Clear affected provenance when ordinary edits replace/remove resolved primitives; preserve it through exact copies, undo/redo and reopen. Never consult the compiler during project evaluation or reopening.
- Upgrade project schema 28 to 29 under the existing lock/transaction mechanism, including every retained undo/redo snapshot and component-definition item. Older documents gain empty provenance without changing resolved animation, timestamps, revisions, media or rendered output.
- Synchronize typed core/headless/MCP surfaces, capability reporting, canonical fixtures, ownership and conformance tests. Domain validation, finite bounds and complexity limits remain in core.

## Capabilities

### New Capabilities

- `animation-presets`: versioned compilation, seed catalog, collisions, descriptive provenance, lifecycle, bounds and deterministic primitive-only evaluation.

### Modified Capabilities

- `project-persistence`: atomic schema-29 provenance migration and fail-closed retained-history handling.
- `agent-bridge`: typed standalone/batch preset inputs, returned provenance and additive discovery under protocol 1.
- `motion-graphics-contracts`: fixture-governed preset catalog, effective parameters, primitive/provenance pairs, failures and capability evidence.
- `edit-drafts`: explicit rejection of new preset application inside draft operation lists for this bounded milestone, while preserving existing draft workflows over committed primitive state.

## Impact

Owners: `crates/editor-core` model, `timeline`, `validation`, `migrations` and store orchestration; `apps/headless`; `apps/agent-bridge`; `contracts`; documentation and tests. The compiler will live under the existing `timeline` owner, avoiding a new private-owner dependency edge. Rendering consumes the existing channels unchanged.

Wire changes are additive: new edit/tool/capability and an optional, ignorable item response field; protocol remains 1. **Persisted compatibility transition:** schema 29 is unreadable by old schema-28 builds, which must fail closed; forward migration covers current state and all retained history. No protocol identifiers, errors or retryability change. CODEOWNER review is required from `@matiHirCab` for implementation contracts and consumers.

## Non-goals

- Shipping or closing issue #46's `impact_slam`, `slide_left`, `scan`, `pulse`, or `radar_expand` pack; adding another renderer, new channel semantics, implicit time/rate changes, effects, masks or audio events.
- User-loaded, executable or external presets; renderer expressions; arbitrary paths, SVG or network resources; auto-updating old projects; `latest` version selection; keyframe splicing or automatic legacy-animation conversion.
- Applying a preset edit inside persisted draft operation lists, component-definition editing, or a desktop preset browser. Existing drafts, component instances and rendering continue to use their resolved channel state.
- Publishing a PR, merging, deploying, changing credentials/security, or altering workflow/approval gates.

## Approval status

DESIGN APPROVED on 2026-10-02 by the explicit forwarded user response recorded in `approval.md`, against proposal commit `32ebd32e329287827e15831528bd9fe041203196`. Implementation, tests and local commits are authorized within the bounded design. Issue #44 approval does not authorize this new change. CLI artifact completeness is not implementation acceptance, and this active change must remain unarchived until implemented and verified.

Approved decision: `scalar_tween@1` is the single issue-45 seed, with exact-channel reject/explicit-replace semantics, metadata clearing on raw replacements, atomic schema 29, and the documented draft/component-definition authoring exclusions. No external publication authorization is implied.
