## 1. Approved specification and governed contracts
- [x] 1.1 Validate this issue-scoped proposal/design/deltas/tasks using bunx @fission-ai/openspec@1.5.0 validate save-versioned-narration-profiles --strict --no-interactive and record delegated approval.
- [x] 1.2 Define canonical narration library/resolved-request fixtures and synchronize ownership, schema46, headless operations, persisted provenance and MCP catalogs before completing consumers. Owner review remains mandatory in the subsequent delivery gates.

## 2. Core domain, persistence and migration
- [x] 2.1 Implement bounded validated profile models, atomic shared library create/read/list/update and optimistic versions under the root lock, with failure-path tests.
- [x] 2.2 Implement pinned resolution and complete-field override precedence with Unicode/overlap/duplicate/boundary tests.
- [x] 2.3 Implement schema46 attribution validation and current/retained-history migrations; test exact reopen/undo/redo, independence from library and atomic failure.

## 3. Thin transports and provider-neutral workflows
- [x] 3.1 Add typed headless profile operations and contract tests over core behavior.
- [x] 3.2 Add MCP profile tools and explicit profile selection to existing speech generation; retain exact resolved snapshot through preview/retry/commit/regenerate and preserve worker protocol and legacy requests. Add deterministic cancellation-during-preflight and FIFO-successor coverage before completing acceptance.
- [x] 3.3 Test actual source MCP apply/generate with fake provider, two-project reuse, unavailable voice/readiness, stale versions/revisions, provider mutation and retained artifact cleanup; cover packaged path.
- [x] 3.4 Document parameters, limits, ownership, precedence, failures and runtime readiness; provide scenario traceability.

## 4. Required verification and delivery
- [x] 4.1 Run cargo +1.97.0 fmt --all -- --check; cargo +1.97.0 clippy --workspace --all-targets -- -D warnings; cargo +1.97.0 test --workspace with full uncommitted logs.
- [x] 4.2 In apps/agent-bridge run bun run typecheck, bun run lint, bun run test:unit, bun run contracts:check, bun run test:integration and the complete original packaged smoke suite; attempt ordinary bun run test:smoke, and after independent native compiler readiness failure use the documented supported pinned cross-build with the existing assembler/integrity verifier. Preserve the failed bootstrap and require ordinary native published-head CI. Run relevant hermetic Python workers and root policy/agent-context suites.
- [x] 4.3 Record local native/platform readiness and limitations without weakening any gate; pass pinned strict all-spec validation and inspect prearchive moon run root:openspec-validate, accepting only rejection naming this active change. All protected native/platform/GUI and foundation jobs remain mandatory at the exact published head.

## Subsequent delivery gates
After every local implementation checkbox passes, use openspec-verify-change for independent requirement/design/task/code/test conformance and resolve every mismatch. Apply openspec-sync-specs and openspec-archive-change for this change only, then pass final protected Moon and strict all-spec validation. Commit/push scoped files, open and attach the cumulative draft PR against main (PR223 has been merged), request designated CODEOWNER review, and require every protected native/platform/GUI, ordinary native package bootstrap and foundation CI job at the exact published head before reporting completion. Report tests, SHA, PR, owner review and exact-head CI, then proceed to the next ready issue. These delivery gates remain mandatory after archival; they are sequenced separately so a future commit/PR does not falsely appear completed before the archive prerequisite. No merge/deployment.
