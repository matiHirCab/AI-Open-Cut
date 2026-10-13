## Context

Main0f includes hero and graphic-slot foundations. Desktop currently renders stored overrides as JSON. `TemplateSlot`/`SlotValue` and ComponentInstanceUpdate already own typed values and semantics; Selection carries scope and complete instance path. Existing desktop dispatch delegates to core, reloads on success, guards busy/conflicts and discards drafts. Existing component-local inspection is read-only and remains so.

The saved cloud has an isolated usable GUI route: source-main desktop built and rendered on private Xorg190 with workspace-extracted Mesa software Vulkan; actual Cues-tab input changed the panel. The first black capture is retained as failure evidence; explicit resize produced actual rendering. No feature behavior is implemented or verified by that baseline. All tooling/project/screenshots stay outside source and separate production desktop/video tasks.

## Goals / Non-Goals

Provide four narrowly supported root-instance slot families with authoritative source labels, one-slot edits and explicit reset semantics, full draft identity and actual GUI evidence. Preserve every old property and core boundary.

No definition/child/path editor, importer, rich-text flattening, new scene/slot evaluator, semantic validator, core/headless/bridge API, schema/migration, renderer, dependency, auto-retry, global configuration/credential change, merge or deployment.

## Decisions

1. **Definition-derived presentation.** A small desktop module resolves the selected instance/definition through existing Selection, enumerates at most the existing128 slots and exposes their stored metadata/default/override. Editable kinds are Text, Color, Boolean on visual.hidden and Asset on media.asset; all others remain explicit read-only. Optional absence is labelled as bound-source fallback, not guessed effective values. Hardcoded hero slot names would hide legal custom/special IDs; a second evaluator would duplicate core semantics.
2. **Separate slot draft/context.** Reuse the established field UI interaction conventions with slot-specific identity and actions, independent of vector/animation/compositing field indexes. Identity includes Selection/revision/component/slot/kind/binding; existing navigation/dispatch resets the slot draft too. Bound UTF-8 text input capacity supports4096 Unicode scalars (up to16384 bytes). Exhaustion reports representation feedback and blocks Apply of a silently truncated value until the user corrects or resets input. Core owns scalar/constraint validation and the additional resolved-text1–4096 UTF-8-byte limit for represented values. A4096 supplementary-scalar draft is fully representable but canonically rejected by current core; it is not claimed to be a valid committed value. Extending generic path/updateKey strings alone would conflate slot metadata with existing fields and weaken context guards.
3. **Literal narrow map edits.** Clone current BTreeMap, insert or remove one literal slot key, and construct existing ComponentInstanceUpdate using unchanged reference/timing/stagger. Existing operation preserves all static visual properties; tests compare full snapshots. Text and color strings remain exact; Boolean representation is explicitly true/false; asset values fix kind=asset/scope=project and carry a literal managed ID. A bounded selection list offers existing managed assets; no importer/URI interpretation. Core decides missing IDs, incompatible types, required fallback, constraints and locks. New domain operations/validators are unnecessary.
4. **Clearly separate resets.** Draft Reset performs no mutation. Clear override (use shared/default source) is a revisioned core mutation and may fail for required-without-default values. These labels preserve old Reset semantics and avoid silently treating empty text as omission.
5. **Use existing asynchronous session safety.** Apply/clear guard busy/needs_refresh and captured identity, dispatch with displayed revision, reload on success and show canonical errors. No optimistic mutation, automatic conflict retry or stale async draft restoration. History/reopen use existing APIs. A new editor service or transport would broaden scope.
6. **Attributable GUI and runtime proof.** Author a deterministic core-backed example/project generator with two differently customized root instances, supported/default/required slots, unsupported kinds and managed original test artwork; document explicit store/project startup. Automated desktop tests cover field descriptions, exact edits, special IDs/Unicode, failure inventories, draft/context/busy guards and history/reopen. Real X input on source-built GPUI drives the same cases; screenshots supplement persisted core snapshots. Environment packages are extracted under workspace, with private display/bus and cache paths; baseline/tool failures stay retained. Native/golden/motion and complete repository gates remain mandatory, with unchanged references/deadlines/budgets.

## Risks / Trade-offs

- Shared/default/absent values are distinct: label all three, with no fabricated evaluator output.
- Complete map replacement could erase an unrelated value: clone authoritative map and compare full other-instance/definition/property/resource snapshots.
- Root versus repeated local identity: retain existing local read-only guard and complete Selection in draft identity.
- Stale async state: reuse session guards and discard slot drafts on every relevant navigation/dispatch/history/revision path.
- Large/legal Unicode values: explicitly support untruncated scalar representation with a bounded buffer; preserve the core resolved-text byte rejection and add no new semantic threshold.
- Optional asset/visibility controls can still fail core constraints: preserve canonical error surfaces and complete rollback; never prevalidate a second domain.
- GUI infrastructure may fail: retain actual captures/logs, prove rendered feature interactions, and block acceptance if required evidence is unavailable.
- Main/pending movement: preserve merged domains normally and review exact199e2a/2006fd overlap and prior pins before publication.

## Migration and rollback

No persisted/public format or dependency changes. Remove only new desktop controls/draft presentation/tests/docs to roll back; existing stored slot values remain readable/editable through current core/MCP APIs. Normal source publication remains draft/unmerged; no deployment is part of this task.

## Open Questions

None requiring a scope decision. Baseline isolated rendering/input works; the complete feature GUI suite remains mandatory and pending. Human CODEOWNER review is separate from delegated artifact/implementation resolution.
