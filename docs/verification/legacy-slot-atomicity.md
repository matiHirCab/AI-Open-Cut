# Legacy slot-edit atomicity verification

Baseline: `c4a9625cc56e6c1476b0fc64512852259f3f0b58` (main, PR222 included). Follow-up to issue #180; no new public contract or schema version.

## Reproduction and correction

The regression fixture writes a genuine schema44 component with empty graphic-slot definitions, an existing instance, and schema44 retained undo/redo directly to project/history files. It captures both authoritative byte streams before submitting an edit and performs no migrating read before rejection.

Before the fix, all three rejection regressions fail their byte-equality assertion despite returning ITEM_NOT_FOUND: a missing local binding target, an unknown instance slot and a late-failing batch whose earlier instance update is valid. The existing adoption test and valid-edit test pass (2 passes, 3 failures).

The production change classifies ComponentDefineSlots and ComponentInstanceUpdate for the existing staged composition path. That path applies the full candidate before the existing recoverable project/history/draft publication. It retains core-owned validation, the project lock, aliases, optimistic revision, tracked resources and journal commit semantics.

## Focused evidence

- `cargo +1.97.0 test -p opencut-editor-core --test solid_graphic_color_slots`: 10 passed, 0 failed/ignored. Includes three exact-byte rejection regressions and all three valid routes with stale revision rejection, one revision increment, current/history adoption, exact undo/redo and stable reopen.
- `cargo +1.97.0 test -p opencut-editor-core --lib legacy_slot_edits_preserve_precommit_bytes_and_recover_complete_generations`: 1 passed, 0 failed/ignored. Exercises three routes at nine existing publication fault points (27 combinations). Before the commit point all authoritative bytes remain exact; after it reopening recovers one complete edited generation at revision2 and is repeatable without managed transaction artifacts.
- Rust format and strict workspace/all-target Clippy pass. Full Windows workspace exits0 with1561 primary passes and12 existing ignores. The initial default-symbol build exhausted disk; its failure log is retained. A fresh full repeat uses `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=2`; only build symbols/cache/concurrency change, with all original assertions and timeouts retained.
- Bridge typecheck/lint pass. The correctly configured serial full unit repeat passes931 cases with17 existing environment guards; full contracts passes627/627 with no omissions. The initial missing-sidecar/concurrent-deadline attempt remains retained as a failure. Windows integration and packaged smoke execute40/42 and37/39 respectively; each has the same two failures from intentionally Unix `#!/bin/sh` controlled-provider fixtures. Those fixture helpers explicitly state Unix runners and separate kernel startup evidence. They are not accepted as passing checks. A separate Ubuntu WSL checkout has identical scoped source/test SHA256 and runs the complete unchanged suites with private Bun1.4.0, Node24.16.0 and Rust1.97.0; integration passes42/42 and the additional regression/fault suites pass10 tests plus27 fault combinations.
- Hermetic Python workers pass 12 unittest +12 pytest; agent-context checks pass18/18.
- Pinned strict OpenSpec passes75/0 before archive. Protected prearchive Moon rejects solely the named active change, as required; this is not final gate success.

## Packaged verification and local compiler limitation

The original full39 packaged smoke cases pass in Ubuntu WSL. The bridge uses the supported pinned Bun1.4.0 Windows-to-Linux cross-build (`bun build apps/agent-bridge/src/index.ts --compile --target=bun-linux-x64`); the headless is the native Rust1.97.0 release build from the same scoped patch. The repository's unchanged assembleRuntimePackage and verifyRuntimePackage validate the package before the unchanged vitest.smoke.config.ts suite runs. No fixture, assertion, timeout or CI policy changes.

The ordinary native WSL bootstrap remains a failed attempt: Bun1.4.0 emits `Error truncating ELF file: EACCES ... ftruncate()`. One-line independent probes reproduce it with baseline/standard official Bun, DrvFS and owned ext4, including identical Bun bytes copied onto ext4. This is an installed compiler limitation, not a successful native build or a claim about the application. Its ordinary exact-head CI build remains required for completion.

## Remaining delivery gates

Stable-tree prearchive validation passes75/0, protected prearchive rejects solely this active change, and all7 tasks plus both scenarios were verified. The requirement is synchronized and the scoped change archived as 2026-10-15-preserve-legacy-slot-edit-atomicity. Final protected Moon passes (exit0), strict all-spec validation passes74/0, and git diff --check passes. Commit/draft publication and exact-published-head CI remain pending. This document does not declare delivery complete.

## Reference baseline CI

GitHub metadata confirms main c4a9625c run38808182141 failed Correctness(windows-latest). The parent supplied five existing5000ms unit timeouts (doctor-cli, two job-restart cases, known-text-alignment and masked-hero-reveal),926 passes/17 guards and an approved52.41/120min duration. Local serial Windows unit verification passes all931 executed cases with the original timeouts. No unrelated workflow/timeout change is included. [Reference job](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/38808182141/job/116674383522).
