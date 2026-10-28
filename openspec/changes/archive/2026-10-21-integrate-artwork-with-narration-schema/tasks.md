## 1. Approval and refreshed integration inventory

- [x] 1.1 Obtain explicit parent/reviewer approval of proposal/design/delta/tasks, including schema48 and rejection of unpublished artwork47 dialect. Record approval and artifact hashes before implementation.
- [x] 1.2 Refresh live main and #229 ownership/head/draft state, inspect applicable living narration/package/persistence/media specs and canonical contract ownership, preserve all unrelated work, then merge main into the existing feature branch without committing or selecting one side wholesale.

## 2. Canonical fixtures and core migration

- [x] 2.1 Freeze independently authored combined migration/retained-history/invalid-field fixtures and exact main47 catalog pins before consumers. Preserve every original pin/capture and package46 marker; define integration inverse order and missing/duplicate/unrelated-drift negative controls.
- [x] 2.2 Reconcile schema48 model/deserialization/migrations and all core Asset/Project initializers. Add native schema1..46/main47/current48/future49, premature artwork47, malformed narration47, retained invalid snapshot and atomic/recovery regressions. Preserve core transaction/resource ledger and both feature modules.
- [x] 2.3 Add a combined native scenario preserving exact existing narration through artwork prepare/commit/reopen/undo/redo and stale revision failure. Run `cargo test -p opencut-editor-core --test artwork_preparation --test narration_blocks --test narration_profiles --test speech_alignment --test channel_style_packs` plus focused model/migration/store tests in the workspace run. Keep the existing raster/ownership/conflict cases unchanged.
- [x] 2.4 Preserve package v1 componentSchemaVersion46 and ordinary capture/import/apply. Reject selected derived artwork with VALIDATION_FAILED in the core package owner before mutation; add unchanged-byte/library/history regression and successful ordinary selection in a project with unrelated derived artwork. No package format expansion.

## 3. Governed transport and catalog integration

- [x] 3.1 Reconcile thin headless operations, TS project/asset schema union, canonical current-schema markers and complete ownership/CODEOWNERS union; retain narration required fields, both feature operations exactly once and old aliases. Preserve unchanged workflows and production deletion code.
- [x] 3.2 Implement explicit guarded integration inverse layers for exact raw/compact/MCP-expanded predecessors; compose older feature inverses at their original schema boundaries. Run focused `bunx vitest run --config vitest.unit.config.ts tests/contracts.test.ts tests/artwork.test.ts tests/artwork-cli.test.ts tests/narration-blocks.test.ts tests/narration-blocks-workflow.test.ts tests/narration-profiles.test.ts tests/channel-packs.test.ts` from apps/agent-bridge; verify expected producer filenames against refreshed main and preserve test bodies.
- [x] 3.3 Synchronize operator/current-schema documentation with schema48 while retaining historical archived approvals, package markers, actual platform limits and no GUI-readiness claim.

## 4. Required conformance and evidence

- [x] 4.1 Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and explicit `cargo test -p opencut-desktop review_tests::desktop_review_native_frame_audio_range_history_and_reopen -- --ignored --exact`. Capture full logs outside Git; report every failure/ignore and exact required explicit native execution separately.
- [x] 4.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke`. Run `python3 scripts/verify-artwork-preparation.py` and separately `python3 scripts/verify-artwork-preparation.py --hermetic` from root; distinguish genuine media tools from hermetic evidence. Do not manually reproduce or investigate Windows disposal behavior; unchanged required suites may report their existing failures.
- [x] 4.3 Run `(cd apps/kokoro-tts && python3 -m unittest test_worker.py)`, `(cd apps/faster-whisper && python3 -m pytest test_worker.py)`, `bun test scripts`, and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`. Install only pinned tools/test dependencies into isolated writable paths; report unavailable platform/tool evidence without modifying gates.
- [x] 4.4 Run `moon run root:openspec-validate` before archive and inspect full output; only this active change's rejection is expected and is not a pass. Use `$openspec-verify-change` to map every requirement/scenario/task to actual implementation and passing required evidence. Required remote platform/native failures block completion independently; no test, timeout, diagnostic or workflow weakening.

## Post-verification lifecycle obligations

After all required implementation checks pass, use `$openspec-sync-specs` and `$openspec-archive-change` only for this approved change. Run protected Moon and pinned strict all-spec validation again; require genuine success. Preserve original artwork/narration archives as historical approvals rather than rewriting them. Do not archive unrelated work to clear policy.

Refresh remote main/source ownership and refs before publication. Commit only verified integration as a descendant of both source and main, push to existing PR229 branch without force, and preserve observed draft=false. Do not merge/deploy/close issues or edit other PRs. Observe unchanged exact-published-head required CI to terminal and report the head, links, full failures/skips/pending evidence and outstanding genuine CODEOWNER acceptance separately. Never count a pending future action as passed or add a source commit only to loop exact-head evidence.
