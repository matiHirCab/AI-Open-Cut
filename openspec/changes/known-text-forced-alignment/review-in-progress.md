# Issue61 delegated source review and pending standard acceptance

Reviewer: the implementation Codex agent, performing a separate substantive review pass under the user's explicit CODEOWNER-review delegation (2026-10-07: “I allow you to give codeowner review”). This is not a distinct human reviewer, GitHub APPROVED review, or authorization to merge.

Reviewed the actual core validator and extracted existing source resolver; typed Rust request/dispatch and TypeScript union; provider-neutral status/input/preview shapes; Python known-token/audio alignment path against the pinned faster-whisper1.2.0 source; shared queue and cancellation lifecycle; owned preview snapshot and pre/post revision checks; canonical consumers and all predecessor projections. Source hashes are in reviewed-contract-source-sha256.json. No renderer, persisted schema, migration or unrelated implementation was changed.

Implementation findings reconsidered in this separate pass:

- The core uses one loaded project snapshot and the existing revision helper, not two independently loaded snapshots or a new error classification. Alignment remains bounded by the owning asset duration. Existing asset/probe compatibility mismatch is rejected by the earlier integrity check.
- Provider result is structurally snapshotted before retention, and both service-close/cancellation checks run after the asynchronous final core validation. Thus delayed validation cannot publish a late token.
- A formatter changed an initially captured JSON predecessor file. The unchanged raw-hash assertion caught it; capture bytes were restored from independently pinned e2985f09 and stored as .raw. Hash expectations remain unchanged.
- New negative core fixtures initially changed compatibility fields without their matching probe fields. The full contracts gate correctly failed with ASSET_INTEGRITY_FAILED. Fixtures now consistently represent non-audio/unknown-duration sources, and a separate mismatched-probe case retains that exact earlier integrity failure. No production integrity rule or assertion was weakened.
- Read-only validation reuses existing ordinary project-open migration/recovery. The requirement wording was clarified to match the already approved design/approval; no new migration or mutation is introduced.

Passing local evidence: Rust formatting/strict workspace Clippy; workspace1366 tests with9 existing ignored; bridge typecheck/lint/unit646 tests with9 existing skips; complete contracts500 TypeScript cases plus all required Rust consumers; Python12 unittest/12 pytest; protected policy457 controls and strict47 specification items. The pre-archive protected gate rejects only this active change, as expected.

Complete integration24 tests passed in a diagnostic Node2048/exposed-GC runtime. Packaged21 tests passed with a diagnostic Node2048 heap limit. Original standard integration/packaged worker-loss failures and other failed diagnostic attempts remain in full external logs. cgroup OOM-kill counters increased. Those diagnostic passes are not standard CI acceptance. Stale task-generated Preset migration fixtures were preserved on workspace storage with original paths retained to relieve RAM-backed temporary storage; no evidence or test coverage was discarded.

Source conformance appears sound, but final CODEOWNER acceptance and OpenSpec verification/archival remain pending until unchanged standard external integration and packaged smoke pass. This draft is intentionally not complete or merge-ready. All11 exact-final-head CI remain mandatory before continuing to issue62.

Main reconciliation: the user merged PR155 as d040cdec5ad44825db6be4dce7eada7cf4e0c8be. That commit contains verified e2985f09 and has exactly its tree. This branch continues from verified e2985f09, preserving prior work. Its PR targets main and now shows only issue61 scope; future cumulative main-targeting PRs must merge after this PR. No merge/deploy/issue closure is authorized.
