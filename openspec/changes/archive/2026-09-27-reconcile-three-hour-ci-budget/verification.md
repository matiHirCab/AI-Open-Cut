## Verification Report: reconcile-three-hour-ci-budget

| Dimension | Status |
| --- | --- |
| Completeness | 8/8 tasks complete; 1 modified requirement and the existing rules-screen requirement implemented |
| Correctness | All exception, timeout, expiry, failure, and bounded-other-job scenarios covered |
| Coherence | Preserves `main`'s 120-minute default, six required job results, final duration audit, and other job limits |

### Requirement and scenario evidence

- `.github/workflows/bun-ci.yml` limits only the rules-screen matrix to 180 minutes, records the owner, 135.25-minute observation, PR #126 run evidence, 2026-10-26 expiry, and 180-minute exception cap. Other required leaf jobs remain at 135 minutes and foundation at 10.
- `scripts/validate-ci-gates.ts` requires those exact workflow properties and retains closed steps, environments, dependencies, failure propagation, and attestation. `scripts/validate-ci-gates.test.ts` accepts the reviewed workflow and rejects missing, short, long, and string shard timeouts, changed duration fields, and extended other-job timeouts.
- `scripts/ci-duration.ts` retains the 120-minute default and fail-closed Actions timing while allowing an exception no greater than 180 minutes. `scripts/ci-duration.test.ts` covers the exact 180-minute boundary, overrun, malformed and expired exceptions, and unavailable timing evidence.
- `docs/ci-parity-gates.md` distinguishes the earlier successful baseline from PR #126's later canceled run and states that the cancellation cause is unconfirmed. The living repository-validation spec matches the approved delta while retaining `main`'s other requirements.

### Check results

- Focused CI policy and duration tests: 363 passed, 0 failed; strict all-spec validation: 30 passed, 0 failed.
- `cargo fmt --check --all`, strict workspace Clippy, and `cargo test --workspace`: passed.
- Bridge typecheck, lint, and unit tests: passed (418 unit tests, 1 skipped). Contract parity: 352 passed. MCP integration: 12 passed. Packaged smoke: 6 passed. Python worker: 10 unittest and 5 pytest passed. Agent context/skill tests: 20 passed.
- Full logs are under `C:\Users\matia\AppData\Local\Temp\opencut-pr126-merge-checks\`; strict spec log is `C:\Users\matia\AppData\Local\Temp\opencut-pr126-merge-openspec.log`.
- The protected pre-archive Moon gate exited 1 solely because `reconcile-three-hour-ci-budget` is active; its strict validation passed 30/30. Log: `C:\Users\matia\AppData\Local\Temp\opencut-pr126-merge-checks\moon-prearchive.log`. This is the expected archive-only rule. The post-archive gate remains to run.

### Issues

No critical or warning issues found. A future GitHub run is needed to observe whether the longer budget is sufficient; the previous cancellation source remains unknown.
