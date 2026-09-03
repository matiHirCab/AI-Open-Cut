## 1. Canonical Fixture and Manifest

- [x] 1.1 Add editor-core test support for the fixed 160x90, 10-fps, one-second synthetic visual/audio project and generated local tone without duplicating evaluator, timeline, animation, or render-plan rules.
- [x] 1.2 Define a versioned strict golden manifest with bounded finite values, unique sample timestamps, safe fixture-relative references, file hashes, environment identity, and unknown-version rejection.
- [x] 1.3 Implement narrow path/workspace token normalization for semantic plans and filter graphs, with tests proving path variability normalizes and semantic argument changes do not.
- [x] 1.4 Add manifest regressions for missing/duplicate references, hash mismatch, non-finite/out-of-range values, absolute paths, traversal, URI input, root escape, and incomplete dependency identity.

## 2. Capture and Comparison Harness

- [x] 2.1 Implement verification mode that renders production frame preview, audiovisual range preview, and export, decodes the declared samples/audio, probes timing, and compares exact semantic/graph evidence plus SSIM >= 0.99, float-PCM RMS <= 0.0001, and one-frame timing tolerance.
- [x] 2.2 Implement explicit update mode that stages every reference in a temporary sibling, validates the complete manifest and hashes, atomically publishes the set, and preserves the prior set on injected render, decode, hash, validation, or publication failure.
- [x] 2.3 Implement report-only baseline capture with fixture/git/tool/font/platform identity, units, warm-up/sample counts, scene-evaluation, filter construction, frame, range, export, total timing, and peak resident working-set memory; do not add performance pass/fail thresholds.
- [x] 2.4 Add repeat-run and intentional-drift tests that prove stable semantic/graph output, tolerance-bound decoded output, and failure when all production entry points drift together from a reviewed reference.

## 3. Golden Evidence and State Integrity

- [x] 3.1 Capture and check in the reviewed manifest, first/middle/final lossless frame references, decoded short-audio reference, exact normalized semantic plan, exact normalized filter graph, hashes, and an initial platform-tagged timing/memory observation without checking in temporary workspaces or machine-local paths; required Linux CI emits its canonical report as a retained workflow artifact.
- [x] 3.2 Add invalid-timing and missing-asset tests that assert `INVALID_ARGUMENT` and `ASSET_NOT_FOUND` respectively before process, file, reference, or publication side effects and prove the immutable project remains unchanged.
- [x] 3.3 Add stale-revision, successful-render, undo/redo, and reopen evidence through existing typed core/headless behavior, proving stable errors, unchanged render state/history, and deterministic evaluated/golden results without a schema migration or new operation.
- [x] 3.4 Make configured native golden dependencies fail closed rather than skip, including unusable FFmpeg/FFprobe, missing required filters, unreadable or mismatched font, and malformed golden data.

## 4. Documentation and CI

- [x] 4.1 Document fixture semantics and provenance, reference formats, normalization tokens, tolerance calculations, dependency/platform scope, verification/update/report commands, atomic recovery, and the reviewer workflow for intended diffs.
- [x] 4.2 Update the motion-graphics implementation plan to record the captured milestone-zero baseline scope while preserving later fixture and performance-budget roadmap work.
- [x] 4.3 Replace the current relative-only native parity invocation in Linux CI with the required golden conformance gate using explicit FFmpeg, FFprobe, and deterministic font configuration, while retaining the render lifecycle test.

## 5. Verification and Completion

- [x] 5.1 Run the focused golden manifest, normalization, update-rollback, conformance, failure, and lifecycle tests with explicit native dependencies, and regenerate into a temporary comparison directory to prove the checked-in set is reproducible.
- [x] 5.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [x] 5.3 Run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, and `bun run test:smoke` from `apps/agent-bridge`, plus the relevant hermetic Python worker tests, and record any environment-gated checks explicitly.
- [x] 5.4 Run `moon run openspec-validate`, `git diff --check`, and inspect the final diff for unintended runtime/public/persisted contract changes, generated workspaces, machine-local paths, or unrelated edits.
- [x] 5.5 Apply `$openspec-verify-change`, resolve every mismatch among requirements, design, tasks, fixtures, tests, documentation, and implementation, then archive with `$openspec-archive-change` so the accepted deltas update living specifications.
