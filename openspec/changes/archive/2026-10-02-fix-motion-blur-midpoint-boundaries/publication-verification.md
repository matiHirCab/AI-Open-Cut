# Published midpoint correction conformance

This record covers the implementation published in PR #133 at
0dc3802c04888da197552b0d272a8c5cdb0160f4. Its tree equals recovered correction
head 199d22f45a2fcf5143e37a1ebf5007224e50014b. The three original correction
commits and main's PR #132 merge remain ancestors; no history was rewritten.

Fresh user implementation approval for that exact head and authorization for
specification synchronization, archival and normal publication are recorded in
approval.md. No GitHub review submission, PR merge, deployment or issue #45
implementation is claimed.

## Verification before synchronization

The repository openspec-verify-change workflow was applied using pinned OpenSpec
1.5.0 status/apply context, all proposal/design/delta/tasks artifacts, the living
requirements, owned implementation and scenario tests.

| Dimension | Result before archival |
| --- | --- |
| Completeness | 7/8 tasks complete; implementation, verification, recovery and fresh owner approval complete; task 7 is this synchronization/archival/final-gate workflow |
| Correctness | The modified canonical shutter interval requirement and all three scenarios have automated coverage |
| Coherence | Exact checked binary64 arithmetic stays in editor-core; inherited clocks, root clipping, equal weights, schema 28, protocol 1, validation, limits, history and audio contracts remain intact |

Requirement/scenario traceability:

- **Sample inherited boundaries:** the existing native inherited shutter,
  nested repeater/stagger/loop/effect and animation-channel suites exercise
  complete inherited transforms, held boundaries and shared render intents.
- **Clamp the timeline boundaries:** canonical_midpoints_and_boundary_clamps,
  integer_boundary_midpoints_match_every_allowed_count_and_angle_neighbor,
  fractional_midpoints_and_subnormal_angles_preserve_floor_and_clipping and
  shutter_offsets_preserve_adjacent_large_integer_times cover clipping,
  duplicate weights, representable neighbors, subnormals and large roots.
- **Floor exact midpoint boundaries for every allowed count:** the same
  arithmetic tests, 25 canonical constant vectors, 70,123 independent Python
  Fraction cases against the compiled production sampler, and
  native_five_midpoint_held_parent_matches_independent_frame_range_draft_export
  cover the exact ratio and independent held-parent pixel/audio oracle.

The 500 ms / 25 FPS / 360 degree / five-sample result is
[484, 492, 500, 508, 516]. On actual FFmpeg 6.1.1 and 7.1.5, frame/draft/range/export
MSE is 0 and SSIM is 1; range/export PCM RMS is 0. The native assertions also
preserve project/history bytes. Existing tolerances and reference fixtures are
unchanged.

## Integrated implementation evidence

Checks were executed in the dedicated environment on the approved published
implementation, using Rust 1.97.0, Bun 1.4.0, Moon 2.3.3 and OpenSpec 1.5.0.
The reviewed DejaVuSans fixture hash is unchanged. Full logs are external,
uncommitted, under /workspace/opencut-evidence/.

| Check | Actual result |
| --- | --- |
| Independent Fraction arithmetic / canonical constants | PASS: 70,123 cases / 25 vectors |
| Rust formatting / workspace strict Clippy | PASS, exit 0 each |
| Rust workspace | PASS, exit 0; 786 executions including child harnesses, nine ignored maintenance entries |
| Required native FFmpeg 6 and FFmpeg 7 | PASS, each 58 animation tests plus 12 focused motion-blur tests |
| Bridge typecheck / lint | PASS, exit 0 each |
| Contract parity | PASS, 191 Rust + 360 TypeScript tests |
| Bridge unit | PASS, 427 tests; one native opt-in skip exercised separately |
| Fake-provider MCP integration | PASS, 14 tests |
| Packaged mocked-provider smoke | PASS, eight tests; not native parity evidence |
| Hermetic Python | PASS, 10 unittest + five pytest tests |
| Native headless lifecycle / geometry / fonts | PASS, one selected lifecycle + 13 geometry + 16 font tests |
| Core cache / feature worker / native bridge cache | PASS, 13 / three / one tests; feature build passed |
| Restored default headless build and tests | PASS, 41 tests |
| Policy regression | PASS, 377 tests / 1848 assertions |
| Strict pre-archive specification validation | PASS, 36 items |
| Protected pre-archive Moon and isolated bootstrap | Expected exit 1 naming only this active change; not final passes |

Retained setup failures were resolved without code, assertion, threshold, golden
or policy changes: native FFmpeg 6's encoder fixture initially lacked rustc on
PATH; the unchanged complete suite passed after fixing the local tool path.
One bridge descendant assertion saw a container zombie; the child-reaping wrapper
rerun passed. Moon initially lacked writable local caches and an installed pinned
Bun toolchain; workspace-local setup now reaches the expected active-change
rejection. No credential or security settings were changed.

The 37 tracked legacy golden/policy input hashes match the recovery record.
Executable, fixture, dependency, workflow, golden and tolerance inputs are
unchanged by this archival work, so the passing implementation evidence remains
applicable under AGENTS.md's evidence-reuse rules. Specification and protected
checks are rerun after synchronization/archival.

## Terminal pre-archive remote CI

[Run 36993221753](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/36993221753)
finished with nine successful jobs and two failed jobs. Actual failed logs were
inspected: OpenSpec rejected only fix-motion-blur-midpoint-boundaries remaining
active, and foundation parity rejected the missing OpenSpec policy attestation.
Foundation's protected duration-audit step passed.

Contract parity, correctness on Linux/macOS/Windows, packaged integration/smoke,
native render parity and all three rules-screen resolutions passed. This includes
fresh remote golden/render/cache and rules-screen execution; those results are
distinct from historical recovery-package corpus reuse. The pre-archive run is
an overall failure, not an overall CI pass.

## Synchronization and archival outcome

The approved exact-ratio sentence and boundary scenario are synchronized into
the existing living canonical shutter interval requirement. Its existing two
scenarios and every other requirement are preserved. The correction is archived
at openspec/changes/archive/2026-10-02-fix-motion-blur-midpoint-boundaries with
.openspec.yaml preserved. All eight tasks are complete.

Post-archive strict validation passes all 35 living specifications. Protected
Moon root:openspec-validate, the isolated CI policy bootstrap and Rust formatting
all pass with exit 0. The former active-change rejection is cleared without
altering policy, thresholds, fixtures, executable code or implementation inputs.

The archival update still requires normal publication and exact-head remote CI.
Its published commit and terminal CI outcome are recorded externally in PR #133
after execution; no future result is claimed here.
