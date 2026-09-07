## 1. Approval and canonical evidence

- [x] 1.1 Obtain explicit user/reviewer approval of proposal, design and all five delta specs; record the approval before editing implementation or contract fixtures.
- [x] 1.2 Add the canonical procedural-grids-v1 catalog with every descriptor, strict representation/semantic negative, limit and independent geometry example; register consumers and CODEOWNER coverage. Trace: procedural-grids / Strict bounded grid descriptors, Canonical grid geometry and paint coordinates; agent-bridge / Typed discoverable grid workflows.
- [x] 1.3 Add source schema-15 and native schema-16 current/undo/redo fixtures, including illegal pre-16 grids, mixed history, unused definitions and future/invalid snapshots before consumer migration changes. Trace: project-persistence / Atomic schema 16 grid activation, all scenarios.

## 2. Core model and durable migration

- [x] 2.1 Add strict GridItem/descriptor models and core validation with checked pre-expansion counting; add Rust tests consuming all catalog examples, raw duplicate/nesting rejection and exact 4096/overflow boundaries. Trace: procedural-grids / Strict bounded grid descriptors and Bounded procedural expansion and evaluated behavior.
- [x] 2.2 Implement source-version grid gating and schema 16 migration in core, preserving previous migration constraints; test mixed current/history, byte-identical rejection, repeated reopen and every existing recovery fault phase. Trace: project-persistence / Atomic schema 16 grid activation.
- [x] 2.3 Audit exhaustive item consumers in core, asset collection, definitions, drafts and desktop; add media-free and invalid hidden/unused-content coverage, preserving owning layers and generic desktop presentation. Trace: procedural-grids / Bounded procedural expansion and evaluated behavior; timeline-editing / Transactional procedural grid editing.

## 3. Core editing

- [x] 3.1 Implement add_grid and complete update_item.grid replacement with defaults, locks, parent/track checks and supported generic visual operations; test each success and invalid combination, including null/omitted fields and non-grid targets. Trace: timeline-editing / Transactional procedural grid editing, both scenarios.
- [x] 3.2 Integrate batch creation aliases and later references; test missing references, unresolved/forward/duplicate aliases, stale revisions, locked tracks and failing trailing edits with full rollback. Trace: timeline-editing / Atomic alias-aware grid transactions, both scenarios.
- [x] 3.3 Test timing, transforms/keyframe exclusivity, move/trim/split/duplicate/removal, visibility/order/parenting and local component definitions through undo/redo/reopen and draft materialization. Trace: timeline-editing / Transactional procedural grid editing; procedural-grids / Evaluate composed occurrences deterministically.

## 4. Core evaluation and rendering

- [x] 4.1 Implement core-owned bounded lattice expansion, canonical ordering/endpoints, local clipping and paint space; add independent formula and expected-pixel tests for all patterns, fractional dimensions, dash phase, alpha crossings and viewport edges. Trace: procedural-grids / Canonical grid geometry and paint coordinates, both scenarios.
- [x] 4.2 Route evaluated grid facts through existing vector raster/compositing owners with scale-aware density and aggregate segment/memory accounting; test transformed, hidden, offscreen and repeated-instance limit failures before side effects. Trace: procedural-grids / Bounded procedural expansion and evaluated behavior, all scenarios.
- [x] 4.3 Add renderer fixtures and independent assertions across root/nested/retimed/animated grids and legacy overlap; compare frame/range/draft/export semantic plans and SSIM/PCM/timing at varied requested output sizes. Trace: rendering-export / Shared complete procedural grid rendering, all scenarios.

## 5. Explicit cross-layer contract activation

- [x] 5.1 Update canonical operation, MCP structural schema/annotation, capability and ownership catalogs for add_grid, update_item.grid, item responses, grid_items and grid_rendering before synchronizing consumers. Trace: agent-bridge / Typed discoverable grid workflows.
- [x] 5.2 Update typed headless declarations and protocol tests for standalone/batch/draft/component/project surfaces; forward semantic decisions to core and verify stable errors/revisions. Trace: agent-bridge / Typed discoverable grid workflows, transport scenarios.
- [x] 5.3 Add mirrored strict TypeScript grid schemas, unions and MCP timeline_add_grid registration; update parity runners and test readiness combinations, exact identifiers and shared fixture acceptance. Trace: agent-bridge / Typed discoverable grid workflows, all scenarios.
- [x] 5.4 Add source and packaged MCP workflows for every pattern, aliased edits, rollback, undo/redo/reopen and rendering; document grid fields/formulas/limits/paint behavior, aliases, schema-16 compatibility and failure semantics. Trace: agent-bridge / Typed discoverable grid workflows; timeline-editing / Atomic alias-aware grid transactions.
- [x] 5.5 Obtain designated @matiHirCab review of the resulting canonical contracts and governed consumers, distinct from the recorded pre-implementation proposal approval. Review targets and automated evidence are in verification.md. Trace: agent-bridge / Typed discoverable grid workflows.

## 6. Conformance and closure

- [x] 6.1 Run `bunx @fission-ai/openspec@1.5.0 validate add-procedural-grids --strict --no-interactive` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` from repository root; keep scenario-to-test evidence and artifacts synchronized.
- [x] 6.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` from repository root, including architecture and migration regression suites; record failures explicitly.
- [x] 6.3 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration` and `bun run test:smoke`; ensure the parity runner includes the new Rust/TypeScript grid suites.
- [x] 6.4 Run `bun run apps/agent-bridge/scripts/run-python-tests.ts` from repository root for hermetic worker regression evidence. Run the repository's renderer golden/parity gate with the new grid fixture; record exact resolved commands, required backend availability and results in verification evidence, and do not treat unavailable or skipped required checks as passing.
- [x] 6.5 Use openspec-verify-change and resolve every requirement/design/task/test/code mismatch. Every normative scenario requires automated coverage; record justification only where automation is technically impossible.
- [x] 6.6 Use openspec-archive-change to synchronize accepted deltas into living specs and archive the verified change, then run `moon run root:openspec-validate` and `git diff --check` from repository root. Final Moon policy validation must run with the required archive-only change inventory; a failed required check blocks completion.
