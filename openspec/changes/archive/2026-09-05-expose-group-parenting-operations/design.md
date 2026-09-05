## Context

Issue #21 builds on archived stacking and group-DAG changes. Schema 10, `add_group`, `item_set_parent`, `item_set_z_index`, their MCP tools and creation aliases already exist. The missing semantic operation is ungroup. Existing local-preserving parenting provides the compatibility baseline; AGENTS.md requires approval before implementing this new behavior.

## Goals / Non-Goals

Goals: complete the typed group workflow with one atomic, bounded core operation, discovery, canonical fixtures and lifecycle evidence through both transports.

Non-goals: world-transform baking, recursive flattening/deletion, renaming existing APIs, new schema versions, renderer algorithms or provider behavior.

## Decisions

1. Add `EditOperation::GroupUngroup { group_id }`, serialized as `group_ungroup` and `groupId`. Reuse the existing headless edit wrapper, MCP project/revision envelope and mutation result. A bridge-built detach/delete batch was considered, but cannot reliably discover all children against the same revision and would duplicate domain semantics. The operation belongs in core timeline dispatch; no new ADR 0003 dependency edge is needed.
2. Remove only the selected group. Immediate children inherit its parent reference or null. Nested child groups survive with their descendants unchanged. Preserve child local transforms, timing, visibility, zIndex, track and relative array order; normalize stackOrder after removal. Keep-world baking was rejected because composed skew/animation/opacity/timing cannot generally be represented losslessly by current editable properties. Flattening to root would also discard unrelated ancestor relationships.
3. Resolve `groupId` using existing earlier-creation aliases. Ungroup creates no identifier and does not accept resultAlias. Existing successful creation alias mappings remain present even if that group is later removed in the same committed batch, consistent with creation-result reporting. A later reference to the removed alias resolves to a missing item and fails the entire batch. Changing alias lifetime rules would unnecessarily alter existing batch semantics.
4. Before changing the candidate, identify the group and all immediate children and require their tracks unlocked. A locked ancestor or a locked deeper descendant that is only read is permitted. Missing group returns ITEM_NOT_FOUND; non-group target returns INVALID_ARGUMENT; affected locked track returns TRACK_LOCKED. Normal revision checks retain precedence and retryable REVISION_CONFLICT. Use the existing project candidate/commit boundary and validate the final graph. Per-child commits were rejected because they violate one-revision rollback.
5. Report the removed group ID first, then immediate child IDs in pre-edit track/item traversal order, plus any additional ordinal-changed IDs from the existing ordering machinery, deduplicated deterministically. Empty groups still delete in one successful mutation. No new result schema or stable errors are necessary.
6. Extend group-parent-v1, headless-protocol-v1 and mcp-surface-v1 plus ownership/consumer evidence before adapter changes. Add a `group_ungroup` status capability while retaining group_parenting and stacking. Reusing group_parenting alone would not distinguish binaries predating ungroup. Preserve existing protocol major and operation names; roadmap group_create/group_set_parent names are not new aliases.

## Risks / Trade-offs

- Local preservation can visibly move or reveal children when an ancestor contribution is removed. Document this explicitly and test against an equivalent explicit reparent/delete sequence through the unchanged evaluated scene and preview/export paths.
- Cross-track children can be missed by a track-local implementation. Test multiple tracks, hidden/inactive children, nested groups and affected/unaffected locks.
- Contract drift can hide behind local unit tests. Canonical fixtures must be consumed by Rust and TypeScript parity tests, real MCP registration and headless protocol evidence; request designated CODEOWNER review after verification.
- Complex input remains bounded by core's 4096-node, 32-edge and 100-operation limits. No expression, SVG, URL, path or provider inputs are added; existing path confinement, media/provenance ownership and privacy-safe errors remain intact.

## Migration Plan

No persisted shape changes: schema stays 10 and existing current/history migration remains authoritative. Run existing migration and recovery regressions, plus reopen/undo/redo on ungrouped state. Rollback an edit through undo; deploying an older schema-10 binary preserves readable project state but cannot execute the new operation. Unknown future schemas remain rejected. Do not add a migration unless a separately approved scope change requires one.

## Verification Plan

Map all delta scenarios to named core, protocol, contract and MCP tests. Cover root/nested/empty groups, all immediate-child kinds, exact properties and changed IDs, aliases, failure rollback, locks, stale revisions, boundaries, reopen and retained history. Compare the resulting evaluated scene with an independently constructed explicit reparent/delete equivalent; exercise preview/export and existing golden regressions without updating baselines solely to hide changes. Run the exact commands in tasks.md, verify via openspec-verify-change, then archive after approval and successful checks.

## Open Questions

The user approved the local-preserving promotion semantics and all artifacts on 2026-09-05. No unresolved design questions remain.

## Approved null-alias correction

Batch deserialization must retain alias field presence with a private helper using Option<Option<String>> and deserialize_double_option. Reject any present resultAlias for GroupUngroup before collapsing the helper to the unchanged public type. Other operations retain existing omitted/null/string and duplicate-field behavior. Headless malformed input returns non-retryable INVALID_ARGUMENT before transaction execution.
