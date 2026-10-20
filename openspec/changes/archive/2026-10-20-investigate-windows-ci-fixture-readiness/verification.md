# Verification: investigate-windows-ci-fixture-readiness

Status: local implementation verified with openspec-verify-change. All required implementation checks passed. The two approved requirements were synchronized and only this change was archived; postarchive protected and strict validation passed. Published-head CI acceptance remains pending and will be recorded in PR metadata, not asserted here in advance.

## Scorecard

| Dimension | Assessment |
| --- | --- |
| Completeness |10/10 implementation/preparation tasks; two requirements and six scenarios have coverage in traceability.md. Postarchive publication/CI execution is explicitly pending in delivery.md. |
| Correctness | Atomic closed PID publication, original-error diagnostic cleanup, real cold job processes and owned child settlement match approved requirements and tests. All original assertions/time bounds remain. |
| Coherence | Three private test/fixture files only; real production classes are bundled at test setup from current source. No production code, contracts, dependencies, workflow or governance hash changes. |

No critical implementation mismatch or unresolved design warning remains. Independent reviewer approved concrete artifacts before edits and independently reviewed the restart code; the main agent reviewed normalization. This is not CODEOWNER approval. Existing root AGENTS ownership boundaries and #208/#211/#228 exclusions remain respected.

## Automated evidence

| Command/acceptance | Observed result and full local log |
| --- | --- |
| cargo fmt --all --check; standalone native rustfmt/strict compilation | Passed; /tmp/opencut-final-rustfmt.log and /tmp/opencut-normalization-changed.log; native harness also compiles with -Dwarnings inside the mandatory suite. |
| cargo clippy --workspace --all-targets -- -D warnings | Passed; /tmp/opencut-clippy.log. Existing proc-macro-error2 future-compatibility notice remains, without suppression. |
| cargo test --workspace | Passed1597 test executions/82 summaries,0 failures,14 existing ignored optional helpers/evidence cases; /tmp/opencut-workspace-tests-prepared.log. |
| Bridge typecheck/lint | Passed; /tmp/opencut-typecheck.log, /tmp/opencut-final-typecheck.log, /tmp/opencut-lint.log (296 files). |
| Focused restart/normalization |12/12 and58/58 passed; /tmp/opencut-review-restart.log and /tmp/opencut-normalization-changed.log. Native publication harness3/3 passed. |
| Full default bridge unit suite |994 passed,15 existing optional/platform skips; /tmp/opencut-unit-all.log. No file-parallelism/configuration changes. |
| Full source MCP integration |43/43 passed; /tmp/opencut-integration-prepared.log. |
| Actual assembled release runtime smoke |40/40 passed; /tmp/opencut-packaged-smoke.log. |
| Hermetic workers/platform preparation |12 unittest+12pytest and5 setup controls passed; /tmp/opencut-python-workers.log, /tmp/opencut-platform-setup-tests.log. |
| Unchanged Motion workflow policies |52 passed; /tmp/opencut-motion-policy.log. |
| Prearchive protected Moon gate |521 policy tests and77 specification items passed, then rejected only this own active change as required; /tmp/opencut-prearchive-protected.log. Expected rejection is not final gate success. |

No required implementation check is skipped. Existing ignored/optional cases are unchanged. All generated tools, caches, native linker preparation, child reaper and lengthy logs remain uncommitted outside the change.

## Retained failures and limits

Chronological evidence.md records the initial API/shell/tool availability mistakes and their resolution through the authorized GitHub connector and isolated official pinned artifacts. No credentials, permissions or security configuration changed.

Unmodified local normalization initially failed27 lifecycle cases because PID1 retained terminated descendants as zombies. The owned subreaper launcher fixes only local test-process reaping; exact PID assertions stay intact. Initial full Rust failed desktop linkage; a temporary linker directory points at installed real system libraries. The prepared complete run passed. Initial source integration passed41/43 with grid polling and PID-absence failures while native work was busy and the owned subreaper was accidentally omitted. The complete unchanged rerun passed43/43; precise grid timing causality is not established. All failed logs remain linked in evidence.md.

The restart optimization's Windows effect still requires exact published-head acceptance. Local startup samples are limited Linux measurements, not proof that all timing variance is eliminated. Native lifetime stub metrics/PCM are process-ownership evidence, not actual media-quality proof.

## Infrastructure evidence and local closure

Both current main Motion runs originally failed Chocolatey FFmpeg7.1.1 with HTTP503; separate read-only PR228 logs confirm HTTP499 at the same feed. Exactly one authorized rerun each of the failed main Windows jobs completed all native tests and uploads successfully at unchanged main756b7b39 (attempt2). URLs and full logs are in evidence.md. Workflow changes were therefore removed from scope. No PR228 mutation/rerun or peer-branch rewrite occurred.

The two approved ADDED requirements were synchronized into existing job-restart-semantics and master-loudness-normalization living specs without replacing unrelated content. Only this verified change was archived at2026-10-20-investigate-windows-ci-fixture-readiness with metadata preserved. Postarchive protected Moon passed521 policy tests/76 living specification items, and separate pinned strict all-spec validation passed76/76. Logs: /tmp/opencut-postarchive-protected.log and /tmp/opencut-postarchive-strict.log. Final prepublication frozen-tree reruns are required after this metadata update; full logs /tmp/opencut-prepublication-protected.log and /tmp/opencut-prepublication-strict.log must pass before publication. The reviewed delivery checklist requires a draft PR on the isolated branch, verified remote tree and terminal exact-head checks. No merge/deployment is authorized. Do not claim main repaired before incorporation.
