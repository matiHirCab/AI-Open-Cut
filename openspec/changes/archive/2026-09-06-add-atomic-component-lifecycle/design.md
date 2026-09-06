## Context

Issue #25 builds on implemented definitions, typed slots and schema-13 instance evaluation. Core already owns ordered edits and store transactions; generic duplicate_items copies instances but has neither override input nor a single-result alias. This change was approved by the user on 2026-09-06.

## Goals / Non-Goals

Goals: complete atomic lifecycle coverage and add one bounded duplication edit, available to headless/MCP clients and batches. Preserve protocol 1, schema 13, existing errors, history and renderer evaluation.

Non-goals: definition deep copies, local nested-item edit operations, new renderer semantics, provider/UI work, arbitrary property paths or new dependency edges.

## Decisions

1. Reuse component_create, component_define_slots and add_component_instance for the existing lifecycle. A synonymous template_instantiate tool would duplicate an existing contract without adding behavior.
2. Add component_instance_duplicate {itemId,offsetMs,slotValues?}. It clones one root component instance on its existing overlay track, retaining component reference, parent, transform, visibility, z-index, trim, duration and rate. It appends to that track like duplicate_items, generates a fresh ID and reports only that ID. offsetMs is a required nonnegative safe integer; checked addition determines startMs. A single-source operation gives resultAlias an unambiguous meaning. Extending generic multi-item duplication with per-item override/alias arrays would broaden unrelated behavior and is deferred.
3. Omitted slotValues retains a deep copy of stored overrides; an explicit map replaces the whole map, matching component_instance_update. Empty clears overrides and then validates required/default rules. Patch merging and null-as-delete are excluded because they differ from existing update semantics. All eight typed kinds and special legal slot keys remain supported.
4. Add the operation to core edit and batch declarations, alias eligibility and source itemId resolution. Keep slot keys and values literal. Core timeline clones and validates the evolving candidate using existing validation; store performs revision checks, lock handling, transaction publication and history. Headless and bridge remain typed adapters, following ADR 0003 without new edges.
5. Add a component-lifecycle-v1 canonical catalog and register its consumers in contract ownership. Synchronize headless-protocol-v1 and mcp-surface-v1 and capability reporting; reuse existing component/slot/evaluation fixtures for value semantics. Require designated @matiHirCab review before archive.

## Risks / Trade-offs

- Override replacement can remove required values: document omission versus empty and test both defaults and missing required values.
- Cloning can exceed aggregate text or expanded scene bounds: validate stored graph/slot/text limits before project/history publication, and retain existing evaluated-scene preflight before render artifact preparation or backend execution (approved clarification in amendment.md).
- Aliases can accidentally rewrite slot strings: resolve only itemId and retain prototype-sensitive own map keys in transport regression cases.
- Duplication retains parent and stacking values: document append order and verify shared evaluation, instead of inventing implicit reparenting or z-index changes.
- New active changes intentionally fail protected archive-only CI policy: validate the proposal directly and run the required Moon gate after verified archival; never bypass policy.

## Migration Plan

No persisted fields change, so no schema migration is introduced. Reuse schema-13 current/history migration and unknown-future-version rejection tests, and add reopen/undo/redo evidence for duplicated instances. New instances remain readable by existing schema-13 runtimes. Clients discover the additive component_lifecycle capability before using the new operation. Rolling back the executable removes the new operation but does not require rewriting saved projects.

Failed operations and later failed batches leave the revision and project/history bytes unchanged. Use existing ITEM_NOT_FOUND, ASSET_NOT_FOUND, INVALID_ARGUMENT, TRACK_LOCKED and retryable REVISION_CONFLICT. Managed asset references and closed typed slots remain the only supported override surface; no raw expressions, executable SVG, filesystem paths or network resources are added. Existing diagnostic redaction stays intact.

## Verification and traceability

Map each delta scenario to automated tests in verification.md as tasks progress. Core tests cover exact copied fields, override isolation, failure atomicity, inclusive bounds, aliases, history and reopen. Transport fixtures distinguish structural rejection from core semantic errors. Source integration and packaged smoke exercise the full lifecycle, including a failed trailing edit. Evaluation/render tests compare duplication to equivalent explicit placement at representative frame/range/export timestamps and preserve source/definition bytes.

## Open Questions

The user approved this API and complete-replacement semantics on 2026-09-06. Final contract-owner review of the implementation and canonical consumers was approved by @matiHirCab on 2026-09-06 with "Approve final contract review and commit"; see verification.md.
