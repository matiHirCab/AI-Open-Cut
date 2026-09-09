## Context

The current transition counter sums ordinary evaluated layers, while copy publication clones their transition vectors without another complete count. Hidden transitions are omitted by the visible transition index. The current audio-closure walker descends through authored component tracks rather than effective slot values. Calling the full slot resolver from that walker would re-enter component/scope validation.

## Goals / Non-Goals

Goals: enforce the existing transition limit on complete retained expansions and enforce visual-only sources on effective content through every core entry point. Non-goals: repeating audio, changing transition timing/roles, adding public surfaces or migrations, or modifying unrelated work.

## Decisions

### Count facts on projected occurrences

Keep a lightweight retained transition-fact count associated with each ordinary source layer, including facts from hidden transitions/tracks. Count one fact per attached endpoint role, including both roles for self endpoints. Generated projection entries reuse the base count; local generated occurrences copied externally count again exactly once per outer occurrence. Accumulate with checked arithmetic in complete-domain projection validation before the single generated clone point. Retain bounded early ordinary checks, but do not treat them as proof of the complete budget.

Use independent fresh budgets for root and every standalone definition, including hidden/clipped occurrences. Do not mutate the visible transition vectors to perform retained validation. Existing visible indexing and rendering remain unchanged. Counting after cloning was rejected because it violates allocation guarantees; counting transition controller records was rejected because endpoints generate distinct facts.

### Separate slot application from recursive validation

Extract the existing non-recursive default/override application and value checks from resolve_component_slots into a shared internal helper. The full resolver continues to validate the effective component afterward. The effective-audio traversal uses the application helper, not the full resolver, avoiding resolve -> validate_scope -> audio -> resolve recursion. Preserve required/default semantics, RichText handling, canonical errors and unknown-slot checks.

Move audio-closure validation out of raw validate_scope into the canonical project validation sequence after reference, cycle and slot validation. Use the same phase for mutation, draft, reopen and direct evaluation callers. Validate local repeater closures in standalone definitions with defaults and in actual effective instances with overrides; independently validate root repeater sources. Recursively resolve component descendants before classifying their media assets. Unused, hidden, clipped or muted content does not bypass this restriction.

Retain active group/component sets even after canonical graph validation. Cache effective validation/audio summaries by definition and canonical effective slot values, scoped to one project validation; never cache by definition alone. Avoid expanding repeater copies for audio classification. Transport-side validation and raw-definition audio checks were rejected because they respectively duplicate ownership and reproduce the bug.

### Preserve history and demonstrate failures

Use existing thread-local generated-materialization instrumentation and facade no-I/O tests. Regression tests must fail against the current implementation before fixes. Add a dated note to the archived predecessor's proposal, tasks and verification, preserving its historical results but qualifying its no-findings conclusion and linking this follow-up. Keep that predecessor archived.

## Risks / Trade-offs

- Moving validation can alter precedence: establish references/cycles/slot validity before effective-audio checks and explicitly test missing references, invalid slots and stale revisions.
- Hidden transition metadata can leak into output: store counts separately and assert hidden transitions remain absent from published facts.
- Different overrides can contaminate caches: cover opposite audio classifications on two instances of one definition in both declaration orders.
- Defaults can independently invalidate an unused definition: each standalone domain remains subject to validation, even when an actual instance overrides it.
- Retained checks add work: reuse bounded metadata and per-effective-value summaries, without copy materialization or external resource probing.

## Migration Plan

No schema migration or wire change. Existing invalid effective-audio sources fail closed through normal validation; never rewrite or silently strip content. Rollback is an implementation revert with unchanged persisted formats. Complete all gates and a fresh verification before syncing deltas and archiving.

## Open Questions

None. The user explicitly approved these artifacts on 2026-09-08 before executable edits.
