## Context

The existing foundation job depends on OpenSpec, contract parity, original render parity, and the three rules-screen matrix jobs. Correctness and packaged smoke run in the workflow but are not its direct prerequisites. The measured candidate run for PR #124 took 2h03m39s; the 1920x1080 rules-screen shard dominated the critical path. GitHub Actions' default six-hour job timeout does not express a two-hour workflow budget.

## Goals / Non-Goals

**Goals:** Make the final protected status enforce a 120-minute default run budget, display measured evidence, and permit only a dated and bounded reviewed exception. Include every required job in the measured critical path and preserve every existing assertion.

**Non-Goals:** Change render outputs, reduce test coverage, optimize the renderer, or compact the MCP catalog. GitHub queue delays before a run starts and reporting after the final protected assertion are outside the measurement interval.

## Decisions

1. **Measure at the protected foundation job.** Add correctness and packaged smoke to its terminal prerequisites, then run a final, unconditional duration assertion in that job. Read the current workflow run's `run_started_at` from the GitHub Actions API with a read-only token; compare it with UTC time at the assertion. Fail closed on API, parse, clock, or arithmetic errors. This makes the budget part of the existing branch-protection status. A separate optional report job would not protect merging, and independent per-job timeouts would not bound aggregate elapsed time.
2. **Keep the exception in reviewed CI policy.** The workflow carries one explicit owner, cause, observed baseline, run URL, expiration date, and hard cap. The initial exception may permit up to 135 minutes for the measured 1920x1080 render cost, expiring 2026-10-26; the default stays 120 minutes. The policy validator checks exact structure, expiry syntax, maximum cap, dependencies, final unconditional assertion, token permissions, and absence of failure masking. CODEOWNER review covers edits to the workflow and validator. A free-form `CI_TIMEOUT_OVERRIDE` environment variable would allow unreviewed bypasses.
3. **Preserve complete evidence.** The original parity assertion still checks OpenSpec attestation and all parity results, while the final duration assertion logs all timing and exception fields. Add focused pure tests for time calculation, boundary values, expiration, invalid API payloads, and adversarial workflow changes. The accepted 2h03m39s baseline is evidence for the short-lived exception, not a new permanent target.

## Risks / Trade-offs

- **GitHub API or runner clock failure** → fail the protected status and show a clear diagnostic; retry the workflow when infrastructure recovers.
- **The audit measures shortly before job completion, rather than the post-run API conclusion** → keep the duration assertion last and define this point as the protected gate boundary; the remaining runner teardown is outside the policy measurement.
- **A 135-minute cap can still exceed the desired two hours** → require a dated cause and owner, expire it, and use telemetry to prioritize renderer optimization without dropping tests.
- **A long-running job can consume time before the final gate detects an overrun** → set explicit bounded job timeouts where the CI policy can enforce them; audit remains authoritative for total elapsed time.

## Migration Plan

Add policy and tests, run local validation, verify OpenSpec conformance, then sync and archive before requiring the protected CI status on the new commit. A rollback reverts the coordinated OpenSpec, workflow, validator, and documentation change. Product contracts and persisted data need no migration.

## Open Questions

None. The first exception is limited to the known PR #124 render baseline and must be renewed or removed through reviewed policy changes after expiry.
