## Why

Issue #29 requires agents to ingest SVG safely through `timeline_add_svg`. Vector primitives and evaluated shape rendering are implemented, but no SVG parser or timeline operation currently provides this behavior.

## What Changes

- Add a core-owned, fail-closed static SVG subset with structural XML parsing, explicit resource budgets, and deterministic normalization into reference-free vector geometry before rasterization.
- Add an SVG timeline item, standalone `add_svg` / MCP `timeline_add_svg`, atomic batch aliases, lifecycle support, and component-local evaluation.
- Reject executable content, event handlers, external resources, font-dependent content, unsupported SVG features, and excessive complexity without silently dropping them.
- Migrate schema 14 to 15 atomically across current state and retained history; preserve legacy rendering and operations.
- Add canonical fixtures, capability reporting, security and lifecycle tests, and native preview/export conformance evidence.

## Capabilities

### New Capabilities

- `svg-ingestion`: Safe SVG subset, normalized persisted content, editing, migration, and shared rendering.

### Modified Capabilities

- `project-persistence`: Preserve schema-14 shape activation as an intermediate migration before the approved current schema 15. Existing timeline, rendering, media, vector and contract guarantees remain applicable.

## Impact

Core model, validation, timeline, migration, evaluated scene and rendering owners; typed headless edits; bridge schemas, registration and capabilities; canonical catalogs and parity consumers; documentation and render fixtures. Add a pinned XML parser after verifying bounded parsing and disabled entity/resource resolution. Keep primitive validation free of SVG strings.

Protocol additions remain version 1 and preserve existing clients and aliases. **BREAKING persisted compatibility:** schema 15 is unreadable by older binaries, which must fail closed; no automatic downgrade. Update contract ownership and obtain designated reviewer @matiHirCab approval.

## Non-goals

Full browser SVG compatibility; script execution; CSS stylesheets; fonts or SVG text; network/file/data URL resolution; raster image embedding; filters, masks, clipping paths, patterns, animation, use/reference expansion; SVG export; direct SVG-source patching; new shape animation or slot properties. Unsupported features are rejected, with no fallback rendering.

## Approval

The user explicitly approved this proposal, design, subset, limits and task list in this task on 2026-09-07 ("Approve"). Implementation is authorized. Final contract and conformance evidence remains subject to review.
