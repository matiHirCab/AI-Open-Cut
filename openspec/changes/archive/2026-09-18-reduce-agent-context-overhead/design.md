## Context

OpenCut now has project-local gpt-6-astra and medium reasoning/planning settings, nine local-marketplace plugin overrides, guidance, and focused tests. The review found that remote desktop skills remain available despite some local overrides and that the completion sequence waits for an archive-only gate before archival. The unrelated content-addressed-font-shaping change is now archived; only this change remains active. Its implementation remains out of scope. The prior renderer.rs compilation failure no longer reproduces in targeted test compilation, which does not establish a passing full workspace test run.

## Goals / Non-Goals

Goals: reduce avoidable context and planning overhead locally; preserve all current safeguards; provide testable settings and honest verification evidence.

Non-goals: application changes, global settings changes, model substitution, modified generated skills, reduced check coverage, guaranteed usage savings, or automatic task creation for testing.

## Decisions

1. Retain `.codex/config.toml` with `model = "gpt-6-astra"`, `model_reasoning_effort = "medium"`, and `plan_mode_reasoning_effort = "medium"`. Prefer explicit project overrides over global edits to isolate the effect. Keep all unrelated inherited settings. Project configuration relies on the existing trusted-project setting; do not change trust or bypass managed policy.
2. Retain `enabled = false` in the existing local-marketplace tables for exactly: `canva@openai-curated`, `figma@openai-curated`, `vercel@openai-curated`, `sites@openai-bundled`, `documents@openai-primary-runtime`, `spreadsheets@openai-primary-runtime`, `presentations@openai-primary-runtime`, `pdf@openai-primary-runtime`, and `template-creator@openai-primary-runtime`. These keys do not prove that a corresponding remote/workspace-managed installation is disabled. Do not invent remote keys, add version-specific skill paths, uninstall plugins, or change global settings. Document setting a local entry to true and opening a fresh task to re-enable that local installation, subject to higher-priority controls. Remote suppression unsupported at project scope is an accepted limitation, not a failed removal promise. The official configuration reference limits these plugin keys to local-marketplace controls: https://learn.chatgpt.com/docs/config-file/config-reference.
3. Keep mandatory policy in AGENTS.md. Add concise context-handling instructions there and practical configuration, log, and verification examples in docs/spec-driven-development.md. Replace only genuinely duplicated explanation with references; preserve every mandatory requirement, boundary, compatibility rule, approval gate, and final check. Do not edit generated skills or protected workflow scripts. Avoid moving mandatory rules behind optional reading.
4. Use filename/heading discovery before relevant spec sections, and exclude openspec/changes/archive from routine searches only. Historical investigation and validators still access archives. Reuse instructions already in context unless changed or missing after compaction. Store lengthy logs in an OS temporary directory, never committed; report command, exit code, summary, failures, and log location. Reuse passing evidence only for unchanged relevant inputs/toolchain/environment, and rerun on invalidation or explicit requirement. A shorter output never means suppressing a failed check.
5. Update `scripts/agent-context-efficiency.test.ts`, run directly with Bun without modifying the closed protected Moon task. Retain parsed exact-value/local-key and negative configuration tests; remove the synthetic JavaScript merge test because it does not exercise Codex configuration loading. Retain safeguard tests and add focused documentation tests plus negative fixtures for unconditional complete-plugin-removal claims and incorrect archival ordering. Keep coverage independent of real user-home configuration. Static tests prove the configuration/documentation contract, not desktop plugin availability or Codex inheritance.
6. Correct contributor guidance and task ordering without changing protected scripts: all implementation/content checks must pass before verification. Run the protected gate before archival and inspect its complete result. Only rejection naming this active change alone is the expected pre-archive state; never label it success. Any other failed substantive check or policy rejection blocks archival. After OpenSpec conformance verification, synchronize and archive only this change, then run the full protected gate and require success before completion. Final-gate failure remains a reported failure with the change not complete; do not fabricate an attestation or waive checks. This resolves the lifecycle ordering without relaxing final merge readiness.

## Verification and traceability

- Project-scoped defaults and reversible local overrides: parsed configuration tests, unchanged global-config hashes, and observed configuration/availability evidence with its source stated. Do not treat a JavaScript merge fixture as proof of the Codex loader.
- Focused context handling: documentation contract checks for targeted reads, historical exception, instruction refresh, concise failure reporting, and evidence invalidation. Agent execution behavior needs a manual audit because static tests cannot prove future model behavior.
- Preserved safeguards: assert retained OpenSpec approval, scenario coverage, verification/archive, ownership/compatibility, all existing required checks, and unchanged generated skills/protected CI files relative to the pre-change baseline.
- Fresh behavior evidence: distinguish a fresh desktop task's supplied skills/tools, a persistent task's catalog, standalone CLI prompt rendering, and static TOML checks. Do not infer remote desktop omission from CLI absence. Check model/planning and another project's unchanged behavior where observable; identify unavailable evidence explicitly. Inventory remote plugins still available as an accepted project-control limitation, not as successful removal. Do not create new user-owned tasks automatically for this check.
- Corrected lifecycle: documentation regression tests require implementation verification before synchronization/archival and protected-gate success afterward, with no claim that expected pre-archive rejection passes. Actual pre/post gate runs provide execution evidence.
- Optional usage comparison: use identical read-only prompts in fresh tasks before/after, same model and available measurement source. Record context/input, cached input, output/reasoning, and allowance separately when exposed. If a baseline or telemetry is unavailable, report that and do not estimate savings from file bytes.

## Risks / Trade-offs

- Lower planning effort can change planning quality: retain explicit escalation for difficult tasks without altering defaults globally.
- Disabled design/document tools need deliberate local re-enablement: document exact mechanism.
- Documentation tests can become wording-sensitive: use focused invariant checks rather than whole-file snapshots; manually review semantic preservation.
- Active unrelated work or platform/toolchain limits can block required checks: report exact failures; do not repair unrelated code, archive another change, skip gates, or claim completion.
- Added instructions themselves cost context: keep the efficiency section concise and avoid duplicating this design in always-loaded guidance.

## Migration Plan

After revised-artifact approval, record a fresh scoped baseline, update tests then guidance/config comments, and run required checks on a stable tree. Preserve prior results as history while refreshing current status. Verify conformance, synchronize/archive this change, then require the protected gate to pass. Missing evidence remains explicitly labeled; accepted remote-control limitations do not imply omitted required checks. Project settings apply to fresh tasks; an app restart may be required. Rollback affects only this change's edits. No data migration, application security boundary, public contract, or dependency direction changes.

## Open Questions

No implementation policy decisions remain. The user accepted documented remote-control limitations and explicitly approved the revised artifacts with "Approve". A newly active fix-font-draft-retention-and-unicode-breaks change was observed at implementation startup; it remains outside this task's scope and can block archive-only readiness.
