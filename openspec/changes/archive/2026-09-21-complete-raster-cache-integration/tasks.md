## 1. Approval and traceability

- [x] 1.1 Obtain explicit approval of this proposal, design, delta specs and tasks before implementation; record it in the proposal. Preserve existing issue-36 work and its archive.
- [x] 1.2 Create a verification index mapping W1-W9 and X1-X6 plus retained C1-C3/B1-B2/V1-V2/L1-L3 to named tests. Record the one-shot architectural clarification and distinguish confirmed behavior from evidence gaps.
- [x] 1.3 Establish pinned Bun 1.4.0, Moon 2.3.3 and Rust 1.97.0; record actual versions. Resolve the local Rust 1.93.0 discrepancy without changing repository pins. Record unavailable required checks as blocking.

## 2. Cross-language contract definition

- [x] 2.1 Add canonical render-worker-v1 fixtures for readiness, closed request/event envelopes, accepted render operations, limit boundaries and typed rejection cases before changing consumers (W2-W5).
- [x] 2.2 Register the worker artifact and native/TypeScript consumers in contract ownership, add shared parity assertions, and obtain designated CODEOWNER review per ADR 0002. Confirm no project-schema migration or MCP surface change is needed (W2-W5).

## 3. Editor-core request scoping and cache evidence

- [x] 3.1 Add the request-scoped Renderer facade method and delegating ArtifactIo identity adapter. Test identity rejection, clone cache sharing, unchanged underlying adapter behavior, independent temporary paths and legacy fallback (W7, W9).
- [x] 3.2 Add default-disabled raster-cache-test-hooks counters usable by the instrumented headless build, without normal wire/debug exposure. Preserve existing bounds, immutable retention and concurrency semantics (X1-X2, B1-B2).
- [x] 3.3 Add valid production-path dependency variations with cached/fresh equality, actual miss/hit assertions and unrelated retained-key reuse. Cover text/runs/spans/layout/fonts/paint stacks, vectors, SVG ordering/viewport and composed sampling; retain focused implementation-version key coverage (X3, C2).
- [x] 3.4 Add composition-only component/repeater/timing/position/opacity cases with byte reuse and independent plan/placement assertions; cover distinct same-revision drafts and changed revision/project identity (X2, X4, C1-C3).
- [x] 3.5 Extend warm preflight failures across applicable render routes for finite values, references, fonts, paths, unsupported SVG, aggregate work and readiness. Assert exact cold/warm errors, both counters, adapter events and unchanged authoritative state (X5, V1).
- [x] 3.6 Verify invalid-input precedence over export collisions and warm workspace-write failure cleanup/no partial publication (X6, V2).

## 4. Headless worker transport

- [x] 4.1 Extract shared render dispatch/event emission without duplicating revision, draft, path, core or publication rules; retain one-shot CLI/health/exit behavior and native contract parity (W2, W9).
- [x] 4.2 Add explicit --render-worker mode with retained services/renderer, versioned readiness, request-scoped clones, bounded incremental UTF-8 line framing and correlated events. Reject non-render requests before dispatch (W3-W5, W9).
- [x] 4.3 Add headless tests for sequential requests, fresh revision/draft reads, stale revisions, typed-error continuation, malformed/oversized input, non-render rejection, EOF and one-shot compatibility (W2-W5, W8-W9).
- [x] 4.4 Forward the test-hooks feature and emit instrumented worker evidence on stderr only in test builds. Prove normal builds retain canonical stdout and no cache-statistics contract (X1).

## 5. Bridge worker lifecycle

- [x] 5.1 Add synchronous reservation of one lazy persistent render worker per HeadlessClient, one-shot overflow for overlapping renders and unchanged one-shot routing for non-render work (W1-W2).
- [x] 5.2 Implement readiness/version checks, per-request deadline including startup, closed correlated event parsing, single-terminal enforcement and existing typed error mapping; never replay a dispatched request (W3-W5).
- [x] 5.3 Implement worker retirement, process-tree termination, wait-before-cleanup ordering, independently scoped cancellation and shutdown. Keep unconfirmed termination retired and prevent client-close races from spawning replacement workers (W6-W8).
- [x] 5.4 Include draft-preview temporary files in owned cleanup, preserve final artifacts, and test timeout/cancel/crash/malformed-output isolation from concurrent overflow requests (W6-W9).

## 6. Native conformance and documentation

- [x] 6.1 Add native bridge/headless successive-request tests proving real avoided raster calls through the same worker, overflow independence, typed-error reuse and cold restart. Run with the instrumented feature and separately verify default binaries (X1-X2, W1-W9).
- [x] 6.2 Compare cold/warm/fresh frame, range, draft and export bytes/plans/warnings/diagnostics and independent visual/audio/timing evidence; preserve aliases, rollback, history and reopen checks (X1-X6, L1-L3).
- [x] 6.3 Document worker protocol/lifetime, legacy compatibility, startup/cancel/restart behavior, scope of overflow reuse and retained versus in-flight memory. Audit ownership against ADRs 0002/0003; update ADR 0003 and architecture checks only if an approved new edge is required.

## 7. Required implementation checks

- [x] 7.1 Run root `cargo fmt --check --all` and `cargo clippy --workspace --all-targets -- -D warnings`; additionally run `cargo clippy --workspace --all-targets --features opencut-headless/raster-cache-test-hooks -- -D warnings`. Keep full external logs and exact exits.
- [x] 7.2 Run `cargo test --workspace` with OPENCUT_GOLDEN_REQUIRED=1, compatible FFmpeg/FFprobe 7.1.1 paths and the reviewed fixture font. Preserve observed parallel failures separately; if a serial rerun is needed, record RUST_TEST_THREADS=1 and do not claim parallel reliability. Never modify goldens or weaken assertions.
- [x] 7.3 Run `cargo test -p opencut-editor-core --lib raster_cach` and `cargo test -p opencut-headless --features raster-cache-test-hooks` with required native dependencies. Build the instrumented headless binary with `cargo build -p opencut-headless --features raster-cache-test-hooks` for feature-specific bridge integration, then rebuild the default binary with `cargo build -p opencut-headless` before ordinary integration/smoke checks.
- [x] 7.4 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration` and `bun run test:smoke`. Ensure both instrumented worker evidence and default packaged compatibility execute rather than silently skip.
- [x] 7.5 From apps/agent-bridge run `bun run scripts/run-python-tests.ts` for the required hermetic worker regressions.
- [x] 7.6 Run root `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `moon run root:openspec-validate`. Before archival, accept only the expected gate rejection identifying this active change; any other failure blocks progress. Record approved locally installed pinned-CLI fallback if sandbox restrictions prevent bunx.

## 8. Verification and archival

- [x] 8.1 Run openspec-verify-change after all implementation checks pass. Reconcile every scenario, design decision, fixture, code edit and check with the verification index; unresolved or skipped required checks block completion.
- [x] 8.2 Use openspec-sync-specs and openspec-archive-change to synchronize and archive only this approved follow-up. Preserve the original archive and unrelated work.
- [x] 8.3 Rerun `moon run root:openspec-validate` and strict pinned all-spec validation after archival. Record final passing evidence, actual toolchain/environment, contract-owner review and any limitations before declaring completion. Do not commit, push or create a PR unless separately requested.
