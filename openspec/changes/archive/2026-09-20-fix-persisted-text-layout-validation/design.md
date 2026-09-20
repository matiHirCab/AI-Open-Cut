## Context

The shared scope validator checks root text documents but omits TextStyle validation. Current and retained snapshots therefore accept negative tracking and missing fitting bounds, and direct rendering can allocate artifacts for them. Retained draft numeric validation exists, but structural layout failures occur during Serde decoding and the generic JSON error conversion maps them to INTERNAL_ERROR.

## Goals / Non-Goals

**Goals:** Enforce canonical layout validation at every persisted root/history boundary and return precise INVALID_ARGUMENT errors for malformed layouts before publication or rendering side effects.

**Non-Goals:** Broaden validation of layout-absent legacy data, replay stale drafts, change schema 21 or public types, alter geometry/fitting/encoding/limits, add dependencies, or change unrelated error classifications. Preserve both previous archives.

## Decisions

### Shared opt-in root validation (P1-P3, R1)

Extend the existing Text item branch in validation's shared scope traversal: validate the document as today, then call validate_text_style only when layout is present. Existing locked loading traverses current and retained undo/redo snapshots before font publication; renderer evaluation also reaches this validator. Hidden root text must be included. Retain existing component and draft validation. This selects one canonical owner under ADR 0003 without adding edges.

Rejected alternatives: checking only store loading misses direct Renderer inputs; checking only rendering permits invalid state/history to reopen; unconditional root style validation retroactively tightens layout-absent legacy behavior.

### Dedicated structural failure classification (E1-E3)

Replace only TextStyle.layout's present-value deserializer with a layout-specific helper. Decode the same strict TextLayout type and return Some on success; omission remains None and explicit null remains invalid. Wrap decoding failures using a reserved private anchored prefix owned by error, accessible through the existing model-to-error edge. CoreError's serde_json conversion recognizes only this exact prefix, removes it from the public message, and returns non-retryable INVALID_ARGUMENT. Unmarked errors retain their existing mapping. Do not inspect error text for field-name substrings or accept invalid values into public model types.

Test marker propagation through buffered Project decoding, History, internally tagged TimelineItem/EditOperation and EditDraft; nested Serde error strings must preserve classification. Test that marker-like user text or unrelated errors cannot trigger classification. No marker becomes a public field or serialized valid-state value. Human-readable messages remain bounded by existing input/error behavior; this change adds no filesystem paths or resource lookups.

Rejected alternatives: globally translating JSON errors changes unrelated contracts; guessing from names such as layout is ambiguous; parsing raw documents in persistence duplicates domain ownership; replaying drafts changes stale-revision behavior.

### Publication and compatibility (P2-P3, E2-E3, R1)

Keep existing locks, revision checks, recovery and transactions in their current order. New validation runs before candidate publication, not before recovery of an already-committed transaction. Do not mutate invalid input to repair it. Snapshot complete authoritative inventories and bytes on failures. Maintain existing old-schema layout restrictions and future-version rejection. Valid legacy/advanced projects and stale drafts retain their current behavior.

## Verification design

Write red regressions before fixes. Persistence matrices cover current/hidden/undo/redo invalid numeric layouts; structural null/wrong-type/unknown-field/enum failures cover current/history and retained add/update/component payloads. Drafts run during schema-20 migration and schema-21 reopen. Compare project/history/draft/font bytes and inventory, and check exact code/retryability. Use existing fake process/artifact adapters for frame/range/export preflight to prove there is no destination inspection, workspace allocation, write, publication or FFmpeg execution. Do not repeat the expensive malformed fit_box native probe. Native/headless and MCP tests must independently check the exposed error envelope.

Run all required Rust, native, bridge, Python and OpenSpec gates serially with Rust 1.97.0, verified temporary FFmpeg 7.1.1 and the working pinned OpenSpec CLI cache. Keep full logs outside the repository. Recheck original fourteen acceptance scenarios and both prior corrections for regressions. Verify conformance before synchronization/archive; protected Moon may reject only this active change before archival, and both final protected and strict gates must pass afterward.

## Risks / Trade-offs

- Nested deserializers could lose the marker: cover actual Project/History/EditDraft decoding before declaring classification complete.
- Broad conversion could alter unrelated errors: require exact anchored prefix matching, control fixtures and no field-name heuristics.
- Validation could change legacy data behavior: guard root style validation on layout presence and retain explicit legacy controls.
- Early publication could leak files: observe fake I/O events and compare complete durable file inventories.

## Migration Plan

No new migration or schema version. Schema-21 reopen and existing migration loading gain validation. Invalid files remain untouched for explicit repair. Reverting the corrective code requires no data rewrite or downgrade. Preserve both existing archives; archive this change separately after conformance and required gates pass.

## Open Questions

No semantic decision remains. Concrete artifact approval was received on 2026-09-20 ("yes").
