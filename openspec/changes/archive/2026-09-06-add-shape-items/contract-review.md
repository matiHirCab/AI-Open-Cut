# Shape contract review

Designated reviewer: `@matiHirCab` (repository CODEOWNERS and canonical ownership catalog).

The implementation and conformance checks are complete. The user approved this final contract review on 2026-09-06 with “Approve”, authorizing specification synchronization, archival and the final protected Moon check.

## Public changes

- New additive MCP `timeline_add_shape`, backed by existing headless `edit` with `operation: "add_shape"`; aliases work in ordered atomic batches.
- Closed seven-variant geometry union; required explicit nullable `fill` and `stroke`; existing bounded vector paints/strokes and structured paths. At least one paint is required; lines require stroke only.
- `timeline_update_item` / `update_item` gain complete geometry replacement and nullable fill/stroke patches for shapes. Existing fields and legacy operations retain their meaning.
- Shape records are accepted throughout project, component, batch and draft schemas. Protocol major remains 1. Editing advertises `shape_items`; complete renderer readiness additionally advertises `shape_rendering`.
- Persisted schema becomes 14, with atomic current/history migration from 1–13. Shapes in older source schemas are rejected; older binaries reject schema 14; no automatic downgrade or legacy rectangle conversion.
- Resource work limits are explicit: authored paths 4096 commands; compiled vocabulary 8192 commands; flattened/dash work 65536 segments per shape and 1048576 per scene; adaptive curve depth 16 and target deviation 0.25 output pixels; existing raster-surface caps remain enforced.

## Files to review

- `contracts/shape-items-v1.json`: complete geometry vocabulary, limits and valid/invalid examples.
- `contracts/headless-protocol-v1.json` and `contracts/mcp-surface-v1.json`: discoverable capabilities and exact tool schemas/annotations.
- `contracts/contract-ownership-v1.json` and `.github/CODEOWNERS`: synchronized consumers/owner.
- `docs/shape-items.md`: exact client semantics and runnable input example.
- `verification.md`: scenario mapping, native evidence and executed check results.

The large MCP snapshot diff comes from inlining the new union into nested project/component/draft schemas. Changes are confined to thirteen affected tool definitions; the new standalone tool uses the existing write annotations. Existing vector primitive catalog values and provider protocols are unchanged.

Approval of this completed contract surface has been recorded for final archival and policy validation. No commit, push or merge has been performed.
