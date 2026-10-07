## Context

Run37613488302 attempts1 and2 on main2d748508 fail the Windows-only descendant fixture at render_worker.rs455. Both wait10 seconds for renderer.pid; neither reaches the5-second post-crash termination assertion. The worker already emits structured events into an unread test channel, while fixture shell/PowerShell stderr is not included in the timeout. Linux-only tests do not exercise this scenario.

## Goals / Non-Goals

Identify the exact startup boundary using real Windows evidence without weakening existing process containment assertions. Production process execution, public contracts and persisted data stay outside this diagnostic phase. Preserve issue60 preparation in the original worktree.

## Decisions

- Add an owned shell-entry marker before the existing fake-tool body and redirect PowerShell stderr to an owned fixture file. Keep the exact command/provider choice initially to avoid correcting an unverified hypothesis.
- Collect correlated worker events during the bounded PID wait and include them with owned startup-marker/error files and worker diagnostics on failure. A typed terminal error remains a failing scenario, never successful cleanup evidence. Keep existing positive PID/handle/liveness checks and exact5-second crash-cleanup assertion.
- Factor formatting of missing/readable/unreadable fixture evidence and events into a test-only portable helper with deterministic tests. Bound emitted evidence to16KiB per source and at most32 captured events to avoid unbounded CI output; truncation is explicitly labeled. No production diagnostics or error semantics change.
- Add a separate focused Windows workflow with pinned toolchain and the exact descendant test. It does not modify or replace existing required workflows/gates or their assertions. Full Windows correctness remains the acceptance authority; focused evidence can fail while diagnosis proceeds.
- Because the first focused instrumented native case passed, execute both original and instrumented fixture bodies sequentially in one Windows diagnostic test process, retaining all startup/liveness/cleanup assertions. The serialized focused comparison is diagnostic evidence only; standard full-workspace acceptance remains unchanged. Report read-only Toolhelp process snapshots filtered to this owned worker's descendant tree, capped64 entries, alongside startup elapsed time on success/failure. Unrelated process identities must never be logged. Additional Windows API feature flags are test-only; production execution remains unchanged.
- Bound both consumed file bytes and rendered UTF-8 evidence to16KiB excluding its explicit truncation label, including malformed-byte replacement expansion; cover that boundary with a binary fixture regression.
- Do not extend deadlines or infer the root cause from PID absence alone. After Windows evidence identifies the cause, amend this change or create a separately approved correction before implementation. A diagnostic PR stays draft and is not described as a resolved baseline.
- Review of full CI37625364905 at1a784c24 shows all substantive jobs pass, but both Windows probes run concurrently and complete together in5.29s. They could warm PowerShell for each other; this is an inference, not a proven explanation. Restore the exact original body as the only default descendant test, keeping correlated event/process reporting. The same test additionally exercises the instrumented body sequentially only when `OPENCUT_WINDOWS_STARTUP_COMPARISON=1` is explicitly set by the focused workflow. Required workflows must not set this diagnostic flag. Collect an unperturbed default full-suite control before drawing corrective conclusions.
- Include the owned PID record's bounded escaped contents on failure so missing publication is distinguishable from incomplete terminators or control bytes. Apply the16KiB rendered-source bound again after debug escaping; perform this file read only on failure, preserving the successful wait path. Cover actual incomplete/NUL-containing files and escaping expansion bounds in portable tests.

## Risks / Trade-offs

- Windows cannot execute in this Linux workspace → run actual Windows CI and report local Windows cross-check/runtime limits separately.
- Worker stdout may contain the missing error evidence → drain only this owned fixture's channel and retain events for failure reporting.
- Shell observation can perturb startup → only write a short owned marker; preserve time budgets and record focused/full CI outcomes.
- Additional focused workflow costs one small native build → its useful early evidence is independent of full desktop compile/tests and cannot weaken required gates.

## Verification

Portable tests verify missing and unreadable files, bounded evidence, exact captured typed error content and truncation labels. Cross-check the Windows target when available; actual Windows startup and cleanup require focused/full CI. Run required Rust fmt/strict workspace Clippy/tests, bridge typecheck/lint/unit/integration/packaged smoke, relevant hermetic Python, all strict specs and protected policy before/after mandatory verify/sync/archive. Final correction must pass exact-head required CI before resuming issue60.

## Migration Plan

No schema, contract or resource migration. All fixture files are beneath the test's owned temporary directory. Rollback removes only these test/CI diagnostics.

## Open Questions

The Windows cause is unconfirmed. Diagnostic outputs must determine shell entry, PowerShell errors and worker response before authorizing a correction.

## Approved prerequisite amendment

The untouched main control now passes renderer startup/cleanup and fails a different independent hero-oracle test at5000ms (job112822127891). This supplies no renderer root cause. Complete the diagnostic outcome with that uncertainty explicit, without production repair. The separately evidenced private verifier timeout is covered by the user's Windows prerequisite-repair authority.

Precompute ordered valid Gaussian neighbors for each coordinate using the same normalized seven weights. Preserve ascending kernel accumulation and horizontal-then-vertical passes; unroll RGBA additions without reassociation. Hoist each plate's invariant quantized hero RGB. Do not vectorize through production code, regenerate references, change frozen catalogs, cache keys/copy isolation, PCM math, tolerances or test deadlines. Existing complete byte/hash/witness/PCM and refusal/cache-isolation assertions are the acceptance authority. Before/after unprofiled timings are local diagnostics; native Windows full-suite evidence is required and a speed claim must identify its environment. This amendment changes only private independent verifier evaluation overhead.
