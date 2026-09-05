## 1. Approval and canonical contracts

- [x] 1.1 Obtain explicit approval of proposal, design and all delta scenarios; record approval before using openspec-apply-change or editing implementation.
- [x] 1.2 For Governed component definition runtime contract, author component-definitions-v1 fixtures and schema-11 current/mixed-history migration fixtures before consumers; update ownership, CODEOWNERS and headless/MCP catalogs together.

## 2. Core persisted model and validation

- [x] 2.1 Implement Stored local component timelines in model with closed definition and nested-instance types; test required/null/unknown fields, scope-local identity, root placement rejection and existing root decoding compatibility.
- [x] 2.2 Implement Bounded nested component graph and duration in canonical validation; test duplicate IDs before indexing, missing references, branching longest paths, hidden/unreachable cycles, shared/repeated references, depth 16/17 and every aggregate limit boundary.
- [x] 2.3 Test and implement exact local/source duration boundaries, zero/negative/fractional/unsafe/nonfinite inputs, overflow and referenced-definition shortening; preserve existing ordinary item rules.
- [x] 2.4 Implement Atomic schema-11 component migration using preauthored fixtures: current and mixed undo/redo migration, idempotent reopen, schema-zero/future/malformed history rejection, and injected publication recovery with unchanged authoritative bytes on failure.

## 3. Core workflows and integration

- [x] 3.1 Implement Atomic definition management in timeline/store facades: create/full update/delete, target and nested-reference aliases, changed IDs, referenced deletion, locked local tracks, revision precedence, per-operation graph validation and late-batch rollback tests.
- [x] 3.2 Implement Component managed media retention in assets/draft integration; audit all root-only traversal sites, preserve media/provenance through definitions, durable drafts and retained history, and test deletion/collection/path confinement.
- [x] 3.3 Verify Preserve root rendering through direct-input validation plus frame/range/draft/export equivalence; run existing group/Transform2D/stacking/golden regressions without changing renderer output or activating root instances.
- [x] 3.4 Check ADR 0003 ownership/architecture tests; if a new private import edge is needed, document the concrete edge and obtain approval for its ADR/test change before implementation.

## 4. Typed consumers and documentation

- [x] 4.1 Add headless typed definition operations, schema/capability reporting and protocol tests for success, aliases, missing/invalid references, locks, stale revisions, byte-identical failure rollback and exact history/reopen.
- [x] 4.2 Add bridge request/response types, closed Zod schemas and thin standalone MCP tools plus batch variants; consume canonical fixtures in parity tests and preserve existing simple operations.
- [x] 4.3 Add a shared real MCP definition lifecycle and failure workflow to integration and packaged smoke; cover aliases, nested graph errors and rollback through actual core calls.
- [x] 4.4 Document definition payloads, local coordinates/time/order, identity, limits, explicit duration, locks/errors, schema-11 reader/migration behavior and the deferred #23/#24 behavior.

## 5. Verification and completion

- [x] 5.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace -- --test-threads=1` using retained FFmpeg/FFprobe 8.1.2, the checked-in DejaVuSans font and OPENCUT_GOLDEN_REQUIRED=1 for native checks.
- [x] 5.2 In apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`. Provider/Python worker tests are unaffected unless implementation changes their contracts or behavior; reassess and run affected hermetic tests if that occurs.
- [x] 5.3 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `bunx --package @moonrepo/cli@2.3.3 moon run root:openspec-validate`; report the active inventory gate explicitly until archive.
- [x] 5.4 Use openspec-verify-change and write verification.md mapping every requirement/scenario to named automated evidence; resolve mismatches and obtain designated CODEOWNER review of the final canonical contracts, consumers and parity evidence.
- [x] 5.5 Use openspec-archive-change to synchronize accepted living specs and archive; rerun pinned Moon with archive-only inventory before reporting completion.

## 6. Approved review correction

- [x] 6.1 Implement and test complete nested timestamp and media-only volume validation without changing root policy.
- [x] 6.2 Implement and test caption provenance/style validation, including moved source words and optional defaults.
- [x] 6.3 Expand canonical native/MCP acceptance fixtures and atomic lifecycle, draft, persisted-history and direct-render regressions.
