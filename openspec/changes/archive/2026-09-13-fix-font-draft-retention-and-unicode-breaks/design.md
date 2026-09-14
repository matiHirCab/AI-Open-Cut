## Context

The three review reproductions violate the approved font-resolution guarantees. The user explicitly requested implementation of the complete plan, including preservation across non-font edits, rejecting ambiguity rather than adding operation IDs, and correcting separators within v2. These artifacts transcribe that approved plan without expanding it.

## Goals / Non-Goals

Goals: correct draft selector capture, deterministic retained-operation matching and Unicode mandatory breaks. Non-goals: new APIs, persisted fields, profile versions or changes to unrelated work.

## Decisions

- Capture unresolved `(component identity or root, local item ID)` before font preparation and extract only those items afterward. Selector keys remain the v2 storage format because all unresolved equal selectors in one preparation select the same binding.
- Before replacing draft operations, align existing steps to new operations in two passes: full serialized structure, then a font-intent projection. Projections retain operation kind, explicit target identifiers and selector presence/value, including component-local text IDs; omit content, typography, paint, timing and transforms. Each old step is consumed once. Match equivalent binding candidates in prior order; reject differing candidates before any publication. This avoids index coupling without a new operation-ID contract.
- Keep core draft matching inside assets/store. Root text uses an empty component scope; component IDs disambiguate local text IDs. Failed matching leaves existing drafts intact. Existing v2 drafts need no migration; corrupted or unavailable recorded content still fails integrity checks.
- Resolve bidi over original Unicode paragraphs, shape between hard separators, and retain original global byte clusters. BreakOpportunity::Mandatory forces a line; Allowed only informs wrapping. CRLF is one explicit break even across styled runs. Line separators do not reset paragraph direction; synthetic text-end does not add a line. This is the user-approved narrow correction to v2, not a new profile.

## Risks / Trade-offs

Without stable operation IDs some edited actions are indistinguishable: reject differing bindings rather than guess. Existing drafts already affected by a past collision cannot recover unknown lost intent; valid recorded hashes remain authoritative. Separator-containing text changes output as explicitly approved; ordinary-text regressions must remain stable.

## Verification and rollout

Add regression tests before implementation, including root/component collisions, inserted/reordered/non-font-edited actions, ambiguous/equivalent bindings, all separators and native intent parity. Run all repository-mandated checks and verify/archive this change. No persisted migration or cross-language field update is needed. Rollback is a binary rollback retaining the same readable formats; it restores the old separator defect.

## Approval

The user's explicit `PLEASE IMPLEMENT THIS PLAN` request approves this design and matching delta requirements, including the narrow v2 correction. No additional approval is required for these planned fixes.
