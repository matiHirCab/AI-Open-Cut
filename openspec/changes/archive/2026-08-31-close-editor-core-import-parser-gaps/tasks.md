## 1. Regression Coverage

- [x] 1.1 Add failing extractor cases for `super` imports, top-level and nested groups, multiline aliases, root aliases, qualified `crate`/`super` paths, and similarly prefixed non-owners.
- [x] 1.2 Add failing source-structure cases proving test-only items are excluded while forbidden production dependencies after them remain visible.
- [x] 1.3 Add parse-failure and policy-failure assertions that identify the source owner, forbidden dependency, and allowed set.

## 2. AST Enforcement

- [x] 2.1 Add pinned `syn 2.0.119` `full`/`visit` editor-core dev-dependency and confirm no production dependency change.
- [x] 2.2 Replace textual import/path extraction with an AST visitor for complete use trees, qualified paths, module-depth-relative roots, exact owners, and scoped crate-root aliases.
- [x] 2.3 Replace suffix truncation with selective test-only `cfg` subtree exclusion while preserving later production items.

## 3. Verification and Delivery

- [x] 3.1 Run `cargo fmt --check --all`, `cargo test -p opencut-editor-core --test architecture --no-fail-fast`, and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 3.2 Run `cargo test --workspace`; from `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, and `bun run test:smoke`; then run `bun run scripts/normalize-openspec-workflows.ts --check` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`.
- [x] 3.3 Run `$openspec-verify-change` and `$code-review` against `main...HEAD`, and resolve every actionable mismatch.
- [x] 3.4 Sync `editor-core-architecture`, archive `close-editor-core-import-parser-gaps`, and re-run strict OpenSpec validation.
- [x] 3.5 Create and push one follow-up commit to `feat/issue-83-editor-core-boundaries`, excluding `docs/motion-graphics-implementation-plan.md`, and wait for all five PR #94 checks to pass.
