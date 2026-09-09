## 1. Approval and canonical evidence

- [x] 1.1 Obtain explicit user/reviewer approval of the proposal, design, and all five delta specs; record the approval before editing implementation or contract fixtures. Trace: all requirements.
- [x] 1.2 Add `contracts/repeaters-v1.json` with exact descriptor representation, identifiers, limits, supported/unsupported sources, matrix/opacity/timing/ordering examples, and structural/semantic negatives; register ownership and CODEOWNER coverage. Trace: repeaters / Strict bounded repeater descriptors, Deterministic additional-copy semantics; agent-bridge / Typed discoverable repeater workflows.
- [x] 1.3 Add native schema-17 and source schema-16 fixtures covering current/undo/redo, illegal pre-17 repeater records, hidden/unused definitions, future versions, and interrupted generations before consumer migration changes. Trace: project-persistence / Atomic schema 17 repeater activation.

## 2. Core model, validation, and durable migration

- [x] 2.1 Add strict `RepeaterItem`, descriptor, source-reference, and transform-offset models plus duplicate-preserving raw decoding tests consuming the canonical catalog. Trace: repeaters / Strict bounded repeater descriptors.
- [x] 2.2 Implement source kind/scope/existence, visual-only closure, deletion protection, and combined parent/component/repeater graph validation for root, component, hidden, history, and draft content; test missing, unsupported, cross-scope, duplicate-ID, self/indirect cycle, and ungroup/delete failures. Trace: repeaters / Scoped acyclic repeater references; rendering-export / Shared complete repeater rendering.
- [x] 2.3 Implement schema-17 source gating and atomic current/undo/redo migration without altering earlier migration steps; test mixed history, byte-identical rejection, repeated reopen, future versions, and every recovery fault phase. Trace: project-persistence / Atomic schema 17 repeater activation.
- [x] 2.4 Audit exhaustive item consumers in core, asset/provenance collection, drafts, components, renderer fixtures, and desktop; preserve repeater media-free behavior and generic non-authoring presentation. Trace: repeaters / Lazy bounded occurrence expansion; timeline-editing / Transactional repeater editing.

## 3. Core editing and transactions

- [x] 3.1 Implement `add_repeater` and complete `update_item.repeater` replacement with overlay/lock/timing/default/reference checks; test every valid descriptor boundary and malformed/semantic failure. Trace: timeline-editing / Transactional repeater editing; repeaters / Strict bounded repeater descriptors.
- [x] 3.2 Integrate supported move/trim/split/duplicate/delete/visibility/z-index/reorder operations and reject parenting, transforms, keyframes, audio, and transitions; test root and component definitions through undo/redo/reopen. Trace: timeline-editing / Transactional repeater editing.
- [x] 3.3 Integrate standalone and ordered batch creation aliases in track/source/later item references; test stale revisions, locked tracks, missing sources, unresolved/forward/duplicate aliases, cyclic graphs, exceeded limits, and failed trailing edits with full rollback. Trace: timeline-editing / Alias-aware repeater transactions.

## 4. Lazy evaluation and shared rendering

- [x] 4.1 Implement checked preflight for per-repeater copies, combined expansion closures, exact 65,536 occurrence accounting, derived matrices/opacities/intervals, and existing layer/resource/vector/surface/memory budgets before allocation. Test all inclusive and one-over boundaries, hidden/unused content, and non-finite derived powers. Trace: repeaters / Lazy bounded occurrence expansion.
- [x] 4.2 Implement deterministic shape, group-subtree, and component-instance copy traversal with source-preserving evaluation, interval intersection, stable occurrence IDs, source-parent `O^i`, clamped opacity, generated-block ordering, and no time shift. Add independent matrix, opacity, clock, hierarchy, identity, and ordering oracles. Trace: repeaters / Deterministic additional-copy semantics.
- [x] 4.3 Route generated ordinary scene facts through existing render planning/artifact owners without persisted repeater inspection; verify no downstream semantics are reconstructed and unsupported readiness fails closed. Trace: rendering-export / Shared complete repeater rendering.
- [x] 4.4 Add renderer golden/parity fixtures for root and nested shape/group/component repeaters, transforms, interval edges, opacity boundaries, equal-z ordering, drafts, and legacy overlap; compare frame/range/draft/export semantic plans and output at SSIM/PCM/timing thresholds. Trace: rendering-export / Shared complete repeater rendering.

## 5. Explicit cross-layer contract activation

