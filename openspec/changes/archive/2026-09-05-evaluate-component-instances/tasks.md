## 1. Approval and canonical contracts

- [x] 1.1 Obtain explicit approval of proposal.md, design.md and every delta spec before editing implementation; record approval evidence.
- [x] 1.2 Add canonical component evaluation operation, alias, timing, transform/order, limit, failure and schema-13 migration fixtures; update ownership and headless/MCP/capability catalogs before consumers (motion-graphics-contracts: Governed component evaluation activation).

## 2. Core persistence and root editing

- [x] 2.1 Implement schema 12-to-13 migration with source-schema validation, current/history preservation, unknown-future rejection and crash-injection tests covering both project-persistence scenarios.
- [x] 2.2 Add root overlay instance validation and typed create/update operations, common-field defaults, incoming reference/slot validation, lock rules and aliased batch support; cover every Typed atomic root instance editing scenario, including unsupported generic edits and failed-batch byte equality.
- [x] 2.3 Verify generic move/remove/duplicate/transform/visibility/order/parent edits, drafts, undo/redo, reopen and asset integrity/GC for root instances; preserve definition-only behavior and root deletion-reference safety.

## 3. Core scene evaluation

- [x] 3.1 Implement bounded pure traversal and per-occurrence effective slot resolution; cover repeated DAGs, hidden/unused invalid data, every expansion limit at/over boundary, special slot keys, all slot kinds and immutable shared definitions (Instance-local slot resolution and identity; Bounded expansion before side effects).
- [x] 3.2 Introduce precise private derived clocks/spans and compose start/trim/duration/rate through nested content; add independent fractional boundary, source-trim, local animation/fade/caption and transition tests for both Composed half-open local clocks scenarios.
- [x] 3.3 Compose local/group/instance affine transforms and opacity, occurrence visibility and hierarchical ordering; cover differing dimensions, noncentral anchors, skew/rotation, same local IDs, repeated instances, overflow and root-only clipping using independent matrix/order oracles (Hierarchical instance transforms and stacking).

## 4. Core rendering

- [x] 4.1 Route mapped visual/media/text/rich-text instructions through the existing shared render plan and path-safe resource binding/measurement pipeline; test occurrence-specific assets/fonts and existing typed failure/publication behavior.
- [x] 4.2 Render mapped audio with pitch-preserving bounded tempo stages, local automation/fades, mute/role/gain and mapped voiceover activity; cover rate boundaries, visibility distinctions and repeated source intervals (Shared component audiovisual rendering).
- [x] 4.3 Add canonical nested visual/audio regression fixtures and frame/range/draft/export comparisons using existing tolerances; test non-unit fractional rates, trims, transitions, slots, local animation and legacy projects without root instances.

## 5. Headless and bridge consumers

- [x] 5.1 Update headless typed requests, status and canonical parity consumers; keep semantic rules delegated to core (Governed component evaluation activation).
- [x] 5.2 Update bridge typed schemas, standalone MCP and batch registrars, status/schema readers and contract parity tests; test alias resolution and validation paths (Typed root component instance workflows).
- [x] 5.3 Add real source integration and packaged smoke workflows covering create/update/batch, preview/export, invalid input, missing references, locks, stale revisions, rollback, undo/redo and reopen.
- [x] 5.4 Update component/slot/API documentation with the approved timing, coordinate, ordering, visibility/audio, migration, limit and fallback rules; clearly scope historical deferred-rendering statements to older runtimes.

## 6. Validation and lifecycle completion

- [x] 6.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` from repository root, including canonical render golden coverage; document commands, outcomes and scenario-to-test traceability in verification.md.
- [x] 6.2 From apps/agent-bridge run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration` and `bun run test:smoke`.
- [x] 6.3 From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts` for hermetic worker parity; document applicability and any technically impossible automated scenario with justification rather than marking skipped checks passed.
- [x] 6.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; obtain designated CODEOWNER review of canonical contracts and consumers.
- [x] 6.5 Use `$openspec-verify-change`; reconcile all requirement/design/task/test/code mismatches and record verification evidence. Any failed or skipped required check blocks completion.
- [x] 6.6 Use `$openspec-archive-change` to synchronize accepted deltas and archive the verified change; run `moon run root:openspec-validate` on the resulting archive-only tree. The active-change policy gate is expected to block before archival and must not be bypassed.
