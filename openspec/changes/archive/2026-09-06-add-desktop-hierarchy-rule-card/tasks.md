## 1. Approval and scenario mapping

- [x] 1.1 Obtain explicit approval of proposal, design and both delta specs before implementation; record the approval source.
- [x] 1.2 Map every scenario to named automated tests and the native desktop checklist in verification evidence; confirm existing schema/public contracts suffice before editing consumers.

## 2. Core fixture and lifecycle evidence

- [x] 2.1 Add deterministic fixture construction/metadata for six visual children, three slotted instances, one root parent and managed synthetic icons/audio; record independent transforms, ordering, slots and timing expectations (Reusable slotted rule-card lifecycle fixture).
- [x] 2.2 Add passing/failing lifecycle tests through existing core standalone and aliased batch edits, including parent movement, z-index, undo/redo/reopen, missing references, invalid slots, cycles, locks, stale revisions and byte-identical rollback; preserve schema/current-history migration coverage.
- [x] 2.3 Add separate validated rule-card references and production native still/range/export conformance for original, moved, undone, redone and reopened states, with explicit dependency failures and deliberate reference updates (Rule-card preview and export conformance).
- [x] 2.4 Invoke rule-card conformance from the existing required native gate without changing the flat-scene golden or performance-report contract; prove wrong slots/transforms/order and coordinated drift fail.

## 3. Desktop presentation

- [x] 3.1 Add the public core dependency and tested session/controller with paired startup arguments, explicit empty/error states, authoritative snapshots, background I/O and serialized revisioned writes (Explicit core-backed desktop project session).
- [x] 3.2 Implement/test lazy scoped hierarchy projection, instance-path selection, hidden items, cross-track parentage and 4096-row expansion cap; connect hierarchy and root timeline selection (Scoped hierarchy and selection).
- [x] 3.3 Implement/test inspector root parent/detach and i32 z-index inputs using existing typed operations; show local content read-only and core failures unchanged (Revisioned parent and z-index controls).
- [x] 3.4 Implement/test undo, redo, refresh, conflict handling and selection reconciliation without automatic mutation replay (Desktop history and refresh consistency).

## 4. Agent integration and documentation

- [x] 4.1 Add headless/MCP rule-card integration evidence using existing standalone operations and timeline_batch_edit aliases; verify failures preserve atomic state and existing contract fixtures remain compatible.
- [x] 4.2 Document startup, hierarchy/timeline inspection, parent-local semantics, stacking, controls, read-only scope and manual smoke steps; update render fixture documentation and milestone status with verified evidence only.

## 5. Required verification and archival

- [x] 5.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo build -p opencut-desktop` and `cargo test -p opencut-desktop`; record results.
- [x] 5.2 Configure explicit OPENCUT_FFMPEG_PATH, OPENCUT_FFPROBE_PATH, OPENCUT_TEST_FONT_PATH and OPENCUT_GOLDEN_REQUIRED=1 per docs/render-regression-fixtures.md; run `cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact` and retain rule-card and flat-scene conformance evidence.
- [x] 5.3 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke` and `bun run scripts/run-python-tests.ts`; report every failed or skipped required check.
- [x] 5.4 Build and manually run the native desktop against the fixture store using `cargo run -p opencut-desktop -- --project-store <fixture-store> --project-id <fixture-project-id>`; verify load, expansion, distinct selections, parent/z-index changes, conflict/refresh, undo/redo and reopen. Record platform and actual observations; inability to execute this workflow blocks completion.
- [x] 5.5 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and use `$openspec-verify-change`; resolve all requirement/design/task/test/code mismatches and document verification before archival.
- [x] 5.6 Use `$openspec-archive-change` to merge accepted deltas into living specs, then run `moon run root:openspec-validate`. The archive-only policy deliberately blocks active changes; do not bypass it or archive unverified work. Completion requires the Moon gate and all affected checks passing.
