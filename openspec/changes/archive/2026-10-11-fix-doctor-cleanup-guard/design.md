## Context

Main `07d2c687` acquires the runtime guard before `runDoctor`, then sets `doctorClean = report.ready`. Readiness is false for any diagnostic error, including missing optional models. The finally block completes service cleanup but calls `release(false)`, causing a false crash marker. The audit reproduced this with no generation or residual files.

## Goals / Non-goals

Restore admission after completed diagnostics and confirmed cleanup even if diagnostics are unready. Keep every actual crash/cleanup/ownership safeguard. Do not alter domain code, contracts, schemas, concurrency, retries, marker format or operator crash recovery.

## Decisions

Track whether `runDoctor` returned, independently from its `ready` result. Set that flag only after return. Keep the existing ordered headless/speech/transcription close sequence, and release only after every close succeeds. If doctor throws (including fixture deletion failure), the flag remains false; if close throws, release is never reached. Unsafe or foreign guards remain protected by unchanged `JobRuntimeGuard` checks.

Writeability probes also create owned temporary files. Their cleanup must run even if writing partially fails, and deletion failure must escape the ordinary unavailable-directory diagnostic handler as a private typed cleanup error. Settle every probe before evaluating results, so an early ordinary failure cannot hide later cleanup debt. Reuse the same safe cleanup error for transcription doctor fixtures; no private path enters its message.

Use a real source CLI test with fresh work roots and unavailable dependencies to reproduce clean unsuccessful diagnostics without native build/model requirements. Supplement it with a subprocess preload fixture that controls diagnostic completion and individual service-close failures while keeping actual CLI/guard code. Test successful readiness, all three service-cleanup failures, thrown doctor cleanup and occupied foreign markers. Existing registry process-death, provider retirement, preview debt and HTTP/artifact isolation suites remain unchanged.

Simply removing the release condition is rejected: a thrown doctor fixture-cleanup failure may leave output outside provider ownership. Conditioning release on aggregate readiness is rejected because readiness is not cleanup evidence. A private writeability helper keeps the existing report shape while distinguishing ordinary diagnostics from owned cleanup debt.

## Compatibility, privacy and rollback

No cross-language contract or persisted representation changes. Existing exit codes and diagnostic reports remain unchanged. Guard nonce, identity checks, confinement and production admission stay unchanged; no private data or audit evidence is published. Rollback restores only the erroneous clean-diagnostic guard retention and requires no migration.

## Verification

First run the new regression against existing production code and record failure. Then run CLI/lifecycle tests, bridge typecheck/lint/unit, Python tests and repository-required Rust/contract/source integration/package checks where the pinned toolchain is available. Run strict OpenSpec and protected policy preflight before archive; it must reject only this active change at that point. Record unavailable checks explicitly, never label incomplete verification as passed, and retain an active change if required checks remain blocked. Synchronize/archive only after conformance and required checks pass, then rerun protected policy. Draft PR and CI reporting remain honest about outstanding verification.
