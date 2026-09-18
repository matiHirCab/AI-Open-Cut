## Why

Issue #34 (MG-M2-08) requires grapheme-addressed styling and ordered text paint layers. Existing rich-text runs and pinned glyph rendering provide the foundations from issues #32/#33, but expose neither indexed spans nor multiple strokes or blurred shadows.

## What Changes

- Add optional, bounded grapheme-indexed style spans to existing rich-text documents, preserving run content and simple text operations.
- Add ordered solid fill, centered stroke and blurred shadow stacks at item and span level, with explicit inheritance, ordering, coordinates and resource limits.
- Render effective styles through the canonical EvaluatedScene and pinned glyph pipeline for frame, range, draft and export.
- Activate schema 20 through atomic migration of current state, retained history and applicable drafts. Preserve existing schema-19 text output when the new fields are absent.
- Add a versioned runtime contract/capability and update governed Rust/headless/MCP fixtures and documentation.

## Capabilities

### New Capabilities

None; extend existing capabilities.

### Modified Capabilities

- `rich-text-documents`: Indexed spans, layer data, validation, inheritance and reversible edits.
- `rendering-export`: Canonical span paint evaluation and bounded blurred glyph rasterization.
- `project-persistence`: Schema-20 activation and recoverable retained-state migration.
- `motion-graphics-contracts`: Governed additive styled-text runtime contracts and parity evidence.

## Impact

Core model, validation, timeline/draft/component evaluation, migration, shaping metadata and text artifacts; typed headless serialization; bridge schemas/capabilities; canonical contracts and parity fixtures; rendering goldens and user documentation. Font content selection remains governed by existing bindings. Add a pinned grapheme segmentation dependency if the current dependency graph does not provide the specified profile.

Public changes are additive optional fields and a uniquely named capability; existing requests remain valid. Persisted schema 20 is a compatibility boundary: older binaries reject it, supported snapshots migrate forward, and no downgrade is offered. Existing contract identifiers must not be repurposed. No provider protocol changes are planned.

## Non-goals

Per-span font families or sizes, gradient/image text fills, arbitrary effects, text-on-path, per-character animation, desktop styling controls, SVG/FFmpeg expressions, network resources and rewriting legacy documents into a new representation.

## Approval

The user explicitly approved this proposal, delta specs, design and tasks with “Approve” in this task. Implementation is authorized within this scope. Existing unrelated work must be preserved.
