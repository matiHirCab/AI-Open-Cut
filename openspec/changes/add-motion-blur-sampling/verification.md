# Verification: add-motion-blur-sampling

Status: implementation and required native regression checks pass locally.
Not archived, published or merge-ready. Designated implementation review remains
required. Historical failures and superseded evidence are preserved outside Git.

## Scope, approvals and dependency

The user's delegated approval covers necessary issue44 OpenSpec decisions only.
`approval.md` records initial planning, reconciliation/failure evidence, and the
reviewed deterministic inherited fallback amendment, approved before code edits.
No unrelated feature, security, account or CI-policy proposal was approved.

The original issue44 commits remain intact: `def74b6d` (approved plan),
`ec5f67d33a1a5eea581545bd7befc222b61c1a77` (implementation), `cd4903d7`
(historical verification), followed by `be25d2c3`, `695c5767`, `a1379379` and
`d5540cca` (bounded reconciliation, rejection fixtures and deterministic fallback).
The local branch is `feat/issue-44-motion-blur`, at implementation checkpoint
`5aa6da8f0aba78587d868521f3d655dd54021cb7` after main reconciliation.

Initially this work stacked on open PR131 at
`e1010bb97174d44a5d19c61a90514bf2336845ca`; main was `08a543f6`.
The separately authorized issue43 correction was published to its existing branch
at `81a6ec682bd65a3b31d84b4d2aee7eef276c0bb4`. Its exact-head CI run
[36918625589](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/36918625589)
passed all 11 jobs, including all 56 rules-screen renders and policy attestation.
Foundation elapsed 49.03 minutes was within the unchanged 120-minute budget.
PR131 merged externally at 2026-10-01T20:56:03Z, producing main
`b2be8f219743639db106330f5839a2efcf77c053`. That commit and 81a6ec6 have
identical code trees `b376f5526eca1ac84ef66e7309a50e57503375a9`.
Issue44 contains both the corrected head and merged main without rewriting its
original commits. No agent merged PR131; issue44 remains unpublished.

No issue/CI/account/access settings, golden references or duration/timeout guards
were changed. New governed files retain the existing designated CODEOWNER. The existing readonly hourly watch is independent. Unrelated TMT
repositories were not used or modified.

## Conformance review

Four requirements and all 11 scenarios map to owned implementation and automated
fixtures. All required implementation checks pass on the stable code tree.
Completeness remains blocked by designated contract-owner review, archival and
the final protected gate. The protected pre-archive check and formal workflow
report are recorded below.

| Requirement / scenarios | Implementation and automated evidence |
| --- | --- |
| Typed bounded shutter settings: author/restore, reject without publication, deterministic disabled inherited fallback | `model/motion_blur.rs`, `model.rs`, `timeline.rs`, `validation/extended_visual.rs`; `canonical_numeric_and_closed_record_cases`, `shutter_alias_batch_lifecycle_and_failure_rollback`, `unsupported_controllers_and_hidden_sample_work_fail_before_publication`, native exact omitted/zero/single controls. Actual headless `rejected_shutter_records_preserve_standalone_and_batch_bytes` submits all 12 rejected canonical records standalone and after a valid batch edit (24 failures), checking non-retryable INVALID_ARGUMENT, positive stored settings, unchanged project/history bytes and reopened state. MCP standalone/batch null fixtures retain unchanged state. |
| Canonical shutter interval: inherited boundaries, project boundary clamping | Canonical sample catalog, `canonical_midpoints_and_boundary_clamps`, `shutter_offsets_preserve_adjacent_large_integer_times`, and `shutter_samples_cross_reflected_turns_and_finite_exhaustion_on_canonical_clocks`. Independent midpoint/parent-transform pixel oracle and native nested component/time-scale/stagger/signed-repeater/loop fixture. Integer-root flooring preserves roots above 2^53 and inherited fractional clocks. |
| Shared bounded temporal composition: inherited ordering, work/output failure, audio/compatibility | `evaluated_scene/extended_visual.rs`, `extended_certification.rs`, `render_artifact/extended_visual.rs`, `render_plan.rs`; per-leaf premultiplied linear-light canvas average after each sample's crop/clip/paint/effects/full affine/transition opacity. Independent analytic 462/487/512/537 ms samples keep MSE<=1.0; exact-frame SSIM>=0.99, aligned decoded PCM RMS<=0.0001 and timing<=one frame. Draft equality, different authored/output FPS, center-inactive shutter tails, nested rotation/effects and native encoder failure/cleanup fixtures cover the same renderer semantics. Hidden/excess work fixture accepts the exact pixel limit and rejects the next weighted leaf without byte publication. |
| Atomic schema and governed contracts: retained adoption, malformed migration, discovery/exercise | Schema28 current plus nonempty undo/redo migration; premature current/history/draft fields fail closed without writes; compatible drafts and deterministic second reopen preserve bytes. `migration_adopts_complete_history_and_rejects_premature_fields_atomically`, `legacy_draft_fields_fail_before_adoption_and_compatible_drafts_reopen_deterministically`, existing publication-fault/lifecycle suites, canonical Rust/Zod/headless/MCP catalogs, capability/version reporting, structural digest and standalone/batch aliases. CODEOWNER review is pending and required before archival. |

