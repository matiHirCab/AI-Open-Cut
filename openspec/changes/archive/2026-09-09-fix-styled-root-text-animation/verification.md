## Approval and scope
The user explicitly requested implementation of the detailed plan on 2026-09-09. Artifacts transcribe that plan. The follow-up changes editor-core evaluation and regression coverage only; no public contract, migration, canonical catalog or dependency edge changed. The original issue #32 archive remains intact.

## Requirement coverage
| Scenario | Implementation / evidence |
| --- | --- |
| Animate each supported legacy property | apply_ancestors stores identity ancestry and measure_layer_affine no longer synthesizes a private parent. styled_root_retains_identity_ancestry_through_finalization covers position, scale and opacity-only evaluation. native_styled_root_animation_conformance compares samples at 0/400/800 ms against explicit static values; it checks 16-pixel position steps, increasing visible width, and increasing brightness. |
| Preserve ancestry and transform compatibility | Evaluator tests retain matrices/clip/opacity through finalization, check plain-text legacy routing and center anchors, preserve group/component transforms and opacity, and assert established rejection of Transform2D with legacy keyframes plus successful finalization without those keyframes. Native snapshots use center anchors. |
| Agree across render intents without freezing | The native animation test compares frame, range, materialized draft and export at all three timestamps against an independent static oracle. SSIM remains >=0.99, draft frames are exact, and authoritative project bytes remain unchanged after fixture media normalization. Existing audiovisual conformance retains audio tolerance coverage. |

## Reproduction and test design
Before the fix, the evaluator regression failed because styled root ancestry was absent. The native regression failed its position static oracle at 400 ms. Both passed after the fix. The motion fixture uses a neutral run-color override (different from the item's base color), keeping styled preparation active while avoiding lossy chroma-subsampling noise; colored-run conformance remains in the original fixture. No existing golden baseline or tolerance was changed. The existing colored-run native_rich_text_render_conformance also passed with the restored animation (13.02 seconds), independently confirming the original color/preview/export checks.

## Checks
| Command | Result |
| --- | --- |
| cargo fmt --check --all | Passed. |
| cargo clippy --workspace --all-targets -- -D warnings | Passed. |
| cargo test --workspace -- --test-threads=1 | Passed; existing opt-in/ignored test status unchanged. Required native drivers run separately below. |
| cargo test -p opencut-editor-core styled_root_ -- --nocapture with required native resources | Passed: evaluator and native regressions; final component-parent extension also passed its focused test. |
| cargo test -p opencut-editor-core native_golden_render_conformance -- --nocapture | Passed: 300.44 seconds with OPENCUT_GOLDEN_REQUIRED=1, cached FFmpeg/ffprobe 8.1.2 and checked-in DejaVuSans.ttf. |
| cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact | Passed: 0.70 seconds. |
| bun run typecheck; bun run lint (apps/agent-bridge) | Passed. |
| bun run test (apps/agent-bridge) | Passed: 390 tests / 20 files. |
| bun run contracts:check (apps/agent-bridge) | Passed: Rust suites and 326 TypeScript tests / 7 files. |
| bun run test:integration (apps/agent-bridge) | Passed: 11 tests. |
| bun run test:smoke (apps/agent-bridge) | Passed: 6 tests. |
| bun run ../agent-bridge/scripts/run-python-tests.ts (apps/kokoro-tts) | Passed: 10 Kokoro and 5 Whisper tests. |
| bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive | Passed: 25 items. |
| git diff --check | Passed. |
| moon run root:openspec-validate | Passed after archive: 24 living specifications and CI parity policy validated. |

Command logs are in Windows TEMP/opencut-animation-*. No baseline recapture or contract fixture changes were made for this fix.

## Assessment
OpenSpec verification reviewed completeness, correctness and coherence against the approved design and all three scenarios. All implementation and required execution checks passed; no remaining code/design/spec mismatch, critical finding, warning or suggestion remains. All seven tasks are complete. Accepted requirements are synchronized in openspec/specs/rendering-export/spec.md. The follow-up is archived at openspec/changes/archive/2026-09-09-fix-styled-root-text-animation; the original issue #32 archive remains intact. The final Moon gate passed on 2026-09-09.

## CI portability correction
PR #118's first CI run used Rust 1.98, while local validation used Rust 1.93. Its newer Clippy rejected four fixed-size chunks_exact calls in the new regression tests. These now use as_chunks::<3>().0.iter(), preserving identical pixel iteration without suppressing warnings. Render parity also exposed a recipe checksum computed from local CRLF bytes instead of Git's committed LF bytes. The reference envelope now records the committed recipe's SHA-256, 14e7426afd82c68aef19922e8c1ef06e803a2711664250d932df915f3218e904, and the local recipe is normalized to the existing .gitattributes LF policy. Embedded frame, audio and semantic-plan baselines are unchanged. These corrections maintain the already-approved test/fixture verification tasks and introduce no executable product behavior or public contract changes.
