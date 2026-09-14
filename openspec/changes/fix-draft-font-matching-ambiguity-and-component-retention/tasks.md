## 1. Approved artifacts and regressions

- [x] 1.1 Record the user-approved plan, constraints and failure cases in proposal, design and delta specs.
- [x] 1.2 Add failing regression tests for complete ambiguity, component child retention and draft-v2 representability; run `cargo test -p opencut-editor-core --test font_draft_retention`.

## 2. Editor-core implementation

- [x] 2.1 Implement bounded bidirectional matching and deterministic equivalent outcomes.
- [x] 2.2 Apply scoped component retention and reject unrepresentable steps before publication.
- [x] 2.3 Complete lifecycle, identity, exact/equivalent and atomic failure tests; update docs/text-layout.md and requirement-to-test traceability.

- [x] 2.4 Extend artifacts with the user-approved selector-reversion correction before implementation.
- [x] 2.5 Add and run failing selector-reversion regressions, then implement explicit resolution markers and inherited replay checks; update traceability and documentation.
- [x] 2.6 Rerun all required checks with Rust 1.97.0 and verify the correction. Updated evidence for 3.1–3.3 is recorded in verification.md; the unrelated Moon blocker remains.

- [x] 2.7 Record the approved preceding-bindings plan and reopen conformance verification before implementation.
- [x] 2.8 Add failing prefix regressions, implement weighted alternatives and chronological preparation, and extend exhaustive and lifecycle coverage.
- [x] 2.9 Rerun all repository checks using Rust 1.97.0 and reconcile documentation and OpenSpec conformance for the preceding-bindings correction. Implementation checks pass; the unrelated Moon blocker keeps task 3.4 pending.

## 3. Verification and completion

- [x] 3.1 Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` including font lifecycle, shaping, draft and component suites.
- [x] 3.2 From apps/agent-bridge run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, and `bun run test:smoke`; run hermetic Python unittest/pytest worker suites.
- [x] 3.3 Run pinned strict OpenSpec validation and `moon run root:openspec-validate`, inspect full logs and verify conformance using openspec-verify-change. Implementation conformance passed; Moon's unrelated active-change blocker is recorded in verification.md.
- [ ] 3.4 Synchronize and archive after required verification gates permit it; rerun Moon and strict all-spec validation after archival. Report any unrelated gate failure without changing unrelated work.
