# Verification: add-motion-blur-sampling

Status: implemented locally, blocked; not complete, archived, published or merge-ready.

## Scope and decision

The approved proposal/design/delta/tasks and `approval.md` record bounded delegated
issue44 specification approval. Designated CODEOWNER implementation review is pending.
The implementation is stacked on unmerged PR131 at pinned
`e1010bb97174d44a5d19c61a90514bf2336845ca`, not main. Planning commit:
`def74b6d`; implementation commit: `ec5f67d33a1a5eea581545bd7befc222b61c1a77`.
The implementation commit changes 39 files, 2108 insertions and 72 deletions.
No upstream branch was changed, pushed or merged. No issue/CI settings or hourly
watch were changed. Unrelated TMT repositories were not used or modified.

## OpenSpec verification scorecard

| Dimension | Assessment |
| --- | --- |
| Completeness | 15/18 tasks complete; 1.2a, 5.3 and 5.4 remain incomplete |
| Correctness | All four requirements have implementations and automated fixtures; supported-backend render conformance remains failed |
| Coherence | Canonical rules remain in editor-core; all consumers use shared EvaluatedScene preparation; no independent rendering approximation |

| Requirement | Implementation and scenario evidence |
| --- | --- |
| Typed bounded shutter settings | `model/motion_blur.rs`, `model.rs`, `timeline.rs`, `validation/extended_visual.rs`; canonical closed/numeric fixture, aliases, revisions, atomic batch rollback, undo/redo, unsupported controllers and hidden-work rejection |
| Deterministic shutter interval | Canonical midpoint/boundary catalog; adjacent roots above 2^53; reflected turns, finite exhaustion and fractional-clock unit oracle; independent parent-translation pixel oracle; nested component, stagger, repeater and animated-effect native fixture |
| Shared bounded temporal composition | `evaluated_scene/extended_visual.rs`, `extended_certification.rs`, `render_artifact/extended_visual.rs`, `render_plan.rs`; premultiplied linear-light per-leaf average, sampled activity, frame/range/draft/export and PCM checks; actual sampled-encoder fault/cleanup test; native 6.1 conformance failed |
| Atomic schema and governed contracts | Schema28 current/nonempty undo/redo adoption, premature current/history/draft failures preserving authoritative bytes, compatible drafts and byte-preserving reopen; capability/Zod/MCP/catalog/ownership fixtures and standalone/batch MCP workflows |

## Passed final checks

Pinned toolchain: Rust 1.97.0, Bun 1.4.0. Native successes use Debian FFmpeg/FFprobe
7.1.5 and the exact reviewed repository DejaVu Sans fixture (SHA256
`ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280`).

| Command | Result |
| --- | --- |
| `cargo fmt --check --all` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS on pinned Rust 1.97 |
| `cargo test --workspace` | PASS, 779 tests; optional native early returns are not counted as real parity evidence |
| Bridge `bun run typecheck` / `bun run lint` | PASS / PASS |
| Bridge `bun run test:unit` | PASS, 427 tests; one opt-in native worker test skipped here and executed separately below |
| Bridge `bun run contracts:check` | PASS, 184 Rust fixture tests and 360 bridge tests |
| Bridge `bun run test:integration` | PASS, all 14 tests, 171.83 seconds on the idle rerun; timeout remains 60 seconds |
| Bridge `bun run test:smoke` | PASS, all 8 tests; packaged smoke mocks FFmpeg and is not render parity evidence |
| Bridge focused contracts + motion-blur fixtures | PASS, 25 tests against the committed ownership catalog |
| `bun run scripts/run-python-tests.ts` in bridge | PASS, 10 unittest + 5 pytest cases |
| Required native `cargo test -p opencut-editor-core --test animation_channels` | PASS, all 55 tests with explicit real tools/font and required render marker; includes all nine motion-blur fixtures |
| Native golden conformance | PASS, 1381.35 seconds; rule-card, shapes, SVG, grids, repeaters, rich text and sampled capture; no goldens updated |
| Required native core raster cache | PASS, 13 tests including actual cold/warm/fresh conformance |
| Feature-enabled headless worker + native bridge reuse | PASS, 3 Rust tests + 1 actual bridge reuse test |
| Default headless rebuild and native tests | PASS; default build restored, 5 + 33 + 2 tests |
| Native transform2d/font resolution | PASS, 13 + 16 tests |
| Strict external performance-report validation | PASS, exact ignored validator executed |
| OpenSpec strict single-change / all validation | PASS / PASS, 34 total items |

