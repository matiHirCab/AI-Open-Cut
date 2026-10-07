## Context

Run37613488302 attempts1 and2 on main2d748508 fail the Windows-only descendant fixture at render_worker.rs455. Both wait10 seconds for renderer.pid; neither reaches the5-second post-crash termination assertion. The worker already emits structured events into an unread test channel, while fixture shell/PowerShell stderr is not included in the timeout. Linux-only tests do not exercise this scenario.

## Goals / Non-Goals

Identify the exact startup boundary using real Windows evidence without weakening existing process containment assertions. Production process execution, public contracts and persisted data stay outside this diagnostic phase. Preserve issue60 preparation in the original worktree.

## Decisions

- Add an owned shell-entry marker before the existing fake-tool body and redirect PowerShell stderr to an owned fixture file. Keep the exact command/provider choice initially to avoid correcting an unverified hypothesis.
- Collect correlated worker events during the bounded PID wait and include them with owned startup-marker/error files and worker diagnostics on failure. A typed terminal error remains a failing scenario, never successful cleanup evidence. Keep existing positive PID/handle/liveness checks and exact5-second crash-cleanup assertion.
- Factor formatting of missing/readable/unreadable fixture evidence and events into a test-only portable helper with deterministic tests. Bound emitted evidence to16KiB per source and at most32 captured events to avoid unbounded CI output; truncation is explicitly labeled. No production diagnostics or error semantics change.
- Add a separate focused Windows workflow with pinned toolchain and the exact descendant test. It does not modify or replace existing required workflows/gates or their assertions. Full Windows correctness remains the acceptance authority; focused evidence can fail while diagnosis proceeds.
- Do not extend deadlines or infer the root cause from PID absence alone. After Windows evidence identifies the cause, amend this change or create a separately approved correction before implementation. A diagnostic PR stays draft and is not described as a resolved baseline.

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
