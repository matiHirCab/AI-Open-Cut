## Context

Issue #23 builds on archived `add-component-definitions` (schema 11). Root instance placement and evaluation belong to #24. ADR 0003 owns validation in core; ADR 0004 requires stable typed property bindings. The existing preparatory fixture demonstrates `text.document`, Unicode scalar constraints and 128 slots per component but does not define the other seven runtime value shapes. This proposal supplies those choices for approval.

## Goals / Non-Goals

Implement bounded typed slot definitions and effective-value validation with complete persistence and agent APIs. Preserve root rendering. Exclude dynamic expressions, slot forwarding, arbitrary property paths, root instantiation, general rich-text layout and renderer changes.

## Decisions

1. **Persist explicit schema-12 fields.** Component definitions gain required `slots: []`; nested instances gain required `slotValues: {}`. Keep request compatibility through optional fields: create defaults to empty; update omission preserves slots. Explicit update slots replaces the list. An omitted nested-instance value map defaults to empty on request only. An alternative of silently defaulting missing schema-12 fields would conceal corrupt persistence; reusing schema 11 would misrepresent format compatibility.

2. **Use closed tagged values and kind-specific constraints.** Every value is `{type,value}`. Text, color and enum carry strings, number/duration carry numbers with distinct validation, Boolean carries a Boolean, asset carries a managed project reference, rich_text carries a bounded flat run document. The template-slots delta is authoritative for exact bounds and fields. Flat runs avoid recursive document expansion and unsafe markup. An untyped JSON union was rejected because it cannot distinguish enum/text/color or duration/number reliably. Rich text remains typed data; a new renderer is outside this issue.

3. **Bind only to stable local properties.** Use the closed property mapping in the delta. One slot writes one target/property and duplicate writers fail. Resolve IDs only within the owner; never accept a caller-provided JSON path or scope. Derive effective candidate properties from defaults and then instance overrides and run existing domain rules against those candidates without modifying stored tracks. Rich-text document structure is validated directly; it does not require adding a rich-text payload to TextItem now. Required values can be omitted only on an uninstantiated definition; optional absence retains the base property. The alternative of materializing values into shared definitions would corrupt independent instances and complicate undo. No renderer may flatten rich text to pretend to render it.

4. **Retain ordered atomic editing.** Add `component_define_slots {componentId,slots}` to both core operation unions and thin headless/MCP variants. Resolve earlier component aliases but keep local slot/target IDs literal. Validate all definitions and incoming instances after each operation, including defaults hidden by overrides. Slot replacement cannot modify bindings on locked tracks; compare old/new affected slots, permitting unchanged ones. Removing a target or slot referenced by an instance fails rather than deleting dependent data. Existing instance values can be replaced through full component_update tracks. No separate per-instance edit API is necessary before #24. A combined component_update with explicit slots supports coordinated local-track and slot replacement.

5. **Keep one validation owner.** Extend core model/validation/timeline/migrations/assets within existing ADR edges. Effective values require validation of target property combinations, local duration, media trims and nested source timing, not just primitive type acceptance. Bound collections and aggregate text before lookup/traversal; use deterministic iteration for failures and results. Add no bridge/domain validation duplication. Reuse core validation in load, drafts and direct rendering before I/O. Assets in defaults and overrides participate in existing integrity and ownership traversal. Error messages identify safe field/slot context without logging full user text or paths. Unknown slot/target/component references use ITEM_NOT_FOUND, safe missing asset IDs use ASSET_NOT_FOUND, malformed/type/constraint/resource inputs use INVALID_ARGUMENT, locks use TRACK_LOCKED and stale revisions remain retryable REVISION_CONFLICT.

6. **Govern runtime adoption separately.** Add `contracts/template-slots-v1.json` and ownership consumers; update component, headless, MCP and capability catalogs manually with mirrored structural evidence. Keep motion-graphics-v1 fixture_only and document its relationship to runtime shapes. Extend schema-12 responses and compatibility fixtures. This avoids retroactively changing preparatory fixture meaning or marking unrelated motion-graphics concepts active. No provider protocol changes; protocol 1 additions remain additive. @matiHirCab reviews public/persisted consumers and canonical evidence.

## Risks / Trade-offs

- Required slots can invalidate existing instances: reject whole edits and document clearing/replacing overrides before incompatible definition changes.
- Rich-text storage precedes rendering: retain existing root rejection of component instances and advertise slot support separately from rendering.
- Multiple properties can interact: validate the complete derived candidate, not each bound value in isolation, and cover duration/source-media interactions.
- Many individually small values can exhaust resources: enforce both local counts and project-wide text/slot budgets.
- Assets can be retained longer than visible use suggests: include defaults even when overridden and use existing history/draft retention rules.

## Migration Plan

After approval, add the deterministic 11-to-12 migration after the existing supported chain. Migrate and validate all current/undo/redo documents under lock before one existing recoverable transaction. Preserve schema-11 component content and root values. Add fault-injection evidence at supported transaction phases, oldest-schema and mixed-history cases, malformed schema-12 required fields and future versions. Reopen performs no further rewrite. Mutation rollback uses existing transactions; no schema downgrade exists. Older binaries must reject schema 12, and undo stays within the current schema.

## Verification Plan

Exact-boundary conformance requires serde_json's `float_roundtrip` parser feature in editor-core: without it, an inclusive floating constraint at 9007199254740991 can round differently on persisted read and reject a previously committed duration. This implements the approved finite-value and deterministic-reopen rules without changing wire identifiers, limits or serialization format.

Map every scenario in the five delta specs to named automated tests in verification.md during implementation. Cover each kind and exact limit independently, multi-property invalid candidates, definition changes with incoming references, asset-only retention, current/history migration, failed batches with byte comparisons, drafts/direct rendering and real source/packaged transport workflows. Run the exact commands in tasks.md. The archive-only repository policy cannot pass while this proposal is active; perform focused strict OpenSpec validation during authoring and the full Moon gate after verified archival. Do not weaken that gate.

## Open Questions

No implementation choice is left implicit: approval includes the schema version, flat rich-text format, seven property identifiers, limits, default rules and deferred rendering boundary above. The specification and completed implementation/public contract review were explicitly approved in this task on 2026-09-05.
