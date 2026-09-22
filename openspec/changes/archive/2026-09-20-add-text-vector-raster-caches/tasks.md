## 1. Approval and conformance map

- [x] 1.1 Obtain explicit approval of proposal, design and delta requirements before editing implementation; record approval evidence.
- [x] 1.2 Map C1-C3, B1-B2, V1-V2 and L1-L3 to named automated tests and audit every raster key dependency against actual raster inputs. Confirm no public/persisted contract or owner-edge change is needed; otherwise update artifacts and obtain approval first.

## 2. Core cache implementation

- [x] 2.1 In render_artifact add versioned text/vector key construction and deterministic dependency-variation tests for C1-C3, including verified font identities, advanced layout, effective SVG order and composed sampling scale.
- [x] 2.2 In render_artifact implement immutable bounded LRU retention and tests for exact byte/entry boundaries, oversize bypass, checked accounting, failed raster retries, concurrent clones and unavailable-cache fallback (B1-B2).
- [x] 2.3 In renderer orchestration carry evaluated revision/project identity into cache preparation and share cache ownership across clones without inspecting persisted records in downstream owners (C1-C3, L3).
- [x] 2.4 Route shaped-text and shape/SVG raster preparation through cache lookup only after existing preflight; preserve workspace writes, local geometry, warnings and diagnostics (C1-C3, V1-V2).

## 3. Core regression evidence and documentation

- [x] 3.1 Add deterministic cold/warm raster fixtures with independent expected pixels and actual raster-call counters; cover content/style/font/layout/vector/density invalidation and component/repeater reuse (C1-C3).
- [x] 3.2 Add warmed-cache failures for finite/complexity checks, missing references, corrupt/missing fonts, unsafe paths, unsupported SVG and injected artifact writes; assert error precedence and absence of publication (V1-V2).
- [x] 3.3 Exercise frame, audiovisual range, materialized draft and final export routes with warm/cold fixtures and existing visual/audio/timing tolerances (L1).
- [x] 3.4 Exercise standalone and batch alias edits, failed atomic edits, stale revisions, missing targets, undo/redo and fresh reopen; confirm unchanged canonical contract fixtures and persisted snapshots (L2-L3).
- [x] 3.5 Document cache scope, budgets, revision invalidation, compatibility and unchanged coordinate/timing/fallback semantics in renderer documentation; maintain a scenario-to-test verification report with exact check exits and external log locations.

## 4. Required verification before archival

- [x] 4.1 Run root `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, including core architecture and deterministic rendering tests. Record any environmental limitations as blocking, not passes.
- [x] 4.2 From apps/agent-bridge run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration` and `bun run test:smoke`. Existing canonical fixtures remain unchanged; any unexpected contract difference blocks completion.
- [x] 4.3 From apps/agent-bridge run `bun run scripts/run-python-tests.ts` for the hermetic provider-worker regression suite required by repository policy.
- [x] 4.4 Run root `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `moon run root:openspec-validate`; inspect the full protected-gate result. Only rejection caused solely by this active change is expected before archival; any other failure blocks archival.
- [x] 4.5 Use `$openspec-verify-change` after implementation checks pass, reconcile code/design/spec/tasks and prove automated coverage of every normative scenario. Record conformance evidence and resolve all mismatches.

## 5. Synchronization and final protected gate

- [x] 5.1 Use `$openspec-sync-specs` and `$openspec-archive-change` on the verified change; preserve unrelated changes.
- [x] 5.2 After archival run `moon run root:openspec-validate` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` again. Require both to pass before declaring completion.
