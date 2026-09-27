## Context

ADR 0002 assigns `contracts/mcp-surface-v1.json` as the single manually synchronized MCP authority. The bridge contract test imports it and compares every live registration after converting Zod schemas to draft 2020-12 JSON Schema and removing only schema descriptions. The catalog is 19,998,586 bytes and 360,953 lines; 144 complete tool schemas contain 66 duplicate copies, and larger repeated nested subtrees dominate the remaining volume. The working tree also contains an unrelated `.codex/config.toml` edit, which this change must preserve.

## Goals / Non-Goals

**Goals:** Reduce catalog size and review volume through lossless sharing; preserve the complete expanded contract and current version; keep manual canonical ownership, fail-closed parity, and CODEOWNER review; make expansion deterministic and testable.

**Non-Goals:** Change live MCP declarations or runtime validation, generate the catalog from implementation code, change public or persisted contracts, change protocol negotiation or error semantics, or introduce migrations.

## Decisions

1. **Single formatted JSON artifact with catalog-local definitions.** Add a top-level `$defs` map of meaningfully named exact schema subtrees. Within `toolDefinitions[*].inputSchema` and `.outputSchema` and within definitions, replace identical subtrees with sole-member `{ "$ref": "#/$defs/<name>" }` objects. Preserve annotations, tool order, prompt/resource/capability lists, and `version`. Prefer repeated subtrees at least 2 KB where references materially reduce size; choose stable semantic names rather than hash or ordinal names. JSON remains formatted with two-space indentation. This keeps ADR 0002 ownership and makes the compact source reviewable. Splitting by tool would retain the duplication; deterministic generation would create a new source/generator relationship and require revisiting ADR 0002.
2. **Strict internal expansion before parity.** A focused TypeScript helper used by the contract test expands only local catalog definitions into fresh JSON values, with memoization and cycle detection. It rejects unknown targets, unsupported paths, malformed or sibling-bearing `$ref` objects, and invalid definition values. It expands every tool input and output schema, rejects unused definitions, and leaves non-schema fields untouched. The resulting value has the legacy catalog shape, and parity compares it with all live schemas and annotations. Because the catalog references are storage syntax, they are never exposed as client JSON Schema.
3. **Independent equivalence evidence.** Before replacing the catalog, record the SHA-256 of the existing parsed catalog serialized with `JSON.stringify`: `2b5c1a5d6c0f74ca80f2c2e813612c7a3d8edcd7bd31ed186c968d3131b40752`. The migration test hashes expanded canonical output with the same algorithm, checks determinism, and exercises missing/cyclic/malformed references and structural changes in shared definitions. Live-registration parity still independently compares all 72 tools and supporting surfaces. A future approved public contract change must deliberately update the pinned digest with its synchronized catalog change.

## Risks / Trade-offs

- **Reference indirection hinders local reading** → Use semantic names, keep definitions in the same file, avoid extracting trivial fragments, and include a short format explanation in contract guidance.
- **A missing or malformed reference could hide schema content** → Fail expansion before parity; require exact legacy digest and mutation tests, including drift through a shared definition.
- **Object ordering can affect a serialization digest** → Preserve original insertion order in expanded objects and use structural comparison for live parity. The pinned migration digest is intentionally exact for this storage rewrite.
- **A large checked-in catalog still has maintenance cost** → Measure final bytes and lines; do not add a generator or weaken the gate to reach a numeric target.

## Migration Plan

After explicit approval, make a one-time lossless catalog rewrite and update the test helper in the same change. No user project or public protocol migration is needed. Rollback is a revert of the catalog and resolver/test edits. Run all required checks, verify conformance, obtain CODEOWNER review, then synchronize and archive. The protected archive-only gate is expected to reject this active change before archival; any other failure blocks archival, and the gate must pass afterward.

## Open Questions

None. Any inability to reproduce the baseline digest or preserve live parity is a blocking mismatch requiring revised approved artifacts, not an implementation choice.
