## Context

Issue #29 depends on vector primitives (#27) and EvaluatedScene routing (#13), both represented in the current living requirements and implementation. Schema 14 supports ShapeItem and tiny-skia rasterization. There is no SVG ingestion operation. The root AGENTS.md requires explicit artifact approval before code changes.

## Goals / Non-Goals

Goals: provide bounded offline SVG ingestion as one editable item, preserve atomic history and deterministic rendering, and keep validation inside editor-core. The exact accepted subset and budgets are specified in specs/svg-ingestion/spec.md.

Non-goals: browser-complete SVG, font/text support, external resources, CSS, gradients, reference expansion, SVG transforms, source editing, or new slot/animation properties. Callers can use the existing item Transform2D and convert unsupported artwork to the documented subset externally.

## Decisions

1. Use a structural XML parser with entity/DTD resolution disabled and core allowlists. Validate byte size before parsing and bound depth, nodes, attributes and normalized work while traversing. Use quick-xml 0.39.4, pinned with default features disabled. Its borrowed read_event API performs no resource resolution; core rejects DocType, PI and general entities, checks element/depth/attribute budgets before growing its own collections, and decodes attribute escapes before allowlisting. Local source inspection covered reader configuration and events/attributes decoding. Regex sanitization cannot safely handle namespaces/escaping. A full browser or permissive SVG backend introduces unnecessary execution, resource and complexity paths. Reject unsupported content rather than strip it and produce misleading artwork.

2. Lower accepted geometry and solid presentation styles into existing ShapeGeometry/Paint/Stroke values and a normalized document viewport mapping. Preserve one SVG timeline item with ordered subdraws and one viewport clip; compose its opacity once across its internal image. Persist normalized values, not original XML. Raw-source persistence would force parsing again on reopen/render and make output depend on parser changes; raster-only persistence loses scale quality and introduces managed-file transactions. A canonical svg-ingestion-v1 catalog will spell out the exact closed normalized JSON records before consumers are changed.

3. Place normalized model records under model, parser/semantic validation under validation, mutation under timeline, schema changes under migrations, and pure subdraw evaluation under evaluated_scene. Render_artifact uses the existing shape raster machinery to prepare the evaluated SVG surface; render_plan consumes the evaluated visual result. Keep source/resource I/O out of the parser. If concrete implementation needs any new private-owner edge, update ADR 0003 and its architecture test in the same review before adding it; do not duplicate validation in transports or raster adapters.

4. Add add_svg to existing edit/batch unions, timeline_add_svg to MCP, and svg_items/svg_rendering to readiness reporting. Reuse common fields and result types, and expose svg only on creation. Include strict duplicate-preserving decoding for normalized SVG inside every existing buffering path. Bridge Zod handles contract structure; core owns SVG meaning and numeric validation. Do not introduce an import path or a separate asset API because inline input needs no path access or probe.

5. Advance schema 14 to 15 with a version-only legacy migration. Existing JSON layouts remain unchanged for older item kinds; older binaries intentionally reject 15. Apply migrations to all retained snapshots using existing locked recoverable transactions. Component definitions and drafts receive the same validation as root items, including hidden content. No asset records, integrity rules or GC references are added.

## Risks / Trade-offs

- Narrow subset rejects many exported SVGs → publish explicit accepted syntax and errors, with fixtures for every rejection category. Approval of this subset is required before implementation.
- XML parser allocation/entity behavior → inspect the chosen pinned parser and prove byte/depth/node limits with hostile fixtures before accepting the dependency.
- Curve lowering, viewport clipping and document opacity may differ from existing standalone shapes → compare independent geometric/pixel oracles and exercise nested transformed/retimed components, rather than relying only on agreement between render intents.
- Persisted normalized content is attacker-controlled on reopen → validate all nested records and budgets, including duplicate fields, unused definitions and history, before publication/render work.
- Raw markup could expose user data in diagnostics → emit category/position diagnostics without source excerpts and never log submitted markup.
- Schema upgrade prevents older binaries reopening projects → document backup-based rollback; no lossy downgrade.

## Migration Plan

After approval, establish canonical fixtures and tests, implement schema 15 migration plus normalization, wire core and transports, then add render fixtures and documentation. Obtain @matiHirCab contract-owner review. Validate affected surfaces, use openspec-verify-change, resolve all findings, synchronize/archive with openspec-archive-change, and run the protected Moon gate once the active change has been archived. The repository intentionally blocks protected Moon execution while active changes exist; use the pinned strict OpenSpec validator during authoring and do not bypass policy.

Rollback before implementation is deletion of these planning artifacts. After migration, restore a pre-migration project backup to use an older binary; retained history is not a downgrade path.

## Open Questions

The user approved the subset, limits and schema strategy on 2026-09-07. Parser package/version selection is an implementation investigation constrained by these requirements; if the parser cannot meet them, revise this design for approval rather than relaxing requirements.

## Verification traceability

Every scenario in the delta specification needs named automated evidence in the final verification report. Map ingestion/complexity to core SVG unit and shared fixture tests; transactions/lifecycle to facade, headless and MCP workflow tests; persistence to migration/history/fault-injection tests; rendering to the required native conformance suite and independent visual oracles. Include exact boundary and one-over-limit fixtures. Python worker behavior is unchanged; run its existing hermetic suite as the repository requires without introducing new worker semantics.

The existing schema-14 activation requirement is clarified as an intermediate migration, followed by schema 15 in the same transaction. This documentation reconciliation implements the already approved schema strategy and adds no behavior. Normalized records use an explicit local `offset` for positioned primitive geometry and a four-number viewBox; transports expose their closed structure while core validates semantics. Desktop hierarchy displays the new variant as SVG. Existing private-owner edges suffice; no architecture exception was added.
