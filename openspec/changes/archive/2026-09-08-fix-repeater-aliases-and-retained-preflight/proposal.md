## Why

Review of the archived repeater correction found that batch updates accept source aliases but do not resolve them, hidden/inactive and unused repeaters bypass complete geometry budgets, and flat evaluation still deep-clones an obsolete repeater snapshot. These defects violate the existing alias, retained-content validation, and bounded-allocation guarantees.

## What Changes

- Resolve update_item itemId and repeater.source.id through the existing ordered batch alias resolver.
- Validate complete root expansion and each retained definition independently of visibility, including effective slots, clocks, nested repeaters, final matrices and geometry budgets before generated-layer materialization.
- Share measurement and checked accumulators with refinement, while publishing only visible occurrences.
- Remove the obsolete flat repeater expansion path and its unconditional deep snapshot.
- Add regression coverage across core, headless, source MCP and packaged smoke, and correct the archived verification record with a dated follow-up note.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `repeaters`: Explicit validation domains for hidden/inactive and unused content, shared measurement, and no redundant ordinary snapshot.
- `timeline-editing`: Source alias resolution when replacing repeater descriptors in batches and drafts.
- `agent-bridge`: End-to-end evidence for aliased repeater replacement and atomic failures.

## Impact

Changes belong to editor-core alias resolution/evaluation and focused transport tests, documentation and OpenSpec artifacts. This is a conformance fix: schema 17, protocol 1, public wire shapes, capability identifiers, deterministic IDs, error codes and persisted representations remain unchanged. No new dependency or migration is needed.

## Non-goals

No new source kinds, timing controls, rendering fallback, public fields, schema version, or generic alias syntax. Preserve all existing uncommitted issue-31 work.

## Approval Status

The user explicitly approved these concrete proposal, design, three delta specifications and tasks on 2026-09-08 ("Approve"), before executable implementation edits.

Subsequent inspection on 2026-09-08 identified an incorrect draft-alias assumption in the design and timeline delta: native drafts have literal-ID EditOperation lists, with no batch alias envelope or resolution. A narrowly revised compatibility requirement preserves that behavior and tests drafts with canonical IDs obtained from batches. The user explicitly approved this correction on 2026-09-08 ("Approve"), before executable edits. The remainder of the original approval is retained.

## Follow-up review — 2026-09-08

The recorded gates remain historical evidence, not proof of complete conformance. A subsequent review reproduced 4,369 accepted transition facts from generated copies and audio introduced into a repeater source through an effective asset slot. Those scenarios were absent from this change's verification. [fix-repeater-transition-budget-and-effective-audio](../2026-09-08-fix-repeater-transition-budget-and-effective-audio/proposal.md) addresses both findings with fresh verification; this predecessor remains archived.
