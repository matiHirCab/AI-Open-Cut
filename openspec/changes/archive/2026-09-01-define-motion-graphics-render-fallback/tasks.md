## 1. Fallback Decision and Evidence

- [x] 1.1 Extend ADR 0004 with a named fallback subsection covering deterministic local priority, complete-scene conformance, shared preview/export selection, prohibited degradation and network acquisition, `DEPENDENCY_UNAVAILABLE`, and pre-publication failure.
- [x] 1.2 Strengthen the editor-core ADR test with fallback-specific required phrases and a focused fixture that removes only the fallback subsection and asserts the missing-policy diagnostic.

## 2. Contract and Compatibility Audit

- [x] 2.1 Confirm schema version 6, capability reporting, `contracts/contract-ownership-v1.json`, and all versioned fixtures remain unchanged because this correction reuses `DEPENDENCY_UNAVAILABLE` and adds no runtime or public surface.
- [x] 2.2 Confirm the archived `2026-09-01-record-motion-graphics-architecture` change remains unmodified.

## 3. Verification and Specification Sync

- [x] 3.1 Run `bunx @moonrepo/cli@2.3.3 run openspec-validate`, `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `bun run contracts:check` from `apps/agent-bridge`, and `git diff --check`.
- [x] 3.2 Use `$openspec-verify-change` to confirm proposal, design, requirement, scenarios, ADR, tests, and tasks agree without critical issues or warnings.
- [x] 3.3 Sync the modified `motion-graphics-architecture` requirement into the living spec, then archive `define-motion-graphics-render-fallback` with `$openspec-archive-change`.
