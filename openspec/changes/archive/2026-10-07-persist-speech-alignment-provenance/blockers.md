# Historical prerequisite blocker before issue60 implementation

On2026-10-07, main `2d748508b7838a7a3150837b6ac2556b50b5dec4` failed required Windows Rust CI in both attempts of run37613488302. The retried Windows job112778703375 completed with failure; its dependent Motion-graphics foundation parity job112790902574 also failed. Original other substantive jobs passed.

Both attempts fail the same Windows-only test, `crashed_worker_terminates_renderer_descendants`, at `apps/headless/tests/render_worker.rs:455`: “renderer PID record never became readable”. Attempt2 reports3 passed/1 failed for that test binary. Failure occurs after about10 seconds waiting for the fixture PID record and before asserting descendant termination. It does not establish a confirmed production process-cleanup defect. This is distinct from the user's known Linux orphan-zombie/Vitest-memory caveats; no diagnostic runtime adjustment or coverage weakening was applied here.

Original full logs:

- `/tmp/opencut-main-ci37613488302-windows-attempt1.log`
- `/tmp/opencut-main-ci37613488302-windows-attempt2.log`

Rerun evidence: https://github.com/matiHirCab/AI-Open-Cut/actions/runs/37613488302/job/112778703375

The latest user delegation explicitly authorized isolated investigation and an evidenced baseline correction under a separate specification. That ownership/scope question is resolved. Work is isolated in `/workspace/AI-Open-Cut-windows-fix`, branch `fix/windows-renderer-startup-20261007`, investigative draft PR154 targeting main. Cause-specific correction, full required checks and exact-head final CI remain unresolved. Do not speculate about the cause, skip the Windows scenario, weaken its assertion/deadline, or classify Linux-only passing tests as Windows acceptance.

Paired native diagnostic workflow37625364944/job112805627441 at head1a784c2427a45f4060661e76851703c1e0801c67 passed the exact original body and instrumented body, including descendant cleanup. Original startup3.7518616s and instrumented216.3538ms; read-only snapshots show each owned worker→cmd.exe→powershell.exe tree. Full run37625364905 then passed all9 substantive jobs, including Windows/macOS/Ubuntu and all native parity/integration checks; active OpenSpec policy and dependent foundation rejection remain expected and are not acceptance. Its two default probes ran concurrently in5.29s, potentially confounding startup through mutual warming. That inference does not establish either original failure's cause.

The latest diagnostic headf526b84c15535a13a827af8325e64ab3eed9e7f5 restores the exact original batch body as the sole default descendant test. Only the focused workflow enables the additional sequential instrumented exercise. New bounded escaped PID-file evidence separates missing/incomplete/control-byte publication without changing PID acceptance. Focused run37630126591 passes original first (1.7838647s) then instrumented (246.6934ms), including cleanup. Local full Rust and strict workspace Clippy pass on this head; bridge original zombie/OOM failures and diagnostic runtime successes remain preserved and distinct from standard CI. Full original-only PR run37630126575 is pending; unchanged main's Windows control is rerunning as job112822127891 in run37613488302. Main remains2d748508. Explicit user prerequisite-repair approval was reconfirmed at13:16:48UTC. Preserve this issue60 preparation until baseline verification is reconciled; no cause-specific correction is yet justified.

Issue60 preparation is preserved locally on `feat/issue-60-speech-provenance-20261007`: proposal, design, all four delta specs, tasks, delegated approval, verification plan, proposed independent21-valid/20-invalid fixtures,17 unmodified predecessor raw catalog pins, and roadmap readiness notes. Strict pinned all-spec validation passes46/46. No implementation code, living specification, CI policy, or test has changed; nothing was committed/pushed and no PR was created. The active change is intentionally unarchived and not merge-ready. Issue60 remains incomplete.

Native development dependencies and pinned tooling are now available in isolated/tmp paths as documented in roadmap-readiness.md. Environment installation is not the remaining blocker.

## Current state after implementation and delegated review

The historical prerequisite is resolved by verified PR154 head72bd389b and the user's later main mergec756a999, retaining identical source and ancestry. Issue60 implementation is published as draft PR155 targeting main. Standard initial CI passes23 integration/20 packaged smoke; all three initial correctness failures reveal missing ambient FFprobe in the newly added protocol test. That confirmed harness mismatch is repaired under an approved amendment with all47 unconditional hermetic cases plus all47 actual-native cases. Corrected full Rust1,365, strict Clippy/fmt, full contract483, unaffected bridge629/Python12+5 and complete conformance checks pass; original unsuccessful logs remain preserved.

At19:02:25UTC the user authorized substantive CODEOWNER review. The separate delegated review records its method, finding, correction evidence and approval decision; no distinct human/platform approval is inferred. No user-input blocker remains for this implementation phase. Synchronization/archive/final protected gate and published-final-head all11CI are still pending delivery obligations, and issue61 implementation must wait for that final success. Human/platform merge approval remains a separate unfulfilled pre-merge obligation; no merge/deployment/issue closure is authorized.