- [x] 5.1 Update canonical headless operation, MCP structural schema/annotation, capability, project-item, and ownership catalogs for `add_repeater`, `update_item.repeater`, `timeline_add_repeater`, `repeater_items`, and `repeater_rendering` before synchronizing consumers. Trace: agent-bridge / Typed discoverable repeater workflows.
- [x] 5.2 Update typed headless request/response unions and protocol tests for standalone, batch, draft, component, project, and error surfaces; keep semantic decisions in editor-core. Trace: agent-bridge / Typed discoverable repeater workflows.
- [x] 5.3 Add mirrored strict TypeScript schemas and MCP registration, update parity runners/capability readiness, and test exact fixture acceptance plus stable error/retryability translation. Trace: agent-bridge / Typed discoverable repeater workflows.
- [x] 5.4 Add source and packaged MCP workflows for aliased creation/editing, rollback, undo/redo/reopen, component definitions, and rendering; document descriptor fields, additional-copy semantics, matrix origin/order, timing, opacity, limits, source restrictions, aliases, schema-17 compatibility, and failure behavior. Trace: agent-bridge / Typed discoverable repeater workflows; timeline-editing / Alias-aware repeater transactions.
- [x] 5.5 Obtain designated @matiHirCab review of the resulting canonical contracts and governed consumers, distinct from pre-implementation artifact approval, and record the review/evidence before closure. Trace: agent-bridge / Typed discoverable repeater workflows.

## 6. Conformance and closure

- [x] 6.1 Run `bunx @fission-ai/openspec@1.5.0 validate add-lazy-repeaters --strict --no-interactive` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; keep scenario-to-test traceability and artifacts synchronized.
- [x] 6.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from the repository root, including architecture, persistence, recovery, evaluation, and renderer regressions.
- [x] 6.3 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; ensure parity runners include new Rust/TypeScript repeater suites.
- [x] 6.4 Run `bun run apps/agent-bridge/scripts/run-python-tests.ts` from repository root and the repository renderer golden/parity gate with repeater fixtures; record exact resolved commands, backend availability, and results, treating unavailable required checks as blocking.
- [x] 6.5 Use `$openspec-verify-change` and resolve every requirement/design/task/test/code mismatch; every normative scenario needs automated coverage or a documented technical impossibility.
- [x] 6.6 Use `$openspec-archive-change` to merge accepted deltas and archive the verified change, then run archive-only `moon run root:openspec-validate` and `git diff --check`; any failed required check blocks completion.

## Verification evidence

- 2026-09-07: @matiHirCab approved the proposal, design, tasks, and five delta specs before implementation. At verification time, the separate post-implementation contract review in task 5.5 remained intentionally pending and was completed afterward as recorded below.
- 2026-09-07: @matiHirCab separately reviewed and approved the resulting `repeaters-v1`, headless protocol, MCP surface, contract ownership, and governed Rust/TypeScript consumer changes after implementation and verification.
- OpenSpec verification covered 9 normative requirements and 25 scenarios. All requirements and scenarios have automated coverage in the canonical fixture parity suites, editor-core repeater/migration/evaluation/renderer tests, headless protocol tests, and agent-bridge schema/source/packaged workflows; no requirement, design, task, test, or code mismatch remains.
- OpenSpec gates: `bunx @fission-ai/openspec@1.5.0 validate add-lazy-repeaters --strict --no-interactive` passed; `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` passed 23/23 items.
- Rust gates: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace -- --test-threads=1` passed. The final workspace run passed 253 editor-core unit tests plus all architecture, compatibility, feature, headless, and protocol suites, including 6 repeater integration tests and 24 headless protocol tests.
- Agent-bridge gates: `bun run lint`, `bun run typecheck`, and `bun run test` passed; `bun run contracts:check` passed 323 focused contract tests; `bun run test:integration` passed 10/10 workflows; `bun run test:smoke` passed 5/5 packaged workflows.
- Python provider gates: `bun run apps/agent-bridge/scripts/run-python-tests.ts` passed 10 unittest and 5 pytest cases.
- Native renderer gate: `cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact` passed with `OPENCUT_GOLDEN_REQUIRED=1`, DejaVu Sans from the repository fixture, and pinned FFmpeg 7.1.1 full shared build. The downloaded archive SHA-256 was `9F28727E8B472A04C1D2E520AAA425DCA82721B995139B35710091130EA6E699`; the focused repeater native conformance test also passed three consecutive runs.
- Closure: the five accepted delta specs were synced into the living specifications and the change was archived as `2026-09-07-add-lazy-repeaters`. Because `moon` was not installed on the local PATH, the repository-pinned Moon 2.3.3 CLI was resolved as `bunx @moonrepo/cli@2.3.3 run root:openspec-validate`; it passed all 231 policy tests and all 23 living specs. `git diff --check` also passed.
