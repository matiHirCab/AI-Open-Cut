# Prepublication conformance: diagnose-windows-renderer-startup

Separate final review pass using repository openspec-verify-change on2026-10-07, including all amended proposal/design/deltas/tasks, complete main-relative implementation diff and native CI evidence. This is the implementing agent's separate review pass, not an external reviewer claim.

| Dimension | Assessment |
| --- | --- |
| Completeness | All13 implementation tasks complete;3 requirements/10 scenarios covered |
| Correctness | Complete local and standard native acceptance; no assertion, deadline, frozen authority or production behavior change |
| Coherence | Diagnostic-only renderer work plus independently approved private scalar verifier correction; original renderer cause remains unresolved |

## Attributable Windows startup evidence

`apps/headless/tests/render_worker.rs` retains the exact original default batch body,10-second PID wait, exact owned handle/liveness check and5000ms post-crash cleanup assertion. Correlated worker events are captured with premature typed result/error/mismatched identity remaining failure. Additional shell/stderr instrumentation runs sequentially only in the focused workflow under its explicit flag, absent from both unchanged required workflows.

Portable helper tests map scenarios: `typed_failure_preserves_owned_startup_and_worker_evidence`; `missing_and_unreadable_owned_files_are_attributable`; `multibyte_sources_and_event_count_are_bounded_and_labeled`; `invalid_utf8_expansion_stays_within_the_rendered_source_bound`; `pid_publication_evidence_preserves_incomplete_and_control_byte_records`; ancestry filtering/reversed ancestry and64-descendant limits. Actual-file PID tests preserve incomplete terminators/NUL and escape-expansion bounds. Output consumes/renders at most16KiB/source with explicit truncation and32 events. Read-only Windows ToolHelp snapshots omit unrelated processes, bound total enumeration and owned descendants, and report OS failures explicitly. Portable deterministic denial of the snapshot API is unavailable; native execution covers successful enumeration, and source review covers its errors/RAII handle closure.

The focused workflow's24 automated assertions pin main targeting, read-only permissions, Bun1.4/Rust1.97, exact native test/flag, no masks/conditional skips and flag absence from required workflows. Latest native focused run37636743865/job112844849273 passes original startup2.0405776s and instrumented226.1963ms with owned worker→cmd.exe→powershell.exe and both cleanup assertions. Full required Windows correctness uses only the original body and also passes.

## Independent scalar verifier correction

Untouched main attempt3/job112822127891 passes renderer cleanup but fails the independent hero authority test at its unchanged5000ms deadline. Approved ordered neighbor precomputation and explicit RGBA accumulation remove redundant loop work without reordering any scalar multiplication/addition, kernel, pass, padding or channel. Invariant color hoisting calls identical quantization/linearization expressions. No production helper, frozen reference/catalog, PCM expression, cache key/copy isolation, refusal assertion or deadline changes.

Unchanged checked-in `masked-hero-reveal.test.ts` verifies all16 plates/eight witnesses/38400 stereo PCM frames, immutable hashes/bytes, cache mutation/input isolation and existing-directory refusal. Optimized local4/4 pass, full proper-init units587 pass with9 existing skips. Supplemental automated original-versus-optimized full-float comparison passes76800 channels over edge impulses, dense and signed inputs. Linux timings1.36–1.47s versus1.69–1.78s are environment-specific. Native standard Windows job112844850213 passes the original renderer and full587 units/9 existing skips; unchanged hero test3074ms, four hero tests3157ms. Full log `/tmp/opencut-windows-correctness-head13497ac4.log`. See oracle-efficiency-evidence.md for provenance and explicit timing limits.

## Required implementation acceptance

All9 substantive jobs SUCCESS on exact code head13497ac4b5c6712b7f05a42b3d70ba1a68a89584, CI37636743742: Windows/macOS/Ubuntu full formatting/strict workspace Clippy/workspace Rust/typecheck/lint/unit/Python correctness, canonical contracts, standard packaged integration/smoke, all3 rules resolutions and native audiovisual/lifecycle/raster-cache parity. Focused native diagnostics also pass. Policy job112844850396 rejects only this active change; dependent foundation112860216192 therefore fails while its protected-duration enforcement passes. This is expected prearchive rejection, not final gate success.

Local full fmt/Rust/strict Clippy/typecheck/lint/smoke20/20/Python12+5 pass. Latest affected full units587 pass under a proper subreaper; hero4/4/Python rerun after the private Python amendment. Strict specs46/46 and protected prearchive451 policy tests pass before rejection naming only this change.

Original local failures remain preserved: initial Rust ENOSPC passes identical suite after moving our cache to larger workspace storage; missing local binary resolved with a target link; exactly two standard Linux unit signal-zero assertions observe owned PID stateZ/parent1 and full suite passes under proper init; original standard and heap-limited integration suffer observed cgroup OOM kills, while complete runtime-only GC diagnostics pass23/23. Initial smoke interruption is preserved and standard recovery passes20/20. These runtime adjustments are diagnostic evidence, never substituted for the current standard native CI success. Detailed original log paths and runtime findings remain in diagnostic-review.md and the earlier review commit history.

## Preservation and archive authorization

No diff in production contracts or the two existing required workflows. All1260 existing living/archive files are frozen in `/tmp/opencut-prerequisite-pre-sync-input-pins.json` and unchanged before synchronization. Approve own-only addition of two repository-validation requirements and one masked-hero-reveal requirement, preserving every unrelated existing block and old archive file. All implementation conformance issues resolved; no critical/warning/suggestion remains. The earlier renderer failure's unknown cause is a recorded diagnostic limit, not an inferred production repair.

Mandatory administrative gates are PENDING: sync/archive only this verified change, prove exact accepted-block/preservation equality, and pass unchanged protected Moon plus strict living-spec validation before publication. Mandatory publication is PENDING: push archived head, update draft PR154 targeting main and require every exact-head required CI job before advancing to60. These future gates are not claimed by this historical prepublication snapshot.

## Final archive validation

Own change synchronized/archived on2026-10-07. preservation.json records exact accepted delta suffixes, all515 original living requirements and1215 prior archive files preserved. No active change remains in this checkout. Protected Moon exits0 with451 policy tests and strict45/45 living specs; independent strict pinned validation also exits0 with45/45. Logs `/tmp/opencut-prerequisite-postarchive-protected-gate.log` and `/tmp/opencut-prerequisite-postarchive-strict-specs.log`. These actual results supersede administrative PENDING above. Executable inputs remain identical to standard passing CI13497ac4. Publication remains PENDING until all required CI on the final archived head passes; no overall completion or issue60 continuation is claimed yet.
