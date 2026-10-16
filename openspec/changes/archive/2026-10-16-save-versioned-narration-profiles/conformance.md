# Conformance review: save-versioned-narration-profiles

Reviewed 2026-10-16 against the approved proposal, design, both deltas, tasks, implementation and automated evidence. This is a source-versus-artifact conformance pass, separate from mandatory designated CODEOWNER review.

| Dimension | Result |
| --- | --- |
| Completeness | 12/12 local implementation tasks;4/4 requirements;10/10 scenarios have automated evidence |
| Correctness | Versioned shared storage, deterministic explicit application, immutable legacy/profiled lifecycle and attribution migration conform |
| Coherence | ADR0003 ownership retained; root-lock/atomic publication reused; headless thin; provider protocol unchanged |

## Coverage

- Shared bounded versioned library: `crates/editor-core/tests/narration_profiles.rs` covers two-project independent snapshots, optimistic updates, byte-preserving failures, Unicode/ordered overlaps, bounds, corruption/future data and Unix symlinks. Persistence owns bounded file primitives; store owns orchestration and models own validation.
- Explicit application: native tests cover whole-field/empty-list precedence and missing/stale identities; `tests/speech.test.ts` and `tests/narration-profiles-workflow.ts` cover unavailable provider/voice, exact effective requests and nonmutation. The actual workflow passes in both source43/43 and packaged40/40 suites.
- Immutable lifecycle: service tests cover provider mutation rejection/cleanup, retained preview conflict retry and immutable version snapshots; actual MCP covers update versus committed intent and item regeneration. The deterministic `tts-preflight.test.ts` fails before the compatibility correction with TTS_TIMEOUT, then passes with JOB_CANCELLED, FIFO progress and exact cleanup. It preserves every existing timeout.
- Attribution compatibility: native current/retained-history migration, premature null/object metadata and future-version rejection preserve authoritative bytes. Reopen/undo/redo and render plans do not require the library/provider. Legacy JSON omits optional attribution. Render equivalence covers all existing alignment cases with and without attribution.
- Contracts: independently authored canonical additions and inverses, exact predecessor/marker pins, all645 parity cases and native targets pass. Protected CI policy preserves all prior suites and rejects omission/masking of either new governed consumer.

## Findings

No remaining critical implementation, correctness or design mismatch. Earlier failures and their corrections, ordinary native compiler readiness failure, the supported local cross-build method and all local commands are recorded in `docs/verification/narration-profiles.md`; full uncommitted logs are retained in the authorized issue207 verification directory. The isolated full packaged rerun passes with exactly the same binaries and deadlines after one contended hero timeout.

Ready to synchronize and archive this verified implementation. Subsequent final protected Moon/strict validation, scoped commit/push, incremental draft PR, designated CODEOWNER review and all protected CI at the exact published head remain mandatory. Native local ELF compiler failure is not claimed as bootstrap success; ordinary published-head CI must verify it. No merge or deployment is authorized.
