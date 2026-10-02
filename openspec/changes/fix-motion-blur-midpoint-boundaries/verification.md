# Exact-midpoint correction verification

The independent P2 finding is fixed. Implementation checks pass; fresh designated
CODEOWNER review and correction archival remain pending. The original PR132
archival and human approval records are preserved byte-for-byte. This report does
not claim merge readiness, final protected success or corrective-head remote CI.

## Base, approval and preservation

PR132 is a draft on codex/issue-44-motion-blur. Its inspected/fetched head is
b03ca51dcf1f1bf00bb8f67f9bf45504363dc9cc. It archives the implementation at
28a1ecf37355bcb171afe2cdd26fce7fb771ffac with explicit human CODEOWNER approval;
its production sampler still contains the reported numerical defect. The sampler
at 28 and b03 is byte-identical (SHA256
52c87816c3efce26aff203885a431b1b92dca934388f0591d8469a5792012a6e).

The correction branch fix/issue-44-exact-midpoints starts directly at b03, preserving
all nine original implementation commits and the user-side archival commit.
Plan approval 6d6985cf precedes implementation 6cf4d93dec673272ef7059e3bb9c5402178f62de.
The earlier private d37eec8e planning commit remains on its original local branch;
no rebase, force push, merge, dependency, golden, policy or credential change.
Issue43 PR131 merged externally to main b2be8f219743639db106330f5839a2efcf77c053,
containing corrected 81a6ec6; its original pinned head was e1010bb9. That dependency
is satisfied. The independent readonly watch and unrelated TMT repos are untouched.

## Conformance workflow

Applied repository openspec-verify-change: pinned 1.5.0 status/apply context,
proposal/design/delta/tasks, living requirement, owned code and every scenario
were read and checked. Planning isComplete describes artifacts, not gate success.

| Dimension | Evidence |
| --- | --- |
| Completeness | 6/8 tasks complete; implementation and recovery preparation done; tasks 6/7 require fresh owner review and post-review archival/final gates |
| Correctness | Modified canonical shutter interval requirement and all 3 scenarios covered; exact-boundary scenario adds 3040 independent parameter combinations plus 25 canonical sample cases and actual held-parent frame/range/draft/export proof |
| Coherence | Checked binary64 integer ratios in editor-core, unchanged inherited clocks/root clipping/equal weights; existing rendering/limits/audio/schema/revisions/batches/history/error contracts retained |

Code: model/motion_blur.rs. Tests: canonical_midpoints_and_boundary_clamps,
integer_boundary_midpoints_match_every_allowed_count_and_angle_neighbor,
fractional_midpoints_and_subnormal_angles_preserve_floor_and_clipping,
shutter_offsets_preserve_adjacent_large_integer_times and
native_five_midpoint_held_parent_matches_independent_frame_range_draft_export.
The canonical catalog remains motion-blur-sampling-v1/schema 28/protocol 1, with
additive timing metadata and constant fixtures. No acceptance bound changes.
Existing migration/reopen/rollback/standalone/batch/alias and native inherited
component/parent/repeater/stagger/loop/crop/clip/effect/audio tests were rerun.

## Independent before/after evidence

Before code edits, the five-sample CPU assertion failed with
[484,492,500,507,516] rather than [484,492,500,508,516]. Validated test setup then
ran the same fixed analytic 3/5:2/5 held-parent reference on real backends. The
reference never derives its weights from the production sampler. Export selects
the actual 25-FPS 480-ms frame and equivalent 488-ms held boundary; it does not call
that frame 500 ms. No epsilon, tolerance or reference change.

| Backend / mode | Before MSE / SSIM | After MSE / SSIM |
| --- | --- | --- |
| FFmpeg6 frame and draft | 81.28125 / .983435 | 0 / 1 |
| FFmpeg6 range | 79.703125 / .992001 | 0 / 1 |
| FFmpeg6 export | 87.90625 / .991753 | 0 / 1 |
| FFmpeg7 frame and draft | 81.28125 / .983435 | 0 / 1 |
| FFmpeg7 range | 81.28125 / .991940 | 0 / 1 |
| FFmpeg7 export | 87.90625 / .991753 | 0 / 1 |

Both phases keep range/export decoded PCM RMS 0. Range duration .080s, export 1 s,
within one 25-FPS frame. Rendering/drafts preserve project/history bytes. The final
reconciled 58-test suites repeat the same zero-error oracle on both backends.
Thresholds remain MSE<=1,SSIM>=.99,PCM RMS<=.0001,timing<=one frame.

## Checks executed on the reconciled implementation

Rust 1.97.0 pinned, additional 1.98.0 strict Clippy, Bun 1.4.0, Moon 2.3.3,
OpenSpec 1.5.0. Actual official Ubuntu FFmpeg/FFprobe 6.1.1-3ubuntu5 and system 7.1.5;
reviewed DejaVuSans SHA256 ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280.
Native required markers are set and golden update/capture flags are unset.
Full logs are external at /workspace/AI-Open-Cut-evidence/issue44-midpoint-correction-20261002/.
Their prefix is opencut-44-midpoint-reconciled-; exact commands are in copied runners.

