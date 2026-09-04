## 1. Specification approval

- [x] 1.1 Create proposal, design, and delta specifications matching the requested plan.
- [x] 1.2 Obtain explicit approval of these concrete artifacts before implementation and record it in the proposal.

## 2. Core image geometry

- [x] 2.1 Add hermetic parser/adapter coverage for frame dimensions, absent/reflected matrices, malformed matrices, invalid dimensions, missing/extra frames, oversized metadata output, and probe failure (Frame-derived static image geometry).
- [x] 2.2 Extend the private geometry port with resolved source kind; implement bounded first-frame image inspection and retain stream-only video inspection, typed failures, and existing matrix logic (Frame-derived static image geometry).
- [x] 2.3 Extend renderer adapter tests for repeated image references, canonical paths, one probe, no materialization re-probe, unchanged project/history, missing-asset precedence, and zero calls/writes for text overflow in both collision states (both changed capabilities).

## 3. Native regression coverage

- [x] 3.1 Add temporary asymmetric JPEG generation and a test-only EXIF builder for absent orientation and values 1-8; compare legacy/identity extent and pixels, including the 800-pixel orientation-6 regression (Preserve EXIF image extent).
- [x] 3.2 Exercise reflected and quarter-turn images with combined transforms/noncentral anchors through draft/frame/range/export; assert SSIM >= 0.99, timing within one frame, and draft isolation (Preserve EXIF images across render intents).
- [x] 3.3 Run `cargo test -p opencut-editor-core --test transform2d` in required native mode with all three configured dependency paths; repeat with absolute executable paths and FFmpeg absent from PATH. Preserve optional/partial/required configuration tests and existing video/audio tests.

## 4. Verification and completion

- [x] 4.1 Update coordinate documentation and add a correction note linking this follow-up from the previous verification archive; retain historical evidence.
- [x] 4.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` with ordinary configuration. Run the required native golden command `cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact` and lifecycle command `cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact` with configured dependencies and OPENCUT_GOLDEN_REQUIRED=1.
- [x] 4.3 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`. Python/provider surfaces are unchanged and require no new checks unless implementation touches them.
- [x] 4.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; use openspec-verify-change and write scenario-to-test evidence with actual results and limitations. Confirm Linux/Windows/macOS correctness CI before merge; explicitly report unavailable remote evidence.
- [x] 4.5 Synchronize accepted delta specifications, archive using openspec-archive-change, and run final `moon run root:openspec-validate` (pinned Moon CLI fallback if needed). Do not claim completion when a required check fails or is skipped.

Remote CI evidence remains pending before merge, as recorded in verification.md; local completion does not attest the Linux/macOS jobs.
