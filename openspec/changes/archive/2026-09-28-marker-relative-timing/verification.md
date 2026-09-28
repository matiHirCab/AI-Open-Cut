## Verification Report: marker-relative-timing

### Summary

| Dimension | Status |
| --- | --- |
| Completeness | 15/15 tasks complete, including sync, archival and final protected validation |
| Correctness | 8/8 requirements and 23/23 scenarios mapped to implementation and automated evidence |
| Coherence | Six design decisions followed; no contradictory dependency or contract ownership edge found |

### Requirement and scenario evidence

| Capability and requirement | Scenarios | Implementation | Automated evidence |
| --- | ---: | --- | --- |
| marker-relative-timing: Scoped marker lifecycle | 2 | `crates/editor-core/src/markers.rs`, `crates/editor-core/src/timeline.rs` | `crates/editor-core/tests/markers.rs` marker creation, scope, bounds, IDs, ambiguity, dangling edits, and rollback |
| marker-relative-timing: Marker-relative item start timing | 3 | `crates/editor-core/src/model.rs`, `crates/editor-core/src/markers.rs`, `crates/editor-core/src/validation.rs` | `crates/editor-core/tests/markers.rs` root/component resolution, signed offsets, missing/ambiguous/out-of-range failures, numeric compatibility |
| marker-relative-timing: Transactional marker operations | 3 | `crates/editor-core/src/timeline.rs`, `crates/editor-core/src/store.rs` | `crates/editor-core/tests/markers.rs`, `apps/headless/tests/protocol.rs`, `apps/agent-bridge/tests/marker-workflow.ts` batch alias, rollback, stale revision, history and reopen |
| motion-graphics-contracts: Governed runtime marker contract | 2 | `contracts/*.json`, Rust and TypeScript model/transport/schema consumers | `bun run contracts:check` including canonical fixture and MCP schema parity tests |
| project-persistence: Atomic schema 24 marker activation | 4 | `crates/editor-core/src/migrations.rs`, `crates/editor-core/src/store.rs` | `crates/editor-core/tests/markers.rs` migration, malformed/future data, retained history and fault recovery; workspace persistence tests |
| timeline-editing: Marker-aware item start edits | 4 | `crates/editor-core/src/timeline.rs`, `crates/editor-core/src/markers.rs` | `crates/editor-core/tests/markers.rs` numeric clearing, compatible trim, split, both duplication paths and overflow; workspace component tests |
| agent-bridge: Typed marker transport and discovery | 3 | `apps/headless/src/main.rs`, `apps/agent-bridge/src/headless-contract.ts`, `apps/agent-bridge/src/schemas.ts`, `apps/agent-bridge/src/server/timeline.ts` | headless protocol, 13 MCP integration and 8 packaged smoke cases, including marker workflow |
| rendering-export: Shared marker-resolved render timing | 2 | `crates/editor-core/src/evaluated_scene.rs`, `crates/editor-core/src/renderer.rs` | `crates/editor-core/tests/markers.rs` preview/draft/range/export pixel comparison and invalid timing before output inspection |

### Design and ownership

The root and component collections, retained expression with synchronized `startMs`, local exact-name lookup, transaction aliases, schema-24 generation migration, and versioned public catalogs follow design decisions 1–6. Scope-local name maps are built once per candidate composition validation. Editor-core owns validation and rendering timing; headless and MCP adapt typed results. `docs/adr/0003-editor-core-module-boundaries.md` and its architecture test include the new marker module edge.

### Checks and evidence limits

- `cargo fmt --all -- --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: pass; full log `C:\Users\matia\AppData\Local\Temp\opencut-clippy-final-f304d9d9-c8c8-42fc-b041-fb4a69342878.log`.
- `cargo test --workspace --no-fail-fast`: pass; full log `C:\Users\matia\AppData\Local\Temp\opencut-workspace-final-b50f0cf4-aace-41c9-8b2c-afb0f37f1617.log`.
- `bun run typecheck`, `bun run lint`, `bun run test`: pass, 419 unit tests and 1 existing skip; full log `C:\Users\matia\AppData\Local\Temp\opencut-ts-validated-bda625ab-a9df-4dc8-b2bd-52493292fdcc.log`.
- `bun run contracts:check`: pass, including 352 TypeScript parity cases; full log `C:\Users\matia\AppData\Local\Temp\opencut-contracts-validated-b0011e32-50a1-4119-b033-d672684cca69.log`.
- `bun run test:integration`: pass, 13 cases; full log `C:\Users\matia\AppData\Local\Temp\opencut-integration-validated-f813a749-8423-4316-b6ee-0a6e5a7ab85c.log`.
- `bun run test:smoke`: pass, 8 packaged cases; full log `C:\Users\matia\AppData\Local\Temp\opencut-smoke-validated-3e0e2f53-df81-4fa6-93da-dd27a3447557.log`.
- Focused actual-pixel render regression: pass with a temporary FFmpeg 9 command compatibility wrapper; the wrapper is outside the repository and changes only the removed `-filter_complex_script` spelling. The full workspace suite runs this test without the wrapper and therefore skips actual FFmpeg execution in the sandbox; the separate focused run provides actual pixel evidence.
- Hermetic Python worker tests: not applicable; no Python worker code or provider contract changed.
- Strict change validation: pass. Prearchive protected gate: exit 1 only because `marker-relative-timing` remains active; 30 specs/changes validate, 0 fail. Full log `C:\Users\matia\AppData\Local\Temp\opencut-prearchive-gate-d36c1cab-6d0f-4b7d-be17-c746db204adb.log`.
- Postarchive strict all-spec validation: pass, 30 specs. Protected `root:openspec-validate` gate: pass; full log `C:\Users\matia\AppData\Local\Temp\opencut-postarchive-gate-6e0f9aba-9eca-43b5-a84d-856a33d91406.log`.
- `git diff --check`: pass; Git reports only line-ending normalization notices.

### Issues and final assessment

No implementation critical, warning or suggestion remains. The delta specs are synchronized, the change is archived, strict all-spec validation passes (30/30), and the postarchive protected Moon gate passes. CODEOWNER review of public contract changes is required for a merge and remains a review-stage action.
