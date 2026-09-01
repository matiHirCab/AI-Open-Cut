## 1. Project confinement

- [x] 1.1 Add fake-storage counters and regressions proving an empty external canonical target returns `PATH_NOT_ALLOWED` before marker probes, locks, reads, recovery, listing, or garbage collection, while an ordinary missing project remains `PROJECT_NOT_FOUND`.
- [x] 1.2 Canonicalize and validate the requested project directory before marker probes in store orchestration, preserving all other canonicalization diagnostics and normal in-root behavior.
- [x] 1.3 Update the Unix linked-project regression to cover an empty external target and retain real-project, recovery, draft, history, asset, and garbage-collection behavior.

## 2. Architecture enforcement

- [x] 2.1 Refactor alias discovery so production module items and block-local `use`, `extern crate`, and path-based type aliases share inherited, fixed-point, cycle-safe canonicalization.
- [x] 2.2 Add `is_symlink` to the native `Path` filesystem operation set without changing permitted adapter-specific operations or the ADR dependency matrix.
- [x] 2.3 Add regressions for local crate/external/type alias chains, test-only local aliases, exact owner matching, method and UFCS symlink inspection in every restricted owner, cycles, and permitted adapters.

## 3. Verification and delivery

- [x] 3.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the targeted native renderer, recovery, drafts, and garbage-collection coverage.
- [x] 3.2 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, the MCP integration suite, and packaged fake-provider smoke.
- [x] 3.3 Run the exact `moon run openspec-validate` steps, apply `$openspec-verify-change`, and resolve every discrepancy before archival.
- [x] 3.4 Run `$code-review` against `main...HEAD`, resolve every actionable finding, and confirm `docs/motion-graphics-implementation-plan.md` remains untracked.
- [x] 3.5 Confirm the implementation did not absorb work assigned to issues #12, #13, #29, #73, #79, #81, #82, or #85.
