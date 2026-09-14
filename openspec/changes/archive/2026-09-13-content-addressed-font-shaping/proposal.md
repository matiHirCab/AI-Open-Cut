## Why

Issue [#33](https://github.com/matiHirCab/AI-Open-Cut/issues/33) requires recorded font hashes and stable text layout after reopening. Today font paths, family lookup, fallback and styled sibling lookup are resolved at render time; rich-text wrapping sums individual character advances, so it does not use a canonical shaped glyph sequence. Prerequisites #32 and #82 are closed and their rich-text and centralized ownership implementations are present.

## What Changes

- Resolve text fonts once in editor-core, copy verified bytes into content-addressed managed storage, and persist face hashes and a versioned layout profile for root/component text, including fonts required by slot values and retained drafts.
- Shape effective rich text deterministically from pinned faces; reuse canonical glyph positions, clusters, line breaks and bounds across frame/range/draft preview and export.
- Migrate current projects and every retained undo/redo snapshot atomically to schema 19; retain font content through the centralized ownership graph and reject missing/corrupt pinned content instead of substituting a different font.
- Keep simple text/document and font selector request syntax, aliases, revisions and atomic edits. Add capability reporting and versioned fixtures for the new behavior.
- **BREAKING persisted/render contract:** schema 19 requires font bindings and selects a new versioned text-layout contract. Correct shaping can change kerning, ligatures, combining-mark placement and wrapping relative to schema 18. Migration preserves content and editing semantics, establishes the first pinned layout, and documents this one-time visual change. Older binaries reject schema 19; no downgrade is provided. Do not classify this rendering meaning change as an additive contract.

## Capabilities

### New Capabilities

- `font-resolution`: Safe, bounded content-addressed font resolution, deterministic shaping and stable lifecycle behavior.

### Modified Capabilities

- `project-persistence`: Atomic schema-19 font activation across current and retained snapshots.
- `media-assets`: Font reachability, integrity and collection through the existing ownership policy.
- `rendering-export`: Shared shaped-text semantics and explicit compatibility transition from legacy text layout.

## Impact

Core model, validation, assets, migrations, persistence orchestration, timeline/draft materialization and evaluated rendering are affected. Headless/bridge pass immutable font configuration into core and synchronize project/capability contracts without duplicating resolution. Update canonical contract ownership, governed Rust/TypeScript/MCP consumers, ADR 0003 and architecture tests for any new module edges, and packaging for a licensed deterministic default font and pinned shaping dependencies. Follow ADR 0002 with a new major text-layout contract and explicit schema-18-to-19 migration evidence; retain existing operation wire names and stable error codes.

## Non-goals

Font installation, network font download, a font marketplace/import endpoint, variable-font controls, color/bitmap font rendering, arbitrary OpenType feature controls, editing glyphs, new caption styling APIs, or extending rich-text syntax. Caption-only rendering is outside this change; existing caption behavior remains covered by regression tests. No implementation is authorized by this proposal until explicitly approved under AGENTS.md.
