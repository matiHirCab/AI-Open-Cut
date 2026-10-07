# Interim conformance review: diagnose-windows-renderer-startup

Separate review pass using the repository openspec-verify-change workflow on2026-10-07; reviewed all apply context artifacts and the complete main-relative diff. This is an interim diagnostic assessment, not final correction acceptance or archive authorization.

| Dimension | Assessment |
| --- | --- |
| Completeness | 8/13 tasks complete; cause and final acceptance unresolved |
| Correctness | Both diagnostic requirements implemented; all6 scenarios have portable or native evidence |
| Coherence | Production/contracts/required workflows unchanged; one missing failure-timing field corrected locally |

## Requirement and scenario evidence

- Attributable startup: apps/headless/tests/render_worker.rs captures correlated worker events and owned marker/stderr files, fails on premature terminal events and uses bounded reporting. `typed_failure_preserves_owned_startup_and_worker_evidence` checks the actual file/event values.
- Missing/unreadable/oversized evidence: tests/support/render_worker_startup.rs uses capped file reads, explicit missing/unreadable labels and UTF-8-safe output bounds. `missing_and_unreadable_owned_files_are_attributable` and `multibyte_sources_and_event_count_are_bounded_and_labeled` exercise real owned files and event limits.
- Exact cleanup proof: original10-second observation and5000ms post-crash waits, pinned OwnedHandle, WAIT_TIMEOUT/WAIT_OBJECT_0 assertions and owned-fixture cleanup are retained. Native workflow37625364944/job112805627441 on head1a784c2427a45f4060661e76851703c1e0801c67 passes both bodies, including the cleanup assertion.
- Paired comparison: the native process executes both bodies sequentially and logs worker→cmd.exe→powershell.exe ownership. Original startup3.7518616s; instrumented216.3538ms. Portable ancestry tests reject unrelated identities, resolve reversed ancestry and enforce64 descendants. Snapshot API failures are explicit; global read-only enumeration is capped16384 records. Deterministically inducing OS snapshot denial is not portable; native execution covers successful enumeration and source inspection covers error handling. Focused serialization is distinct from unchanged full-suite acceptance.
- Malformed bytes: `invalid_utf8_expansion_stays_within_the_rendered_source_bound` verifies replacement expansion cannot exceed16KiB before its truncation label.
- Focused workflow:22 automated policy assertions pin tool versions, main targeting, read-only permissions, exact paired selection and unsuppressed failures. Existing two required workflows have no main-relative diff. Actual Windows execution passes; Linux compilation is never presented as Windows runtime proof.

## Findings and resolution

The design promised startup timing on failure as well as success. The original diagnostic timeout/terminal-event paths omitted elapsed time. The local review correction adds elapsed time to those messages and includes owned process evidence on unexpected PID-file read failure. It changes reporting only; Windows cross-compilation and the next native head must verify it before final acceptance.

## Critical completion blockers

1. Task3.3: focused native passes do not explain either original full-main failure. Wait for full exact-head Windows evidence and approve any correction only after an evidenced cause. No speculative production fix is justified.
2. Task4.1: no cause-specific correction is implemented or approved.
3. Task4.2: full final-head standard CI acceptance is pending. Original and heap-limited local integration runs lost workers to observed OOM kills; complete GC-runtime diagnostic execution passes23/23 with no additional OOM kill. Standard Linux unit execution retains two zombie-related failures; the complete suite passes587 tests under a proper subreaper. These runtime adjustments are diagnostic evidence, not standard CI acceptance. Initial full Rust compilation exhausted temporary disk space; its log is preserved and the identical suite passes after moving artifacts to the larger workspace filesystem. Initial smoke had an interrupted worker; standard rerun passes20/20. Final typecheck/lint pass; focused formatting/Clippy/Linux tests/Windows cross-compilation/spec validation pass after the reviewed reporting correction. Standard Ubuntu correctness and packaged integration pass at1a784c24 in CI37625364905. Preserve these originals and await final Windows evidence.
4. Task4.3: sync/archive and final protected policy remain blocked until all required implementation checks and final conformance pass. The actual pinned Moon task ran451 policy tests and strict validation46/46, then rejected only this active change as required; this rejection is not gate success. MOON_HOME/PROTO_HOME/XDG_CACHE_HOME were redirected into writable workspace locations and MOON_TOOLCHAIN_FORCE_GLOBALS used the verified pinned tools without omitting any task checks.
5. Task4.4: PR154 remains draft targeting main and explicitly investigative. Final correction description and exact final-head full required CI are unresolved. Do not resume issue60 implementation prematurely.

Five critical task blockers remain. Keep the change active and the draft incomplete; no archive or baseline-resolution claim is warranted.