Coherence: canonical semantics and checked arithmetic stay in editor-core;
transports carry typed additive fields. No new dependency edge, RNG, raw
expression input, executable SVG, path-bearing setting or network resource was
introduced. Existing audio rules and layer stacking remain unchanged.

The disabled fallback fix extends the existing private command-thread guard only
for unsampled finalized affines with animated ancestors. Those inherited st/ld
expressions share mutable registers across FFmpeg blend slices. Captured omitted
and zero controls had identical graphs/inputs but 48 differing pixels. Diagnostic
serialization demonstrated the cause; final certification uses the unmodified
actual executable. Plain root and sampled branch selection and existing Bezier
conditions remain unchanged. The shared expression/slice behavior is visible in
[FFmpeg n6.1.1 blend source](https://github.com/FFmpeg/FFmpeg/blob/n6.1.1/libavfilter/vf_blend.c);
runtime captures and serial replay identify the inherited-register race.
The independent oracle adds the renderer's unchanged
final YUV420 conversion after explicit RGBA stacking, with every numeric coverage
expectation and tolerance retained. See the approved design amendment.

## Final completed checks

Toolchains: Rust 1.97.0 (pinned), additional strict Rust 1.98.0 Clippy, Bun 1.4.0,
Moon 2.3.3. Required native runs use official Ubuntu FFmpeg/FFprobe
6.1.1-3ubuntu5, with a second animation run using Debian 7.1.5. The reviewed
repository DejaVu Sans SHA256 is
`ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280`.
Actual tools/font paths and required markers are explicit in retained runners;
OPENCUT_UPDATE_GOLDENS and OPENCUT_CAPTURE_GOLDENS_TO are unset.

| Command / run | Result |
| --- | --- |
| `cargo fmt --check --all` | PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS on Rust 1.97 and 1.98, exit 0 each; existing proc-macro-error2 dependency emits Cargo's future-compatibility notice, not a project lint failure |
| `cargo test --workspace` | PASS on Rust 1.97; 780 successful test executions including 9 isolated helper invocations; all root suites pass, 9 ignored helper/maintenance entries. Optional native early returns here are not counted as actual render parity. |
| Bridge `bun run typecheck` / `bun run lint` | PASS / PASS |
| Bridge `bun run contracts:check` | PASS, 185 Rust fixture tests + 360 TS tests |
| Bridge `bun run test:unit` | PASS, 427 tests; one opt-in native test skipped here and actually executed separately |
| Bridge `bun run test:integration` | PASS, 14 tests, 170.21 s idle rerun; 60 s timeout unchanged |
| Bridge `bun run test:smoke` | PASS, 8 packaged tests, 51.84 s; mocked FFmpeg, packaging evidence only |
| Bridge `bun run scripts/run-python-tests.ts` | PASS, 10 unittest + 5 pytest |
| Required native `cargo test -p opencut-editor-core --test animation_channels -- --nocapture` | PASS on actual FFmpeg 6 and 7: 55/55 each, 80.77 s / 86.11 s; includes all 9 issue44 fixtures with required marker |
| Unchanged native seven-corpus golden conformance (`--release`) | PASS, 3042.13 s under four concurrent native runs; all references/tolerances unchanged |
| Strict captured-report validator (`--ignored --exact`) | PASS, explicitly executed; schema 3 baseline captured from implementation checkpoint 5aa6da8f |
| Required core raster cache: `cargo test -p opencut-editor-core --lib raster_cach` | PASS, 13 tests, 21.56 s, actual cold/warm/fresh parity |
| Feature-enabled worker / build / actual bridge reuse | PASS, 3 Rust worker tests + 1 native bridge test; actual 6.1 rendering and required hook markers |
| Default headless restoration / `cargo test -p opencut-headless` | PASS, default build restored; 5 + 34 + 2 tests |
| Required actual headless lifecycle | PASS, 1 selected native test; other tests intentionally filtered |
| Actual JSON headless shutter reproduction | PASS, 22 requests: aliased authoring, standalone edits, native frame/range/export/draft, canonical retryable revision conflict, undo/redo/reopen and byte preservation. Independent MSE 0.0; SSIM range 0.999710 / export 0.999772; unchanged PCM RMS 0.0 for both; durations 0.3 s / 1.0 s exactly. |
| Native `--test transform2d --test font_resolution` | PASS, 13 + 16 tests with explicit reviewed font and real tools |
| Workflow-policy regression (documented 4 scripts) | PASS, 377 tests / 1848 assertions, including real-Moon bootstrap regression |
| Pinned OpenSpec strict all validation | PASS, 35 items at implementation checkpoint; final documentation rerun recorded below |
| Native PR rules-screen 960x540 / 1280x720 / 1920x1080 | PASS, exit 0 each; 25 / 25 / 6 actual renders, all five semantic/lifecycle states at each resolution; unchanged SSIM, PCM and timing assertions |

Full logs are outside Git. Cargo project artifacts were rebuilt from this checkout;
canonical positive stored-setting checks prove the actual schema 28 binary was used.
Linux PID1 retains orphan zombies, so workspace/bridge process tests use a local
child-subreaper that reaps only its own adopted test children and preserves exit
status. It changes no assertion or repository process-containment policy. Native
library linking uses temporary symlinks to installed libraries; no global or
security configuration changed.

## Final native matrix

The complete required PR matrix passes with actual FFmpeg 6.1.1-3ubuntu5 and the
reviewed font. Each test asserts exact render counters, all five lifecycle states,
unchanged project bytes, reviewed frame SSIM >=0.99, aligned PCM RMS <=0.0001 and
timing within one frame. The 1920 PR shard renders original/edited and still checks
undo/redo/reopen semantics; the unchanged weekly full-render scope is separate.

| Resolution | Renders | Resolution elapsed | Peak own process-tree resident bytes |
| --- | ---: | ---: | ---: |
| 960x540 | 25 | 6419.909 s | 853028864 |
| 1280x720 | 25 | 7694.084 s | 1208049664 |
| 1920x1080 | 6 | 5315.050 s | 2126905344 |

These are manual Linux observations under a shared four-CPU quota, with the three
resolution runs initially concurrent with the golden corpus. They are not issue44
remote CI elapsed times or a policy attestation. Existing native graphs in these
matrix runs retain their ordinary thread selection. No reference, threshold,
workflow, job timeout or protected budget changed. The prerequisite's separate
exact-head CI passed within its 120-minute protected budget. `final-native-matrix.json`
and full per-resolution logs retain counters, state timings and exit statuses.

## Protected gate and workflow report

All implementation checks are complete. Final strict all-spec validation passes
35 items. `moon run root:openspec-validate` runs normalization, 377 passing policy
tests / 1848 assertions and all 35 strict specs, then exits 1 solely because
`add-motion-blur-sampling` is unarchived. The isolated
`bun --config=/dev/null --no-env-file run scripts/run-ci-policy.ts` also exits 1
solely for that active change, before launching Moon or writing an attestation.
No other policy or specification error is present. Full logs are
`handoff-protected-moon.log` and `handoff-protected-bootstrap.log` outside Git.
This expected pre-archive rejection is not a passed protected gate.

The repository's `openspec-verify-change` workflow was applied to this named
change: pinned CLI status and apply context were loaded, all four artifacts read,
and requirements, scenarios, design, tasks and owned code reviewed. CLI
`isComplete: true` describes planning artifacts only; implementation workflow
completion is governed by the task checkboxes and protected gate.

| Dimension | Assessment |
| --- | --- |
| Completeness | 19/21 tasks complete; all four requirements implemented; two workflow tasks blocked |
| Correctness | 4/4 requirements and 11/11 scenarios have implementation/test evidence in the mapping above; required native and transport checks pass |
| Coherence | Approved sampling, inherited evaluation, composition, limits, migration and private fallback decisions followed; canonical ownership preserved |

CRITICAL before archival:

1. Task 1.2a: obtain designated `@matiHirCab` contract-owner implementation review
   required by AGENTS.md:50, the ownership catalog and the approved delta. Human
   implementation approval cannot be substituted by an automated fixture or the
   agent's delegated specification approval.
2. Task 5.4: after that review, synchronize and archive this verified change with
   the repository workflows, then rerun the protected Moon gate, isolated policy
   bootstrap and strict all-spec validation. Both protected pre-archive checks
   currently reject the active change as described above.

No implementation/spec/design mismatch or uncovered implementation scenario was
found. No additional warning or suggestion is recorded. The change is ready for
designated implementation review, and remains blocked for archival and merge
readiness. Post-archive validation has not been run because archival is blocked.

## Failures, skips and unavailable evidence

- Preserved superseded failures: original unsupported issue43/issue44 Ubuntu6
  conformance; corrected prerequisite then 54/55 issue44 native tests with exact
  disabled-control failure; diagnostic oracle MSE 1.7252604166666667 before the
  unchanged final YUV conversion was modeled. Approved fixes now pass focused
  and complete 55-test actual 6/7 animation runs without changed tolerances/references.
- Final bridge first attempt had 13 passes and one 60 s ungroup/component timeout
  while workspace tests/compilers were active. The complete idle rerun passes 14;
  failed log remains `superseded-loaded-integration.log`. Smoke/Python were not
  reached by that fail-fast attempt and then both ran and passed.
- An additional outside-Git headless reproduction initially assumed revision
  conflicts were non-retryable. The canonical error catalog declares them retryable;
  the runner now reads that catalog and its complete fresh rerun passes. Production
  behavior was correct and unchanged. The failed journal/artifacts remain preserved.
- Historical shared Cargo output reused a prerequisite schema 27 binary; the new
  rejected-record fixture exposed it. Cleaning only generated core/headless output
  and rebuilding fixed tooling state. The positive control proves stored typed
  settings before rejection tests. Superseded assertion/lint drafts are retained.
- Ordinary unit/workspace opt-ins/ignored maintenance helpers are listed above;
  required native gates execute separately. Reference regeneration/recapture is
  intentionally not run. Weekly full 1920x1080 scope is not the required PR shard
  and is not run. No newly published issue44 remote CI or Windows/macOS issue44
  execution is claimed.
- AGENTS.md:50 and the delta require designated CODEOWNER implementation review.
  Delegated spec approval does not satisfy that review. Archival/living-spec sync
  and post-archive protected validation remain blocked pending that review;
  all required implementation checks already pass. The active change must not be
  hidden or prematurely archived to obtain a policy attestation.

## Performance and limits

Canonical caps: 16 samples and 268435456 sample-weighted canvas pixel units per
output scene frame, retaining stricter existing effect/geometry/occurrence/surface
limits. Exact-boundary and hidden-content rejection tests pass before publication.
A focused actual 6.1 64x64, 10 fps, 1 s export measured 4 samples at 428.479 ms and
one sample through the same raster/FFV1 pipeline at 361.522 ms. Full-suite loaded
observations are 508.559/631.253 ms (6.1) and 440.686/626.555 ms (7.1).
These are report-only local observations; contention makes them unsuitable as
universal latency or speedup claims. The native baseline is strict schema 3,
records three measured benchmark samples and 80506880 peak resident bytes
(approximately 76.8 MiB) during those benchmark observations only. It does not
measure peak memory across the complete corpus. The complete seven-corpus run
took 3042.13 seconds under four concurrent native runs. Complete matrix timings
and peak process-tree observations are listed above; no budget or concurrency
policy changes were made.

## Deliverables and next review

Local branch and focused review commits are retained in this isolated checkout.
Full logs, historical evidence, reproduction runners, native baseline, final patch
and SHA256 manifest live outside Git at
`/workspace/AI-Open-Cut-evidence/issue44-20261001/`.
Actual transport artifacts, report and request journal are in
`headless-native/`, including `shutter-frame.png`, `shutter-range.mp4` and
`exports/shutter.mp4`; their analytic oracle is computed independently of output.
`proposed-pr.md` is proposed text only. Obtain designated implementation review,
finish synchronization/archival and the unchanged protected final gate, then obtain
separate user publication approval. No issue44 branch or PR has been published.