| Check | Actual result |
| --- | --- |
| cargo fmt --check --all | PASS exit 0 |
| cargo clippy --workspace --all-targets -- -D warnings, Rust 1.97 | PASS exit 0 |
| cargo +1.98.0 clippy --workspace --all-targets -- -D warnings | PASS exit 0 |
| cargo test --workspace | PASS exit 0; 786 executions including 9 child-harness executions, 777 parent-suite passes, 9 ignored maintenance entries |
| Bridge typecheck / lint | PASS exit 0 each |
| bun run contracts:check | PASS exit 0; 191 Rust and 360 TypeScript checks |
| bun run test:unit | PASS exit 0; 427 pass, 1 native opt-in skip (executed separately below) |
| bun run test:integration | PASS exit 0; 14 tests, 176.12s; timeout unchanged |
| bun run test:smoke | PASS exit 0; 8 packaged cases with mocked FFmpeg; not native parity evidence |
| Hermetic Python runner | PASS exit 0; 10 unittest + 5 pytest |
| Required animation_channels, actual FFmpeg 6 | PASS exit 0; 58 tests, 95.06s |
| Required animation_channels, actual FFmpeg 7 | PASS exit 0; 58 tests, 87.95s |
| Native core raster-cache / feature worker / real bridge cache | PASS exit 0; 13/3/1 tests; feature build passed |
| Restored default headless build/tests | PASS exit 0; 41 tests |
| Selected actual headless lifecycle | PASS exit 0; 1 test, remaining deliberately filtered |
| Native geometry/font | PASS exit 0; 13+16 tests |
| Existing schema3 performance-report validator | PASS exit 0; 1 selected ignored test executed explicitly |
| Workflow-policy regression including actual Moon bootstrap | PASS exit 0; 377 tests/1848 assertions |
| Pinned strict all-spec validation | PASS exit 0; 36 items (35 living+correction) |
| Protected Moon / isolated bootstrap, pre-archive | Expected exit 1 solely for fix-motion-blur-midpoint-boundaries; not final passes |

Rust checks ran at implementation 6cf4d93d. This report/tasks/proposed-PR commit
changes documentation only; final strict/protected/diff checks are rerun after it.
Final immutable package metadata supplies its exact head/tree/checksum separately.
No passing code evidence is reused after an executable/fixture/dependency change.

## Reuse, failures, skips and performance

Reused, not rerun: historical seven-corpus native golden/performance and56-case
PR rules-screen matrix from28. All tracked golden sources/fixtures, render docs
and protected policy/workflow inputs were hash-compared unchanged (37 records in
unchanged-legacy-inputs.json). Their scenes author no motionBlur and default to
None; they cannot execute the changed enabled sampler. Tools/font/tolerances are
unchanged; the old schema3 report was revalidated on the corrected code. Required
affected motion-blur/native suites and all mandatory Rust/bridge gates were rerun.
This reuse is under AGENTS.md efficient-verification policy; it is not newly
executed final-head full-matrix or remote CI evidence.

Observed 64x64 / 10 FPS / 1 s export timings in concurrently running native fixtures:
four samples .546506s on6 and .426912s on7; sampled-pipeline control1.507858s/.630463s.
These are local observations, not speedups or protected-budget attestations.
Sample count stays<=16 and canonical pixel work<=268435456 alongside stricter
existing limits; limit/failure tests pass. No budget/concurrency/timeout relaxation.

Preserved failures: old CPU/native numerical assertions exit 101 before the fix;
initial native test setup lacked the existing parent legacy-transform selection
and was corrected before numerical capture. An intermediate old-branch driver
exited127 after passing fmt/Clippy/workspace because its script was edited while
running; an immutable driver reran required checks on the reconciled branch.
Dependency-file symlinks caused9 false Biome reachability diagnostics on unchanged
fixtures; materialized dependencies passed without source/rule edits. Native
bridge/smoke initial attempts failed ENOENT because the new worktree's debug/release
paths were absent; restored local artifact paths and passing reruns are recorded,
including a corrected smoke path command. An early policy run omitted Moon from
PATH and did not exercise its opt-in probe; the377/1848 rerun and real gates do.
No failure is hidden or converted to a pass without a successful rerun.

Not run: post-archive protected gates (fresh owner review pending); corrected-head
remote CI/publication. PR132 baseline run 36950285139 is separate: latest snapshot
has 8 successful jobs, 2 rules-screen jobs still running. This is not overall CI
success and does not test the new regression or fix. No credential changes,
GitHub approval review, merge or deployment. Existing blocked Git transport is not
bypassed; recovery contains only missing commits beyond b03.

## Remaining review

CRITICAL: task 6 fresh designated @matiHirCab implementation review required by
AGENTS.md:50 and the living contract before task 7 synchronization/archival.
The old human approval remains valid for its explicitly named 28 head, not this
new correction. After review, use repository sync/archive workflows and rerun the
protected Moon/isolated bootstrap/strict gates. No implementation/spec/design
mismatch or uncovered scenario remains. Private Library delivery is reported by
its confirmed receipt outside this immutable source record.
