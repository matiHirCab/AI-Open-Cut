## Context

TextItem currently owns text, fontSize, color, optional fontFamily/fontPath, TextStyle, common visuals and keyframes. RichTextDocument already exists as a closed run sequence for template slots. Component evaluation carries styled overrides into EvaluatedScene and renderer preparation. Schema 17 is current; all prior migration milestones remain applicable. Issue #32 extends this existing behavior to stored text items.

## Goals / Non-Goals

**Goals:** One canonical document representation; legacy request/output compatibility; atomic migration of current and retained state; shared evaluation; executable evidence for every scenario.

**Non-Goals:** The broader roadmap's grapheme range API and typography extensions, font hashing/new shaping (#33), caption conversion, new provider behavior, or UI controls.

## Decisions

1. Reuse `RichTextDocument { runs }` rather than introduce a competing span-index representation. A run is a span of literal Unicode text; no byte/scalar/grapheme indices are exposed. Preserve run order, whitespace, newlines and Unicode without normalization. Run boundaries do not introduce breaks. Item-level font, color and style are defaults; optional run bold/italic/color retain their existing slot semantics. The alternative indexed model would break the already published slot vocabulary and expand this milestone.
2. Add required persisted `document` and retain `text` as the exact run concatenation for protocol-1 consumers. Core validates equality on persisted input and updates both atomically. Current-schema documents cannot silently default. Legacy requests normalize to one unstyled run; a document request recomputes the projection. Creation requires exactly one of text/document; updates accept at most one, with omission preserving both. Explicit null and simultaneous fields are invalid. Replacing text deliberately clears run styling while retaining item typography. Item-level style-only edits retain run overrides. Component-create/update requests with legacy plain text-item records also normalize an omitted document in core; native persisted schema-18 decoding still requires it. The alternative removing `text` would break existing readers.
3. Reuse existing core validation helpers for slot documents and add text-target checks: 1-256 runs; concatenated text 1-4096 UTF-8 bytes; existing color, scalar, finite-value and project complexity bounds. Do not impose a new project-wide text bound on previously valid legacy items. Validate hidden/unused components as well as root items. Closed transport schemas reject malformed shapes without duplicating core semantics. No run path/font/link/expression properties are introduced; markup-like text remains literal. Existing item font path checks still apply.
4. Normalize in editor-core mutation and component-binding paths. Existing add/update names and protocol 1 remain; advertise `rich_text_documents`. Carry optional document fields through headless requests, bridge Zod schemas, batches, draft materialization and output schemas. Splitting, duplication, movement and trimming preserve document content. Template text slots yield one run, rich-text slots replace the effective document, and neither changes the base definition. Copying raw TextItem values for component creation must also retain documents.
5. EvaluatedScene receives stored runs unless an effective slot override replaces them. For documents without effective per-run styling, use the legacy text preparation path; otherwise reuse styled slot preparation. This preserves legacy pixels and default-font behavior. Missing styled font faces retain DEPENDENCY_UNAVAILABLE. All render entry points share the evaluator, bounds and resource safety. No promise of cross-machine font stability is added before #33.

## Migration Plan

Introduce schema 18. Validate each source current/undo/redo snapshot under its declared version before conversion, including nested component text. Schemas below 18 reject a text-item document field instead of laundering future content; existing rich-text slot values remain legal at their established milestones. For schemas 1-17, apply previous migration steps and synthesize exactly one unstyled run from each text item's original string. Preserve all other text fields, item IDs, ordering, timing, revisions, transforms, media and provenance. Update every retained snapshot in the existing locked, recoverable transaction and validate the complete result before authoritative publication or managed-asset writes.

Schema-18 input requires matching document/text. Schema zero and unknown future versions fail using established errors, including INTERNAL_ERROR for future versions. Reopening performs no redundant migration. Durable draft operations that use simple text remain accepted and materialize through the same normalization; do not rewrite or drop drafts. Fault-injection tests cover every existing publication phase. Recovery selects a complete old or new generation. There is no downgrade; rollback requires a complete pre-migration backup with its matching assets/history.

## Contract and Verification Plan

Follow ADR 0002 and contracts/contract-ownership-v1.json, updating canonical persisted/request/response/MCP/capability fixtures and every governed Rust/TypeScript consumer. Keep fixture-only roadmap concepts inactive. Obtain @matiHirCab contract review before completion. Python provider contracts remain unchanged; run hermetic worker regression checks as required by repository policy.

Map every delta scenario to named automated tests in verification.md: core model/mutations and slot precedence; standalone/headless/MCP/batch/draft parity; migration source rejection, mixed history, undo/redo, reopen and fault recovery; fixed-font legacy/styled render fixtures across frame/range/draft/export. Run Rust format, strict workspace Clippy and tests, bridge format/lint/typecheck/unit/contracts/integration/packaged smoke, relevant hermetic Python tests and OpenSpec validation. Record exact commands and outcomes. No automated-coverage exemption is proposed.

The repository's Moon preflight rejects active OpenSpec changes. Validate the active change with the pinned CLI; do not weaken that gate. Complete implementation checks and openspec-verify-change, archive with openspec-archive-change, then run the archive-only Moon OpenSpec gate and any checks blocked by preflight. A failed or unavailable required check blocks completion.

## Risks / Trade-offs

- Duplicate persisted text projection can drift: reject mismatches and centralize normalization; test all creation/update/copy paths.
- Styled-run preparation can alter legacy output: preserve the plain-text path and compare decoded fixtures before/after migration.
- Existing component overrides can shadow stored documents: specify replacement precedence and test independent instances.
- Migration can accidentally accept future fields: source-version checks run before default insertion or version updates.
- Run representation is intentionally limited: future typography additions require their own reviewed contract/spec changes.

## Open Questions

The user approved the run-based scope and input-conflict rule with these artifacts on 2026-09-09. Any newly discovered contract ambiguity requires an artifact amendment and approval before implementing the affected behavior.

## Implementation notes

Styled root text uses the existing affine composition path with an identity parent, preserving legacy transform/anchor and animation semantics while preparing individual runs. Semantically plain stored documents retain the legacy single-text renderer. Effective rich-text slots continue through the evaluator override map without rewriting stored text, preserving their published scalar-count bounds independently of the stored document byte bound. Native conformance covers both paths with fixed resources.
