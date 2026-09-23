## Context

Issue #37 and milestone 2 in docs/motion-graphics-implementation-plan.md require minimal inspectors and a static frame composed from native primitives. Desktop already has a revision-bound EditorCore session, selection identities and read-only component occurrences. Living shape, rich-text, advanced-layout and rendering requirements remain authoritative.

## Goals / Non-Goals

Goals: edit selected root vector/text properties through existing operations; demonstrate all named motifs at three resolutions, through history and render intents. Non-goals are listed in proposal.md, including new contracts and component-local editing.

## Decisions

1. Extend the existing inspector with labeled fields and Apply/Reset actions. Shapes expose rectangle/rounded-rectangle/ellipse dimensions, rounded radii, solid fill color and stroke color/width; grids expose spacing, angle and line color/width using existing descriptors and Transform2D rotation. Other geometry and non-solid paints are displayed read-only and preserved. Text exposes literal content, font size, base color, selected run bold/italic/color, tracking, line height, layout bounds/fit and existing stroke/shadow layer fields. Show pinned font identity read-only. Each action patches a clone of the selected authoritative descriptor/style, preserving unedited fields. A raw JSON editor was rejected because it hides usable controls and risks accidental replacement.
2. Bind local input drafts to selection identity and source revision. Apply submits one existing typed core edit; parsing only handles representation, while core enforces semantic/finite/complexity limits. Refresh/history/selection changes rebuild drafts; conflicts never retry automatically. CoreError code/message/retryability remain visible. Component occurrences remain read-only. Duplicated UI domain validation and optimistic local project mutation were rejected to preserve ownership and transactional behavior.
3. Add a separate rules-screen recipe and core example, following the existing rule-card fixture structure. Use a 1920x1080 canvas, 10 fps, one-second duration and samples at 0/500/900 ms. Arrange three outlined cards across the upper portion with accent bars and corner brackets; put diagonal grid and concentric circles behind layered EVERY./SINGLE./ONE. text below. Commit exact positions, colors, font hash, layer order and independent expected geometry in the recipe/reference metadata. Generate only a local test tone; graphic panels are never imported. Use an original composition because the issue provides no source frame. Reusing the old rule-card baseline was rejected because it would change unrelated evidence.
4. Construct and edit with existing typed standalone and aliased batch operations; exercise the same edits through MCP integration. Verify original/edited/undone/redone/reopened scenes and failure rollback. Add references at 1920x1080, 1280x720 and 960x540 using the existing shared evaluator and native golden gate. Semantic expectations and reviewed images must detect shared drift across render intents. Cache cold/warm samples must agree. Tests must not introduce renderer workarounds.
5. Add `fontSize?: integer` to `update_item` because the existing edit operation cannot change a stored text item's size. Limit it to 1–1000, require a text target, and validate in editor-core within the existing atomic transaction. Omission preserves size; supplied values preserve document, font binding, style and item identity. Synchronize headless request declarations, MCP standalone and batch schemas, capability/catalog fixtures, and parity tests under ADR 0002. Existing requests remain valid. Rebuilding the item through delete/add was rejected because it would change identity, references and history semantics.

## Risks / Trade-offs

- Style replacement can reset unrelated layout or spans: clone authoritative values and test preservation explicitly; content replacement intentionally follows existing simple-text reset semantics.
- Font and native renderer availability affects pixels: record deterministic dependencies and fail required checks if absent. Never substitute an unrecorded system font.
- Broad fixture coverage is expensive: keep the scene static and one second long, retaining all required lifecycle/resolution samples.
- Manual GPUI evidence may be unavailable in automation: record that limitation as a completion blocker until a reviewer performs the documented workflow; unit tests do not prove native interaction.

## Migration Plan

The request change is additive within the existing `update_item` operation. Existing projects retain the current migration and history paths; no persisted migration is needed. Rollback can stop sending `fontSize`, with no stored-version rollback. Update governed contract artifacts and consumers together; obtain designated CODEOWNER review. Any further contract/schema change requires another artifact amendment before implementation.

## Security and failure handling

No external resources, paths, SVG text or FFmpeg expressions enter inspector values. Core remains responsible for managed font ownership, path safety, locked tracks, references and all numerical bounds. Invalid input and failed edits preserve authoritative state/history. Fixture generation targets a fresh directory and refuses overwrite. Reference updates remain deliberate, hash-validated and separate from routine conformance runs.

## Open Questions

The initial scope and additive request-field amendment were approved in the task on 2026-09-22.
