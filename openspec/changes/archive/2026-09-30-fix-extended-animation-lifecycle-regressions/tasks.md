## 1. Approval and evidence

- [x] 1.1 Obtain explicit approval of this proposal, design, two delta specs and tasks; record actual approval before editing implementation.
- [x] 1.2 Preserve the existing working tree and external review logs; map D1-D3 and each new scenario to owned regression tests and fresh correction logs in verification.md.

## 2. Editor-core regression tests and corrections

- [x] 2.1 D1: Add failing public-API draft-base eviction coverage with deleted original target, extended current effects, legacy-only control, successful eviction edit, project read/edit/reopen/discard, and stale apply/preview REVISION_CONFLICT with byte preservation.
- [x] 2.2 D1: Add missing-base schema migration coverage with atomic current/undo/redo adoption and unchanged draft bytes; retain malformed structural/resource failure and matching-base invalid-target rejection controls.
- [x] 2.3 D1: Correct applicable-base selection without unrelated-current fallback, preserve structural/font/resource validation and transactional migration/fault recovery; pass new and existing persistence tests.
- [x] 2.4 D2: Add failing offset identity-vignette transition tests with analytic 0.5 gain and verified legacy control, decoding frame/range/draft/export; cover root, scaled, nested/repeated fractional clocks, supported fade/crossfade and half-open boundaries with owned exact semantic assertions.
- [x] 2.5 D2: Reuse canonical occurrence-clock transition handling for sampled visuals exactly once; preserve opacity, effect ordering, stacking, audio and cache dependencies; pass new and existing rendering tests.
- [x] 2.6 D3: Add failing sampled encoder fault injection that consumes stdin and emits long ASCII/Unicode plus quoted Windows/POSIX private paths; assert typed stage/exit, 4096-byte valid-UTF-8 safe excerpt, process reaping, temporary cleanup and unchanged destination/state.
- [x] 2.7 D3: Use shared core stderr sanitization, preserve bounded collection and existing transport translation; verify safe direct core/headless errors and unchanged MCP diagnostic/catalog compatibility.

## 3. Documentation and compatibility

- [x] 3.1 Confirm schema 27, public operations/aliases, errors/retryability, canonical catalogs, capabilities, CODEOWNERS and ADR dependency edges remain compatible; stop for an approved amendment if any new contract change is needed.
- [x] 3.2 Update verification traceability and add truthful superseding evidence notes to both issue #43 archives; distinguish reproduced failures, passed corrections, unrelated green checks and unavailable platform evidence.

## 4. Required implementation checks

- [x] 4.1 With Rust 1.97.0, run focused red/green regressions, `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; retain full commands, exits and logs outside the repository.
- [x] 4.2 From apps/agent-bridge run `bunx biome check --formatter-enabled=true --linter-enabled=false --assist-enabled=false src tests`, `bun run typecheck`, `bun run lint`, `bun run test:unit`, and `bun run contracts:check`; account for skips and native opt-ins explicitly.
- [x] 4.3 From apps/agent-bridge run `bun run test:integration`, `bun run test:smoke`, and `bun run scripts/run-python-tests.ts`; retain full logs and do not conceal failed initial runs.
- [x] 4.4 With compatible FFmpeg/FFprobe 7.1.1, canonical DejaVu Sans, OPENCUT_GOLDEN_REQUIRED=1 and OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED=1, run `cargo test -p opencut-editor-core --test animation_channels --test transform2d --test font_resolution` and `cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact`; require actual native new transition intent tests to execute.
- [x] 4.5 Run `cargo test --release -p opencut-editor-core --lib renderer::golden::native_golden_render_conformance -- --exact --nocapture` with a fresh external OPENCUT_GOLDEN_REPORT_PATH, then `cargo test --release -p opencut-editor-core --lib renderer::golden::validate_external_performance_report -- --ignored --exact`; never update or recapture goldens.
- [x] 4.6 For 960x540, 1280x720 and 1920x1080 with OPENCUT_RULES_SCREEN_SCOPE=pr and corresponding OPENCUT_RULES_SCREEN_RESOLUTION run `cargo test --release -p opencut-editor-core --lib renderer::golden::rules_screen::native_rules_screen_resolution_conformance -- --exact --nocapture`; distinguish PR evidence from unavailable weekly full scope.
- [x] 4.7 With required native dependencies and OPENCUT_RASTER_CACHE_TESTS_REQUIRED=1 run `cargo test -p opencut-editor-core --lib raster_cach`, `cargo test -p opencut-headless --features raster-cache-test-hooks --test render_worker`, and `cargo build -p opencut-headless --features raster-cache-test-hooks`; from bridge run `bun run test:unit --no-file-parallelism tests/render-worker-native.test.ts`; restore default binary using `cargo build -p opencut-headless` and verify `cargo test -p opencut-headless`.

## 5. Conformance and archival

- [x] 5.1 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `moon run root:openspec-validate` before archival; only rejection naming this active change is expected, all other failures block archival.
- [x] 5.2 Use $openspec-verify-change; resolve all code/spec/design/task/test mismatches, map every new normative scenario to automated evidence, and record real platform/CI/CODEOWNER limits without overstating local checks.
- [x] 5.3 Use $openspec-sync-specs and $openspec-archive-change after conformance and required checks pass; synchronize both living specs and preserve historical archive evidence with correction links.
- [x] 5.4 Require final `moon run root:openspec-validate`, `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`, `bun --config=NUL --no-env-file run scripts/run-ci-policy.ts`, and `git diff --check` to pass; verify protected gates and goldens unchanged, report fixes and limits without commit/push/PR.
