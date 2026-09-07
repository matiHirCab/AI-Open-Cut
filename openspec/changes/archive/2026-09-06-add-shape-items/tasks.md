## 1. Approval and canonical contracts

- [x] 1.1 Obtain explicit user/reviewer approval of proposal, design and all delta scenarios; record approval before executable edits.
- [x] 1.2 Add canonical shape fixture/ownership entries and update operation, capability, project and MCP catalogs for Typed discoverable shape workflows and Closed bounded shape geometry; cover all variants, bounds, malformed representations and error acceptance stages.

## 2. Core models and migration

- [x] 2.1 Implement Closed bounded shape geometry in the core owning layer with pure structural/semantic tests for every valid/invalid scenario, including paint/stroke and anchor semantics.
- [x] 2.2 Implement Atomic schema 14 shape activation with tests for every supported source version, mixed current/undo/redo, old-schema forbidden shapes, hidden/unused definitions, future/invalid history, byte preservation and all existing publication fault points.
- [x] 2.3 Implement Transactional shape editing across creation, update, aliases, lifecycle, scoped components and drafts; add scenario tests for locks, missing references, conflicts, rollback, undo/redo and reopen.

## 3. Evaluation and rendering

- [x] 3.1 Implement Canonical shape evaluation and bounded raster work with independent analytic geometry tests, nested occurrence transforms/clocks, hidden/unused validation and inclusive/overflow complexity tests.
- [x] 3.2 Implement shared raster/compositing support preserving all vector semantics; test fill/stroke order, gradients, fill rules, dashes/caps/joins, clipping and preflight failures before output side effects.
- [x] 3.3 Add native golden evidence for Shared complete shape rendering through frame/range/draft/export, independently assert expected geometry/color, and verify exact plan equality plus SSIM/audio/timing tolerance and unchanged legacy fixtures.

## 4. Typed transports and documentation

- [x] 4.1 Synchronize headless request/status/response handling and tests for Typed discoverable shape workflows while keeping handlers thin.
- [x] 4.2 Synchronize TypeScript types, strict Zod shape/project/batch/component/draft schemas and MCP registration; extend canonical parity evidence for every affected consumer and partial renderer readiness.
- [x] 4.3 Add real MCP source integration and packaged shape workflows covering all seven variants, aliases, lifecycle, success and typed atomic failures.
- [x] 4.4 Update vector activation and shape/client documentation with exact geometry, coordinates, timing, ordering, limits, migration, errors, capability reporting and examples; preserve legacy operation meanings.

## 5. Conformance and closure

- [x] 5.1 Run root `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; explicitly run the required native render golden suite with its documented dependencies and record results rather than accepting skipped render tests.
- [x] 5.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; include new shape fixtures in the standalone parity command if necessary.
- [x] 5.3 From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts` for hermetic provider regression evidence; provider contracts remain unchanged.
- [x] 5.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; use openspec-verify-change and record scenario-to-test evidence and all required check results in verification.md. Resolve every mismatch; failed/skipped required checks block completion.
- [x] 5.5 Obtain designated contract-owner review, use openspec-archive-change to merge accepted deltas, and run `moon run root:openspec-validate` after archival. The protected policy rejects active change directories; do not weaken it to pass during planning.
