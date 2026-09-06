## 1. Approval and contract evidence

- [x] 1.1 Record explicit approval of proposal, design, and all vector-primitives scenarios, including primitive-only scope and bounds, before implementation.
- [x] 1.2 Add contracts/vector-primitives-v1.json and register canonical ownership/consumers; include every tag, strict shape, boundary, overflow, invalid grammar, and core_primitives_only activation case from the specification.

## 2. Core primitives

- [x] 2.1 Add focused tests for strict vocabulary, immutable round trips, wrong types, unknown fields, unsafe strings, finite values, and INVALID_ARGUMENT; implement and export the core primitive module to satisfy those scenarios.
- [x] 2.2 Add color/gradient acceptance and rejection tests; implement all paint types and pure validation using the explicit color/coordinate/stop semantics.
- [x] 2.3 Add stroke/corner boundary and rejection tests plus independent asymmetric radius-resolution examples; implement strict stroke validation and pure radius resolution.
- [x] 2.4 Add path tests for every command, fill rule, open/closed/multiple/move-only subpaths, malformed transitions, and 4096/4097 boundaries; implement the bounded iterative validator.

## 3. Bridge and documentation

- [x] 3.1 Add reusable strict Zod schemas and shared catalog acceptance/negative evidence matching production Rust validation; extend contracts:check coverage without registering new tools or operations.
- [x] 3.2 Document wire vocabulary, limits, color/geometry semantics, stable failures, and #28 activation boundary; maintain a requirement/scenario-to-test table in verification.md.
- [x] 3.3 Verify unchanged persisted schema, legacy color strings, headless/MCP contracts and capabilities; document migration/missing-reference/new-alias non-applicability and use existing revision, rollback, undo/redo, reopen, and preview/export tests as regression evidence.

## 4. Verification and archival

- [x] 4.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from the root; record results and resolve failures.
- [x] 4.2 From apps/agent-bridge run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke`; record results and resolve failures.
- [x] 4.3 From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts` for hermetic worker regression evidence; report any unavailable required suite explicitly.
- [x] 4.4 Run `bunx @fission-ai/openspec@1.5.0 validate add-vector-paint-primitives --strict --no-interactive`; use openspec-verify-change and resolve all scenario, test, implementation, and design mismatches.
- [x] 4.5 Obtain designated @matiHirCab contract-owner review, archive/synchronize the verified change using openspec-archive-change, then run `moon run openspec-validate` (root:openspec-validate if qualification is required). Record protected-policy and all required check results; failed or skipped checks block completion.
