## 1. Replaceable Persistence

- [x] 1.1 Add facade-level fake-storage regressions for lock, create/list/read, recovery, atomic replacement, draft cleanup, and garbage-collection failures, asserting existing codes and warning strings.
- [x] 1.2 Extend the private storage interface with `Debug + Send + Sync` and an opaque exclusive-lock guard; make `EditorCore` own the selected adapter while preserving its public constructor, `Clone`, and `Debug`.
- [x] 1.3 Route project/draft directory operations, reads, writes, transactions, recovery, managed assets, and GC through the selected adapter; remove concrete filesystem selection and direct filesystem calls from store orchestration.
- [x] 1.4 Replace nominal store-GC enforcement with structural delegation rules and confirm project, history, and durable-draft references retain identical collection behavior.

## 2. Complete Artifact I/O

- [x] 2.1 Add recording/failing artifact-adapter tests for workspace creation, temporary paths, resource/font/filter I/O, path inspection, process-failure cleanup, publication, metadata, and workspace cleanup across frame, range, and export entry points.
- [x] 2.2 Expand the private artifact interface and filesystem implementation to cover the complete workspace/resource/publication lifecycle without adding public APIs.
- [x] 2.3 Retain the injected adapter in `RenderWorkspace` and route asset/font resolution, text/filter preparation, temporary output, cleanup, publication, and metadata through it while preserving existing error stages and best-effort cleanup.

## 3. Architecture Enforcement

- [x] 3.1 Add failing regressions for standard out-of-line modules, rejected `#[path]`, parent crate-root aliases, chained external aliases, structured `tracks` patterns, direct/aliased asset filesystem access, and renamed store-owned GC.
- [x] 3.2 Recursively load standard Rust module files with owner/file diagnostics and model lexical/module alias frames with fixed-point canonical path resolution.
- [x] 3.3 Record structured pattern fields and replace spelling/function-name responsibility checks with structural filesystem, low-level storage, scene-enumeration, and canonical-GC-delegation assertions.
- [x] 3.4 Run `cargo test -p opencut-editor-core --test architecture -- --nocapture` and focused editor-core persistence, recovery, draft, GC, render-plan, renderer-facade, and native-renderer tests.

## 4. Compatibility and Repository Verification

- [x] 4.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `git diff --check main...HEAD`.
- [x] 4.2 From `apps/agent-bridge`, run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, and `bun run test:smoke`; run the repository's hermetic Python worker tests.
- [x] 4.3 Run `bun run scripts/normalize-openspec-workflows.ts --check` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`.
- [x] 4.4 Run `$openspec-verify-change`, resolve every mismatch, archive with `$openspec-archive-change`, and repeat `$code-review` against `main...HEAD` until no actionable finding remains.
- [x] 4.5 Create one additional commit excluding `docs/motion-graphics-implementation-plan.md`, push without force to `feat/issue-83-editor-core-boundaries`, and wait for all five PR #94 checks to succeed.
