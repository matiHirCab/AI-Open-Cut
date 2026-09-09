## 1. Approval and traceability

- [x] 1.1 Obtain and record explicit user/reviewer approval of proposal, design, delta specs and tasks before implementation; confirm run-based scope and mutually exclusive text/document inputs. User approved in this task on 2026-09-09.
- [x] 1.2 Create verification.md mapping every scenario in all four delta specs to automated tests and record the public contract ownership/review checklist.

## 2. Core model and migration

- [x] 2.1 Add core document/projection validation and schema-18 model support; add tests for ordered Unicode runs, inclusive bounds, malformed/unknown/null values, hidden/unused definitions, finite styles and literal text (rich-text-documents: Canonical bounded text documents).
- [x] 2.2 Implement source-version validation and one-run conversion across current and all retained snapshots using existing locked transactions; preserve source inputs and legacy drafts (project-persistence: Atomic schema 18 text document migration).
- [x] 2.3 Add deterministic migration fixtures/tests for supported and mixed versions, root/component text, existing slots, unchanged fields/assets, future/malformed current/undo/redo, every publication fault, undo/redo and idempotent reopen (all migration scenarios).
- [x] 2.4 Update canonical persisted/request/output/capability and positive/negative fixtures under contracts and the ownership catalog before implementing governed transport consumers; keep unrelated fixture-only concepts inactive (agent-bridge: Additive rich text transport parity).

## 3. Core editing and evaluation

- [x] 3.1 Normalize legacy/document creation and update inputs, reject conflicts/null/non-text targets, synchronize projections, and preserve runs through split/duplicate/component copy/move/trim; add focused success and typed-failure tests (Compatible reversible document edits).
- [x] 3.2 Add alias-batch success/rollback, stale revision, missing reference, locked/incompatible track, undo/redo/reopen and retained draft tests using the core mutation paths (all reversible-edit scenarios).
- [x] 3.3 Evaluate stored runs with effective text/rich-text slot replacement precedence without changing base definitions; keep semantically plain documents on legacy rendering and styled documents on existing run preparation (Shared stored rich text evaluation).
- [x] 3.4 Add fixed-resource regression fixtures for legacy style/Unicode/migration output, independent styled instances, unavailable fonts/path failures and frame/range/draft/export parity using existing tolerances (all rendering scenarios).

## 4. Headless and bridge consumers

- [x] 4.1 Extend the typed headless union and responses with document input/projection output; advertise rich_text_documents and schema 18; add protocol tests for legacy compatibility, document round-trip and failures (Additive rich text transport parity).
- [x] 4.2 Update bridge declarations, strict Zod/MCP schemas, project-state schemas and registrations for standalone/batch/draft document edits while delegating semantic validation to core; add MCP workflow and packaged smoke assertions (all transport scenarios).
- [x] 4.3 Update every governed parity consumer, including explicit rich-text fixture coverage in contracts:check, and obtain @matiHirCab review of canonical artifacts and consumers.
- [x] 4.4 Document request examples, limits, projection/conflict rules, timing/order/default behavior, migration/downgrade policy, capability discovery and existing font fallback/error behavior; synchronize design/specs/fixtures/tasks with implementation.

## 5. Verification and archive

- [x] 5.1 From repository root run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; include all migration, architecture and render-regression tests without skipping dependency failures.
- [x] 5.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint` (includes formatting), `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; record commands, outcomes and scenario evidence.
- [x] 5.3 From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts` for hermetic worker regression evidence; provider contracts remain unchanged.
- [x] 5.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; use `$openspec-verify-change`, resolve every code/spec/design/test mismatch and record results in verification.md. Any missing approval or failed/unavailable required check blocks completion.
- [x] 5.5 Use `$openspec-archive-change` to merge accepted deltas into living specs after implementation verification; then run `moon run root:openspec-validate` and any checks previously blocked by archive-only policy. Do not weaken preflight to allow an active change. Record final outcomes and report completion only when all required checks pass and specifications are synchronized.