Bridge unit/contracts/integration/smoke and native bridge reuse were run beneath an
external Linux child-reaper runner. This container's PID1 retains orphan zombies;
the unchanged base failed its worker-cancellation assertion for that reason, then
passed all 16 worker tests with the runner. The runner only reaps its own orphaned
test descendants; no assertions, repository settings or security permissions were
changed. Earlier integration timeouts under concurrent rendering disappeared on
the idle rerun. Formatting and native goldens initially exposed a wrong system
font selection; the reviewed repository font was used for successful native gates.
Desktop linking used temporary local symlinks to existing installed native libraries.

## Failed, blocked, cancelled and not reached

1. **CRITICAL: Ubuntu 6.1 parity.** Official Ubuntu `6.1.1-3ubuntu5` packages
   reproduce PR131's three failures on the unchanged pinned base: identity color
   drift MSE `124.97607421875`, nested SSIM `0.858022`, compound SSIM `0.830426`.
   Issue44's nine focused tests on this backend produce 7 passes and 2 failures:
   exact zero-shutter compatibility pixels differ, and nested frame/range SSIM is
   `0.860994`, below 0.99. The compatibility failure occurs before the independent
   oracle/range/export/audio portion of that fixture, so that portion is not
   certified on 6.1. Correct the base pipeline, then diagnose any remaining issue44
   failures and rerun required parity. Do not relax tolerances or change references.
2. **CRITICAL: base CI lint.** Extra Rust 1.98 core Clippy fails at unchanged
   `render_artifact/extended_visual.rs:55` (`chunks_exact_to_as_chunks`), matching
   remote PR131. The pinned 1.97 workspace check passes. The bounded corrective
   plan in `/workspace/AI-Open-Cut-issue43-fixes/openspec/changes/fix-extended-animation-ci-parity/`
   is strictly validated but awaits separate explicit approval; issue44 approval
   does not approve unrelated issue43 implementation. No corrective code or push
   has been made on that worktree.
3. **CRITICAL: incomplete rules-screen evidence.** Release-mode required PR scope
   at 960x540 completed the original state's five renders in 717.739 seconds.
   The sweep was deliberately interrupted during the edited state after the
   supported-backend blocker was confirmed; its process failed as a consequence
   of that interruption, not a comparison regression. Remaining 960 states and
   1280x720/1920x1080 commands were not completed/run. Run the documented complete
   required matrix after reconciliation; do not infer success from this partial run.
4. **CRITICAL: protected Moon gate blocked.** Final pinned Moon invocation cannot
   download `ghcr.io/moonrepo/javascript_toolchain:1.1.0`: registry connection
   fails with `tunnel error: unsuccessful` at the manifest URL. Earlier attempts
   also failed loading the Bun toolchain. Plain strict OpenSpec validation is not
   a substitute. No plugin, configuration or access restriction was bypassed.
5. **CRITICAL: CODEOWNER review and archive pending.** Obtain designated contract
   review, resolve 5.3/5.4 and all failures, then synchronize/archive and rerun the
   protected final gate. AGENTS.md and the repository verification skill prohibit
   calling this complete while required checks remain failed or unfinished.

A custom official-source FFmpeg 6.1.2 build produced additional failures and was
not equivalent to CI; official Ubuntu packages were used for the decisive replay.
No Windows/macOS or newly published issue44 remote CI run has been claimed.

## Performance and limits

Canonical limits are 16 samples and 268435456 sample-weighted canvas pixel units
per scene frame, with existing stricter effect, geometry, occurrence and surface
limits retained. Exact-boundary and hidden-content failure fixtures pass.
A report-only 64x64, 10-fps, one-second export measured four samples at 673.875 ms
versus one sample on the same raster/FFV1 pipeline at 532.418 ms. A loaded run
measured 1214.684 ms versus 1289.261 ms. Shared CPU contention makes these local
observations unsuitable as universal speed or latency claims; they demonstrate
bounded execution and preserve the repository's report-only policy.

## Evidence and next review

Logs, SHA256 manifest, native command/exit/time records, reaper runner and local
patch are retained outside Git at
`/workspace/AI-Open-Cut-evidence/issue44-20261001/`; logs are not committed.
`proposed-pr.md` contains a reviewable draft title/body, not a published PR.
Review the issue44 commits and separately approve the bounded PR131 corrective
OpenSpec plan if those fixes should proceed. Reconcile/rebase onto the corrected
verified issue43 head without rewriting its branch or merging PR131, resolve native
6.1 failures, finish required gates and owner review, archive, then obtain separate
issue44 publication approval. The existing hourly readonly watch remains independent.
