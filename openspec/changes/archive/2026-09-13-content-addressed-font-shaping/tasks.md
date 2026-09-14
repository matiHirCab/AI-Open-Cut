## 1. Approval and contract foundation

- [x] 1.1 Obtain explicit approval of proposal, design and delta specs, including one-time layout changes and required four-style families; record approval before implementation.
- [x] 1.2 Add canonical text-layout v2, schema-19, capability and invalid-input fixtures; register their single owner and every governed consumer under ADR 0002. Retain legacy migration evidence. Trace to all four delta specifications.
- [x] 1.3 Core architecture: update ADR 0003 and architecture-test import matrix for the exact font-owner edges in design decision 2. Verify dependency releases and licenses, pin shaping/Unicode versions, and document the profile. Trace to Canonical versioned shaping.
- [x] 1.4 Packaging: add the reviewed licensed four-style default family, byte hashes and licenses; fixtures include custom fonts and missing styles. Trace to Durable bounded font bindings.

## 2. Core persistence and ownership

- [x] 2.1 Add schema-19 source-validation and migration fixtures before production consumers: mixed current/undo/redo, hidden/unused components, slots, drafts, invalid historical fields, unavailable fonts and future versions. Trace to every Atomic schema 19 font activation scenario.
- [x] 2.2 Core model/validation: implement closed font catalogs, bindings, profile fields and all format/size/discovery bounds with inclusive-limit and malformed-input tests. Trace to Durable bounded font bindings.
- [x] 2.3 Core assets: implement safe deterministic resolution, exact-byte hashing/deduplication and managed ingestion through persistence I/O, with symlink/traversal/network and source-change tests. Trace to Durable bounded font bindings and Centralized managed font ownership.
- [x] 2.4 Core assets: extend centralized integrity/reference/GC traversal for current, component, slot, history and draft owners; test missing/corrupt content, hidden references and cleanup warnings. Trace to every Centralized managed font ownership scenario.
- [x] 2.5 Core store/migrations/persistence: migrate and publish current/history/draft bindings plus staged fonts as one recoverable generation. Test every existing and added injection phase and byte-identical failed sources. Trace to Atomic schema 19 font activation.

## 3. Core editing and evaluated shaping

- [x] 3.1 Core timeline/store: resolve selectors within atomic standalone/batch edits, preserve bindings on content/style-only edits and lifecycle copies, and enforce conflict-before-side-effect ordering. Test aliases, invalid later operations, missing IDs, undo/redo and source removal. Trace to Reversible stable font editing.
- [x] 3.2 Core drafts: persist/materialize/rebase/commit retained bindings without ambient re-resolution or authoritative preview mutation. Cover legacy migration and font retention through draft-only owners. Trace to Reversible stable font editing and Atomic schema 19 font activation.
- [x] 3.3 Core fonts: implement versioned bidi/script shaping and cluster-safe wrapping with exact independent glyph/cluster/position fixtures and glyph/line overflow tests. Trace to every Canonical versioned shaping scenario.
- [x] 3.4 Core evaluated_scene: carry logical managed face keys, prepare verified byte inputs and finalize shared shaped runs/bounds after effective slot resolution and component expansion; preserve timing/ancestry/animation/Transform2D. Trace to Shared pinned glyph rendering.
- [x] 3.5 Core render_artifact/render_plan: rasterize canonical glyph outlines and remove schema-19 string reshaping/ambient resolution. Add native frame/range/draft/export parity, independent layout/animation checks and no-output failure tests. Trace to every Shared pinned glyph rendering scenario.

## 4. Governed consumers and documentation

- [x] 4.1 Cross-layer contract update: synchronize headless DTOs, project/version/capability responses, bridge TypeScript/Zod/MCP fixtures and layout negotiation; pass immutable font configuration into core. Add standalone/batch/draft/reopen integration coverage and include new parity suites in contracts:check. Trace to Reversible stable font editing and Advertise the compatibility transition.
- [x] 4.2 Documentation: publish selectors/fallback/order/coordinates/timing, font limits, layout profile, default font licenses, schema migration/backup guidance and missing-content recovery behavior; obtain designated CODEOWNER review for canonical artifacts and consumers. Trace to every normative requirement.
- [x] 4.3 Packaging: verify the compiled bridge/headless distribution includes default faces and works with ambient fonts removed, while preserving caption/media/audio behavior. Trace to Pin exact selected bytes and Compare every render intent after reopen.

## 5. Validation and completion

- [x] 5.1 Maintain scenario-to-test traceability in verification.md for every normative requirement; no untested scenarios without documented technical impossibility.
- [x] 5.2 Root Rust checks: `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. Run native render fixtures with the repository's required FFmpeg/FFprobe/font configuration; report unavailable or skipped fixtures as blockers.
- [x] 5.3 From apps/agent-bridge: `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, `bun run test:smoke`. Resolve all failures; packaged smoke must exercise default font distribution.
- [x] 5.4 From apps/kokoro-tts: `bun run ../agent-bridge/scripts/run-python-tests.ts` for hermetic worker regression evidence; no provider contract or inference behavior changes are intended.
- [x] 5.5 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `moon run root:openspec-validate`. Record any archive-only policy conflict while active; do not bypass the gate.
- [ ] 5.6 Use `$openspec-verify-change`, resolve specification/design/task/code/test mismatches, record review and validation evidence, then use `$openspec-archive-change` to merge accepted deltas into living specs. Re-run `moon run root:openspec-validate` after archival and keep completion blocked on any failed or skipped required check.

Task 5.6 status (2026-09-13): verification, approved contract review, spec sync,
archival and the Moon rerun are done. Moon exits 1 solely for the separate active
`reduce-agent-context-overhead` change; all 26 spec items pass. Keep this task
open until that repository-wide gate passes. See verification.md for the log.
