## 1. Base and approved contract

- [x] 1.1 Materialize isolated checkout and pin PR131 head e1010bb97174d44a5d19c61a90514bf2336845ca; inspect reviews, CI, instructions and inherited sampling code.
- [x] 1.2 Validate complete proposal, design, delta and tasks; record bounded delegated OpenSpec approval.
- [ ] 1.2a Obtain designated contract-owner implementation review before archival.
- [x] 1.3 Add canonical motion-blur catalog, accepted/rejected and independent timing/composition fixtures, ownership and capability declarations.

## 2. Canonical persisted model and lifecycle

- [x] 2.1 Implement finite typed motionBlur leaf settings and eligible-target validation in editor-core; preserve unknown-field and stable-error policy.
- [x] 2.2 Extend update_item and alias-aware batch handling with atomic rollback, revisions and undo/redo tests.
- [x] 2.3 Atomically adopt schema28 current/history/drafts with legacy identity and premature/future-field failure fixtures; prove byte-preserving reopen.

## 3. Evaluated sampling and preparation

- [x] 3.1 Implement checked centered midpoint sample policy with integer-root flooring, interval clamping and independent boundary oracles.
- [x] 3.2 Carry authored settings through EvaluatedScene and evaluate complete ancestor, nested, repeated and staggered clocks per sample.
- [x] 3.3 Reuse issue43 local crop/clip/effects/affine pipeline; average weighted premultiplied canvas rasters before stacking and preserve cache dependencies.
- [x] 3.4 Bound cumulative sample-weighted pixel, geometry and effect work before persistence/artifact I/O; test cleanup and encoder failures.

## 4. Governed consumers

- [x] 4.1 Synchronize headless, bridge/Zod/MCP catalogs and desktop declarations without parallel domain rules; cover standalone and batch aliases.
- [x] 4.2 Document coordinate, timing, boundary, composition, fallback, numeric and migration semantics and performance limits.

## 5. Conformance and final review

- [x] 5.1 Run cargo fmt --check --all; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace.
- [x] 5.2 From apps/agent-bridge run bun run typecheck; bun run lint; bun run test:unit; bun run contracts:check; bun run test:integration; bun run test:smoke. Run applicable documented hermetic Python worker runner.
- [ ] 5.3 Run required native FFmpeg frame/range/draft/export oracle fixtures, decoded audio synchronization and existing regression corpora according to docs/render-regression-fixtures.md; measure sample-count performance and record limits. Do not count optional early returns as render evidence.
- [ ] 5.4 Run strict OpenSpec validation and protected Moon pre-archive gate; invoke openspec-verify-change and resolve every mismatch. Synchronize/archive only after all required checks pass, then rerun final protected gate and strict validation.
- [x] 5.5 Record exact commits/diff, dependency state, full verification results and proposed PR text. Leave branch local pending separate publication approval.
