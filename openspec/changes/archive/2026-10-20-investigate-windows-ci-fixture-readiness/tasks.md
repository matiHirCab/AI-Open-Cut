## 1. Establish current evidence and approval

- [x] 1.1 Read root instructions, README, OpenSpec workflow conventions and relevant living specs/history; verify clean checkout and remote main HEAD; create isolated branch.
- [x] 1.2 Use the working GitHub connector to read current main runs, exact failing jobs/logs and #208/#211/#228 boundaries; retain terminal rerun outcomes. Rerun each relevant failed job at most once if needed; retain URLs, attempts and terminal outcomes. Remove already resolved failures from scope.
- [x] 1.3 Obtain independent review of concrete artifacts; resolve scope using current evidence; pinned strict proposal validation passed. No implementation before this step.

## 2. Scoped implementation if failures remain

- [x] 2.1 Add regression for incomplete native PID publication and fix only private normalization readiness protocol while preserving original limits/assertions/diagnostics.
- [x] 2.2 Bundle the real crash fixture once in bounded setup and require exact-head Windows acceptance for the startup hypothesis without changing job contracts or timeouts; add regression.
- [x] 2.3 Record both successful unchanged Motion terminal rerun outcomes and remove workflow implementation scope. Stop if permission, secret or security changes are necessary.

## 3. Required verification and delivery

- [x] 3.1 Use isolated official pinned tools (preparation recorded in evidence.md; normal repository setup is `proto use`); `cargo build -p opencut-headless`; in apps/agent-bridge run `bun install --frozen-lockfile`, `bun x --no-install vitest run --config vitest.unit.config.ts tests/job-restart.test.ts` and `bun x --no-install vitest run --config vitest.unit.config.ts tests/master-normalization-lifetime.test.ts`, `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, `bun run test:smoke`.
- [x] 3.2 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `python3 scripts/test_setup_platform_renderer.py`, `bun test scripts/platform-renderer-policy.test.ts scripts/motion-release-policy.test.ts`, and relevant hermetic worker checks from repository task definitions.
- [x] 3.3 Run pinned strict OpenSpec validation and prearchive `moon run root:openspec-validate`; only this active change may explain rejection. Invoke openspec-verify-change and record observed implementation conformance with the synchronization/archive/final-gate checklist. Execute that mandatory local closure next; no archive before required implementation checks pass.
- [x] 3.4 Prepare reviewable draft PR body and postarchive publication/terminal-check checklist in delivery.md. Actual publication and exact-head outcomes remain separately pending until observed in PR metadata. No merge or deployment.

- [x] 3.5 Obtain explicit review of the concrete complete-live-control follow-up in design.md before editing executable code. Require the exact tracked owned PID inside the existing wait, preserve all bounds and original assertions, and rerun affected restart/full-unit/types/lint checks. Prearchive policy/spec checks pass with expected rejection only for this own active change. Conformance, rearchive and final protected/strict validation remain mandatory next steps; actual terminal published-head evidence belongs in PR metadata.
