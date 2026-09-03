## 1. Safe Golden Generations

- [x] 1.1 Add strict versioned `CURRENT` parsing, content-addressed generation resolution, manifest-digest validation, and migration of the checked-in flat fixture to revision 2 under `generations/<digest>`.
- [x] 1.2 Implement staged immutable-generation installation and cross-platform atomic pointer replacement, treating pointer replacement as the commit point and post-commit cleanup failure as non-fatal.
- [x] 1.3 Implement bounded cleanup for recognized staging and inactive digest generations while preserving the selected generation and every unknown path.
- [x] 1.4 Add injected failure and reopen tests covering pre-install, pre-pointer, pointer-replacement, post-commit cleanup, interrupted staging, orphan generation, and unknown-path preservation.

## 2. Reference Validation and Audio Comparison

- [x] 2.1 Reject RFC 3986-style URI schemes before filesystem path parsing while retaining absolute, traversal, canonical-root, and symlink escape protection.
- [x] 2.2 Add URI validation tests for `file:`, `data:`, mixed-case and scheme punctuation, plus valid portable relative references.
- [x] 2.3 Replace end-only PCM trimming with bounded bidirectional offset search and minimum-overlap enforcement, retaining the existing one-frame and RMS limits.
- [x] 2.4 Add non-periodic PCM tests for zero, positive, negative, boundary, excessive, non-finite, insufficient-overlap, and genuine-drift cases.

## 3. Reproducible Performance Capture

- [x] 3.1 Add development-only cross-platform process inspection and Windows atomic-file support without adding production dependencies or changing production process interfaces.
- [x] 3.2 Split one-shot conformance capture from sampled performance capture, running one discarded warm-up plus three measured captures only for report, recapture, or update modes and requiring their deterministic evidence to agree.
- [x] 3.3 Implement five-millisecond recursive process-tree RSS sampling, median phase/total timing aggregation, maximum memory aggregation, and schema-2 scope/aggregation metadata.
- [x] 3.4 Add deterministic aggregation tests and a platform-aware child-process allocation test proving descendant memory contributes to the reported peak.

## 4. Fixture, Documentation, and CI Evidence

- [x] 4.1 Recapture and validate the revision-2 content-addressed generation with declared FFmpeg, FFprobe, and DejaVu dependencies in a Linux environment compatible with required CI.
- [x] 4.2 Update golden documentation for generation selection, URI rules, bidirectional alignment, sampling counts, aggregation, process-tree memory, atomic commit, cleanup, and rollback.
- [x] 4.3 Update the Linux CI artifact expectations for performance schema 2 while preserving required native conformance and lifecycle coverage.

## 5. Verification and Completion

- [x] 5.1 Run focused editor-core golden unit tests and the native golden conformance/update/recapture/report commands with `OPENCUT_GOLDEN_REQUIRED=1` and explicit FFmpeg, FFprobe, and font paths.
- [x] 5.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the exact headless lifecycle test.
- [x] 5.3 From `apps/agent-bridge`, run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, and `bun run test:smoke`; also run the relevant hermetic Python worker tests and record any environment-gated result.
- [x] 5.4 Run `bun run scripts/normalize-openspec-workflows.ts --check`, `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`, and `git diff --check`; inspect the final diff for runtime/public/persisted contract drift, temporary captures, and unrelated changes.
- [x] 5.5 Apply the repository-local `$openspec-verify-change`, resolve every mismatch, sync the accepted delta into living specs, and archive `correct-golden-render-fixture-conformance` only after all required checks pass.
