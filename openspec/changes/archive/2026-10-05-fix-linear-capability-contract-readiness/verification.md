# Readiness conformance correction evidence

Approval is recorded in approval.md before the sole test edit. Production renderer/reporter, canonical catalogs, schemas, toolchains and protected workflows are unchanged. Issue50 work is preserved in root's stash e0458ce94a72be78ef14f415a769e2864e798049; this change does not activate or restore it.

## Scenario mapping

| Scenario | Actual automated evidence |
| --- | --- |
| Detect ready corrected rendering | Existing ready_renderer_advertises_canonical_linear_composition_in_protocol_v1 executes default and explicit protocol1 requests with exact canonical rendering/editor/top-level lists and unique linear capability; configured native required mode passed1/1 |
| Omit unsupported capability | Same test passed1/1 with deliberately nonexistent FFmpeg/FFprobe and native-required mode unset; exact empty rendering list/editor-only top-level list/stable nonretryable DEPENDENCY_UNAVAILABLE; explicit missing health regression unchanged and passed1/1 |
| Verify unavailable dependencies in hermetic conformance | Controlled missing-tools focused-unavailable.log executes both requests, no conditional return, zero ignored, asserts boolean false and exact absence at every scope |
| Require configured native readiness evidence | Configured /usr/bin/ffmpeg,/usr/bin/ffprobe, bundled reviewed DejaVuSans.ttf and OPENCUT_GOLDEN_REQUIRED=1 passed focused-native-required.log; deliberate missing-tools+required mode returns101 specifically at required-ready assertion, proving fail-closed behavior |

All logs are external in /workspace/.opencut-tools/logs/issue49-readiness/. Shells source activate.sh/native-env.sh, process-tree commands use reap.py. The deliberate negative invocation is intentional expected-failure evidence, not a waived failed required gate: its exit101 and readiness assertion were explicitly verified. Ordinary full contracts/workspace runs unset OPENCUT_GOLDEN_REQUIRED, OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED, OPENCUT_RASTER_CACHE_TESTS_REQUIRED and native tool/font path overrides.

## Current checks

- Focused controlled unavailable1/1 passed; actual configured native-required1/1 passed(.26s); explicit missing health1/1 passed; native-required negative missing invocation rejected as expected.
- cargo fmt --check --all and strict cargo clippy --workspace --all-targets -- -D warnings passed9.76s.
- Full contracts:check passed exit0: typecheck,274Rust/headless tests zero failed/ignored,16bridge files399tests. Complete unfiltered Rust workspace passed exit0: 1,019 top-level tests passed, zero failed, nine declared ignored, zero filtered across 39 runs; desktop22/core947/headless50, including long GC and the readiness regression. Log final-rust-workspace.log and status file preserve completion.
- Pinned change/all strict proposal validation passed41items0failed; prearchive protected active-change rejection is expected and is not a gate success.
- Independent reviewer review49 explicitly granted final global approval after inspecting the sole test diff, all four focused positive/negative observations, fmt/Clippy, full contracts and full workspace evidence. Root independently concurred and authorized verify/sync/archive on2026-10-05; no findings remain.

## Unchanged evidence reuse

Root AGENTS.md line31 permits reuse only while relevant inputs/toolchain/environment remain unchanged. This test-only correction changes no production code, catalog, renderer, media, fonts, references, TS/Python source or configuration. The verified9de55a93 issue49 evidence remains valid for unchanged full native golden/rules/linear/lifecycle/cache rendering, bridge type/lint/unit/integration/smoke and hermetic Python workers; their exact commands/counts/logs remain in the existing archived issue49 verification. Affected headless protocol/native readiness, full contracts, Rust formatting/strict Clippy/unfiltered workspace and OpenSpec policy are explicitly rerun here. Reused renderer evidence is distinct from the new executed native readiness proof.

Verification confirms one modified requirement and all four scenarios covered, complete implementation/design coherence and no critical, warning or suggestion findings. The modified readiness requirement was synchronized without removing either original scenario and the completed change archived at openspec/changes/archive/2026-10-05-fix-linear-capability-contract-readiness/. All eight tasks are complete. Final unchanged moon run root:openspec-validate passed:377policy tests, zero failed, one Moon task completed and40strict specs passed. Independent pinned all-spec strict validation also passed40items, zero failed. Logs final-protected-moon.log and final-all-strict.log preserve exact evidence; git diff --check passed. All verification dimensions are complete with no findings. No commit/push/branch/restack action is performed by this implementation agent; designated human CODEOWNER review and remote CI remain parent-managed.

## Remote evidence limitation

The first remote PR142 run completed with Contract parity and all three Rust correctness jobs failing; rendering, rules-screen, packaged smoke and OpenSpec jobs passed. Local controlled missing-tools reproduction proves this readiness assumption defect, but remote job logs remain inaccessible despite the saved network configuration addition, so no claim is made that every remote failure detail is known. Root will publish this reviewed correction and inspect the new remote run. Designated human CODEOWNER review remains pending on the draft PR.
