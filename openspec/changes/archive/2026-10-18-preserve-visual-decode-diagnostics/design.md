## Context

`SystemProcessExecutor::decode_visual_frame_at` collects status/stderr but returns a provisional read error with `result?` first. This loses diagnostics on precisely the empty/short-read paths reported by #177. Rendering stays in the ADR0003 core process adapter; ADR0004 seek, sampling, color and scene semantics are unchanged.

## Goals / Non-Goals

Preserve bounded sanitized diagnostics, distinguish failure classes and guarantee child cleanup. Do not change public schemas/catalogs, dependency direction, project persistence, migrations, speech or provider code. Do not diagnose the unavailable private recording or promise a Windows media fix.

## Decisions

1. Extract the bounded two-buffer frame reader so deterministic short, interrupted, EOF and injected I/O failures can be tested independently. A private failure enum supplies fixed messages; avoid raw `io::Error` text because it may contain paths. New public error fields/codes would need cross-language governance and are unnecessary.
2. Collect status and stderr before forming the public error. EOF/incomplete EOF gives the producer a bounded 100ms exit-status collection grace, then kills/reaps it if still alive. Missing pipes and non-EOF I/O failures kill and reap immediately. Prefer the observed read failure when both a read and child failure occur, but retain child status/stderr. No new timeout or cancellation protocol is introduced.
3. Reuse the bounded collector behind `read_bounded_tail` and `stderr_excerpt`; preserve the fallible wrapper for other consumers and do not change sanitization policy. Keep error code, stage and retryability intact. More specific message copy is additive diagnostic detail within existing error fields, not a new wire contract.
4. Use a tiny Rust fake decoder executable for deterministic process outcomes without shell portability assumptions. Reader tests inject I/O errors unavailable deterministically through an ordinary OS pipe. A configured native synthetic 720p30 source tests real successful decoding around a boundary, separately from private-runtime reproduction.

## Risks / Trade-offs

- A child closing stdout but remaining alive must not expand indefinite waiting into output-failure paths. The internal EOF-failure grace is bounded at 100ms and then uses the existing kill/reap policy; successful output retains its existing wait behavior. No public timeout/cancellation protocol changes.
- Forced termination after I/O failure may provide a signal rather than an exit code. Preserve whatever the OS supplies without inventing status.
- Generic existing stderr sanitization remains the established boundary. Add private-path/Unicode/length assertions to this path.
- This fixes evidence loss, not the unknown Windows decode cause. Keep #177 open and explicitly record the missing affected-runtime reproduction.

## Migration Plan

No data/contract/dependency migration. Revert the scoped process adapter/tests/spec change to roll back. Existing failure publication paths continue handling errors; no artifact creation is added by the decoder.

## Verification

Use scenario-mapped reader tests, fake subprocess tests (including successful complete output followed by a nonzero exit), missing executable, cleanup, and a native source control. Run strict all-spec validation, formatting, workspace Clippy/tests, TypeScript lint/type/unit, contract parity, source MCP integration, packaged smoke and hermetic worker suites. Run the protected gate before archival, verify conformance, synchronize/archive only after required suites pass, then run final protected/strict validation. Failed required checks block verified publication.

## Independent review follow-up

The read-only reviewer confirmed that loss of accumulated stderr on collector I/O error predates this branch, but conflicts with the new preservation guarantee. Extend the internal bounded collector to return both its accumulated tail and completion result. The existing fallible wrapper retains its signature/error behavior for other consumers; visual decoding retains the tail even on collection error. Retry Interrupted according to normal Read semantics. Add an injected-reader test that interrupts, emits more than the bound, then fails, plus coverage of the legacy wrapper's non-interrupted failure behavior. This remains editor-core internal behavior with no dependency/public contract changes. Approved under the same delegated specification authority before the follow-up implementation.

The reviewer confirmed that the initial EOF wait expanded the non-exiting-producer risk to failure paths. Resolve it before publication: poll try_wait during an internal bounded 100ms collection grace after Empty/IncompleteFrame, preserve any observed status, then terminate/reap if still alive. Missing-pipe/non-EOF I/O failures retain immediate termination. A deterministic fixture closes stdout, remains alive, and writes a marker only if it survives; absence of that marker after the call proves cleanup without a timing-threshold assertion. This is an internal failure-cleanup guard, not a new public render timeout/cancellation protocol. Approved under delegated authority before implementation.
