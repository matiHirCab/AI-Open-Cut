## 1. Approval and canonical evidence

- [x] 1.1 Obtain explicit approval of proposal, all five delta specs, design and this task list; record the approval before implementation. Recheck current schema and active changes before starting.
- [x] 1.2 Add canonical runtime template-slots-v1 fixtures and ownership/CODEOWNER mappings before consumer changes; include every kind, binding property, exact bound, default/override case and structural versus semantic rejection. Update component/headless/MCP/capability catalogs and document preparatory catalog adoption (motion-graphics-contracts delta).

## 2. Core persistence and value model

- [x] 2.1 Add closed core slot/value/constraint/binding types and required persisted fields; implement schema-12 migration before transport consumers (Closed typed slot definitions and values; Atomic schema-12 slot migration).
- [x] 2.2 Add automated oldest-schema, schema-11 nested-component, mixed current/undo/redo migration, malformed/future snapshot, reopen and persistence fault-injection coverage; assert exact preserved values and atomic publication (project-persistence delta).
- [x] 2.3 Implement bounded core validation for all eight kinds, Unicode scalars, defaults, enum uniqueness, safe managed references and aggregate limits; test every valid boundary and rejection (Bounded slot content and constraints).

## 3. Core bindings and mutations

- [x] 3.1 Resolve stable local targets, reject incompatible/duplicate writers, validate complete derived effective candidates and incoming instance values; test ID collisions across scopes, order independence, required/optional/default precedence, duration/media interactions and unchanged stored tracks (Stable local binding targets; Closed typed slot definitions and values).
- [x] 3.2 Add component_define_slots to standalone and batch core edits; add compatible optional create/update fields and nested values. Test aliases, locked targets, stale revisions, removed references, one undo step and byte-identical failed batch rollback (Atomic slot definition replacement; Compatible slot-aware component editing).
- [x] 3.3 Include defaults and overrides in core asset integrity/deletion/GC traversal for current/history/drafts; test assets retained only by each owner and overridden defaults (Retain assets referenced by slot values).
- [x] 3.4 Exercise the same core rules through durable drafts, current/history load and direct rendering; prove preview/range/draft/export evaluated output remains equivalent and invalid unused content fails before artifact preparation (Stored slots preserve current rendered output).

## 4. Public consumers and documentation

- [x] 4.1 Extend thin headless typed requests/responses and protocol tests, maintaining protocol 1 and advertising typed_template_slots (Typed template slot workflows).
- [x] 4.2 Extend bridge headless declarations, closed Zod schemas, MCP standalone/batch registration and project response schemas; consume canonical evidence for all types, limits, old-request compatibility and exact failure stages (agent-bridge and motion-graphics-contracts deltas).
- [x] 4.3 Add real source integration and packaged smoke workflows for all eight kinds, standalone and aliased batches, missing references, invalid inputs, revision conflicts, locks, rollback, undo/redo and reopen (Typed template slot workflows).
- [x] 4.4 Update component/agent/contract documentation with exact fields, mapping, limits, local timing/order, migration and deferred renderer scope; obtain @matiHirCab contract review (Canonical runtime template slot evidence).

## 5. Verification and archival

- [x] 5.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` from repository root; resolve every affected failure.
- [x] 5.2 From apps/agent-bridge run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration` and `bun run test:smoke`; record all results. Run `bun run scripts/run-python-tests.ts` from apps/agent-bridge for hermetic provider regression evidence; no provider behavior is changed.
- [x] 5.3 Run `bunx @fission-ai/openspec@1.5.0 validate add-typed-template-slots --strict --no-interactive`. Use `$openspec-verify-change`, record each normative requirement/scenario with named tests and evidence in verification.md, and resolve all discrepancies. Any required skipped/failed check blocks completion.
- [x] 5.4 Use `$openspec-archive-change` to synchronize accepted deltas and archive the verified change; run `moon run root:openspec-validate` after archival, since the protected gate requires archive-only change inventory. Keep the change incomplete if the full gate fails, and record its result without weakening policy.
