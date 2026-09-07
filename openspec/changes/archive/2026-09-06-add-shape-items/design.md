## Context

Issue #28 activates the vector vocabulary implemented by #27. Current schema is 13; vector primitives are strict and reference-free. Core already owns timeline transactions, components, EvaluatedScene and renderer preparation. No active approved change existed at intake. The user approved this document and its deltas on 2026-09-06.

## Goals / Non-Goals

**Goals:** seven editable shape variants, shared deterministic rendering, bounded validation/raster work, typed agent access, and complete schema/history migration with conformance evidence.

**Non-Goals:** see proposal; notably no SVG parser, external resources, new animation/slot channels or desktop drawing UI.

## Decisions

1. Add a distinct ShapeItem using existing common visual properties and a closed geometry union. Retain legacy rectangle records/operations to preserve compatibility rather than translating them into new semantics. Reuse vector primitives unchanged, including strict object/string representation rules.
2. Use explicit geometry with required nullable fill/stroke. Creation defaults only common properties; geometry/style are explicit. Existing update_item accepts complete geometry replacement and nullable paint/stroke updates. The delta defines dimensions, vertex limits, star orientation and anchor behavior so client semantics are reviewable.
3. Normalize geometry within core to bounded path/drawing instructions; keep analytic unstroked bounds for anchors and separate padded stroke/raster bounds. Feed these into the existing graphics preparation and compositor through EvaluatedScene. Use deterministic adaptive curve subdivision and bounded CPU fill/stroke/paint evaluation within the core rendering layer. Use the existing lockfile version tiny-skia 0.11.4 for coverage rasterization of core-flattened paths. Core explicitly computes gradient interpolation and paint compositing; backend cap/join/dash/fill behavior receives independent conformance tests. This selects the approved third-party rasterizer alternative without adopting its gradient defaults.
4. Reuse existing transactions and item lifecycle, including component definition replacement, draft materialization and alias resolution. New headless/MCP handlers only translate typed input/output. Shapes are overlay visuals without audio or transition-endpoint semantics. Existing common-property slot bindings retain their current allowed target rules; new shape-specific bindings are excluded.
5. Introduce schema 14 and validate source schema before migration. Relabeling schema 13 is sufficient for formerly valid documents; converting legacy records is unnecessary and risks pixel changes. Enforce shape prohibition in older current/history/definition data before relabeling.
6. Add canonical shape fixtures with structural versus semantic invalid cases and named numeric/work limits. Update ownership, headless, MCP and capability catalogs plus every governed consumer in one change. Keep protocol major 1; schema 14 requires a capable binary. Request designated CODEOWNER review before merge.
7. Add native shape golden coverage using the established fixture infrastructure and deliberately reviewed references. Cross-intent equality alone is insufficient: separately assert geometry, gradients, stroke and occlusion expectations. Keep existing required CI boundaries intact.

## Risks / Trade-offs

- Gradient interpolation and stroke defaults can differ across backends → assert linear-light premultiplied colors, dash phase, caps/joins and fill rules independently.
- Finite geometry can cause expensive raster work after transforms/instance expansion → preflight command/segment/depth and existing surface bounds before allocation or I/O; fail rather than degrade.
- Shapes can be hidden in unused definitions/history → validate the entire envelope and source-version eligibility before migration or transaction publication.
- New union variants affect many consumers → canonical fixture parity, exhaustive Rust handling, real source/packaged workflows and designated owner review.
- Approval may change geometry/API choices → synchronize all artifacts before implementation and obtain approval for changed behavior.

## Migration Plan

Validate old current/history snapshots in their original versions; run deterministic existing migrations and the version-only 13-to-14 step; validate the complete new envelope; publish once under the existing project lock and recoverable generation mechanism. Test every existing fault-injection phase and future/malformed history. New projects use schema 14. Rollback requires restoring a complete prior generation with a compatible binary; no automatic schema downgrade is provided.

## Verification and Traceability

Each task references named requirements and scenarios. Cover structural and semantic boundaries in Rust and TypeScript, transactional and migration behavior in core/headless, real MCP source and packaged workflows, and shared native render evidence. Run strict OpenSpec validation, formatting, Clippy, workspace tests, TypeScript typecheck/lint/unit, contract parity, integration, packaged smoke and relevant hermetic worker checks. Apply openspec-verify-change, reconcile all findings, then archive with openspec-archive-change and run the protected Moon validation after the active directory is removed. Active-change planning validation uses the pinned direct CLI; the protected Moon preflight intentionally rejects active changes.

## Open Questions

User approval received on 2026-09-06 for the complete proposed scope and contracts.
