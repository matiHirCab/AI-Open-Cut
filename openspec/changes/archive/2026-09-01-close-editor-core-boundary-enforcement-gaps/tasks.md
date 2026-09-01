## 1. Project Path Confinement

- [x] 1.1 Add deterministic fake-storage tests for linked-entry exclusion and canonical paths outside `projects_root`, asserting `PATH_NOT_ALLOWED` occurs before lock, read, recovery, or GC.
- [x] 1.2 Add a Unix real-filesystem regression showing a directory symlink under `projects_root` is omitted from `list_projects`.
- [x] 1.3 Classify storage entries without following links, skip linked project/managed entries, validate canonical project-root confinement, and route project-asset canonicalization through the injected storage adapter.
- [x] 1.4 Re-run focused persistence, recovery, draft, history, and GC tests and confirm ordinary error/warning behavior is unchanged.

## 2. Parsed Rust Coverage

- [x] 2.1 Add failing regressions for nested production `cfg_attr(path)`, standalone `#[test]`, `self::super` paths, `extern crate self`, external-crate aliases, alias cycles, and similarly prefixed owner identifiers.
- [x] 2.2 Recursively inspect conditional attributes, exclude only production-disabled or direct test items, and preserve owner/file parse diagnostics.
- [x] 2.3 Normalize relative roots and resolve lexical, module, and `extern crate` aliases to a deterministic fixed point with exact owner matching.

## 3. Filesystem Responsibility Enforcement

- [x] 3.1 Add regressions rejecting native `Path` filesystem methods and aliased equivalents in assets, store, renderer, and render-planning owners while accepting authorized adapter calls.
- [x] 3.2 Rename overlapping private adapter methods, route the remaining direct store canonicalization through storage, and record native filesystem method calls structurally.
- [x] 3.3 Preserve the canonical asset-GC delegation assertion and confirm persistence/render-artifact adapters remain the only filesystem owners.

## 4. Verification and Delivery

- [x] 4.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, focused native-renderer coverage, and `git diff --check main...HEAD`.
- [x] 4.2 From `apps/agent-bridge`, run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, and `bun run test:smoke`; run the hermetic Python worker tests.
- [x] 4.3 Run `bun run scripts/normalize-openspec-workflows.ts --check`, `moon run openspec-validate`, and strict pinned OpenSpec validation.
- [x] 4.4 Run `$openspec-verify-change`, resolve every mismatch, archive the change, and repeat `$code-review` against `main...HEAD` until no actionable findings remain.
- [x] 4.5 Create one additional commit excluding `docs/motion-graphics-implementation-plan.md`, push without force to `feat/issue-83-editor-core-boundaries`, and wait for all five PR #94 checks to succeed.
