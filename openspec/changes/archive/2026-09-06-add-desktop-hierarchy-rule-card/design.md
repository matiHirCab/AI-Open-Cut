## Context

Issue #26 closes the first scene-graph milestone. Core already owns group parenting, signed z-index, component definitions, typed slots, root instances, lifecycle edits and schema 13 persistence. Dependencies #25 and #14 are closed. Desktop currently has persistent GPUI panel entities but no project session and no core dependency. Existing specs for timeline-editing, template-slots, component-evaluation and render-regression-fixtures remain authoritative.

## Goals / Non-Goals

Goals: make existing agent-created projects inspectable, edit root parent/z-index through core, and demonstrate the six-layer/three-instance milestone through lifecycle and native render evidence.

Non-goals: see proposal. In particular, no new component-local editing API, preview player, renderer semantics, slot type or schema is needed.

## Decisions

1. Add a desktop session shared by shell/panels, using the public EditorCore facade. Startup accepts paired `--project-store <directory>` and `--project-id <id>` arguments; omission gives the empty state, partial or unknown arguments give a usage error. Use core PathPolicy to configure access. A separate session/controller layer supports automated tests without a native window. Keep GPUI event handlers thin and serialize outstanding writes; run blocking project I/O off the render path and discard stale load results. Alternative: a new headless subprocess protocol would add orchestration without needed isolation for this minimal native UI.

2. Build a presentation-only hierarchy projection from a validated immutable Project. Scope and instance-path keys disambiguate repeated local IDs. Expand component occurrences lazily; cap displayed rows at 4096. Show parentage and track identity independently because cross-track parentage does not redefine compositing. Local definition rows are inspectable but read-only. Alternative: eagerly flatten the component DAG would duplicate evaluation and risk excessive expansion. No UI projection is used as renderer input or domain validation.

3. Send typed ItemSetParent and ItemSetZIndex edits with the loaded revision; undo/redo follow the same session path. i32 parsing is input decoding only. Root group choices are candidates, with core making the final legality decision. Display core code/message/retryability and preserve committed state on failure. Offer refresh after conflicts, never automatic replay. Alternative: optimistic local mutation would require duplicating rollback and validation. Reparenting retains existing local-coordinate semantics and can visibly move content.

4. Keep a separate deterministic rule-card fixture rather than replacing flat-scene-av-v1. Use six children: background, accent strip, number text, title text, body text and managed icon. Text slots vary number/title/body, an asset slot varies icon, and a numeric slot varies opacity. Three simultaneously visible instances share one root group, with deliberate overlap making z-index observable. One root group translation supplies the movement oracle; fixed local sine-wave audio makes range/export audio comparisons meaningful. Creation/alias results are mapped to deterministic fixture identities for references, without adding caller-defined IDs to runtime creation APIs. Alternative: a numeric-to-text slot would expand the existing closed binding contract and is unnecessary for this issue.

5. Add core semantic/lifecycle tests and native production render checks, plus headless/MCP integration using existing standalone and batch operations. Reuse existing comparison helpers and dependency policy. Extend the existing required native conformance entry point to invoke the rule-card case, so the protected CI command remains effective without weakening or bypassing its exact workflow policy. Keep flat-scene performance capture/report schema unchanged. Rule-card references carry version, hashes, safe relative paths and explicit fixture/font/timestamp identity; reject malformed references before rendering and require explicit update mode to replace them. Alternative: parity only between newly rendered outputs would miss coordinated drift.

## Risks / Trade-offs

- Desktop platform dependencies may prevent native window verification on a host. Build and controller tests remain required; lack of actual manual verification is a reported completion blocker, never evidence of a passed workflow.
- Scope could grow from inspection into a full editor. Keep root-only mutation controls and the existing preview panel boundary explicit in documentation.
- Fixture references can vary with tools/fonts. Pin the declared font identity, use decoded comparisons and current tolerances, and keep reference updates explicit and reviewable.
- Native conformance cost grows across lifecycle states. Use a short fixed clip and a bounded timestamp set while covering all states.

## Migration Plan

No persisted schema or public request/response/catalog semantics change. Existing migration tests for current state and retained history must continue passing. Unknown future schemas continue to fail in core. Rolling back this desktop/test-only change requires no project downgrade. No privacy or network feature is introduced; synthetic assets are imported through core, and fixture paths remain confined to controlled fixture/temp roots.

## Verification and Traceability

Map each desktop scenario to controller/projection tests and a native manual checklist covering load, expansion, selection, parent, z-index, conflict, refresh, undo/redo and reopen. Map each rule-card scenario to core semantic/lifecycle and native conformance tests; existing MCP operations get integration coverage for alias construction and rollback. Record test names/results in tasks or verification evidence. No normative requirement has an automation exemption; native interaction additionally requires a manual smoke check.

## Open Questions

The user approved the minimal startup entry path and root-only control scope on 2026-09-06. If implementation reveals missing core behavior or a required public/schema change, amend these artifacts and obtain approval before that implementation.
