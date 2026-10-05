## 1. Approved correction

- [x] 1.1 Read living mask conformance and approved50 requirements, preserve approved51 planning, compare actuallocal/pinnedCI/toolchain diagnostic evidence, validate these artifacts and obtain explicitparent/independent specification approval before test-source edits.
- [x] 1.2 Replace only the two fixed-width iterators and PCM array conversion; retain every length/witness/threshold/identity assertion and add no suppression or unrelated behavior.

## 2. Stable verification and review

- [x] 2.1 Run cargo fmt --check --all, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace, plus all16 existing mask_models native tests under actual required tool flags. Record full command exits/counts and evidence limitations; reuse unrelated native gates only if exact relevant inputs remain unchanged.
- [x] 2.2 Run bridge bun run typecheck/lint/test:unit/contracts:check/test:integration/test:smoke and root bun run apps/agent-bridge/scripts/run-python-tests.ts; preserve command exits and actual execution counts.
- [x] 2.3 Run pinned strictall validation and prearchive protectedMoon (only ownactive rejection expected), apply openspec-verify-change and obtain independent final diff/conformance approval with findings resolved.
- [x] 2.4 Apply openspec-sync-specs/openspec-archive-change only this change and pass unchanged final protectedMoon and strictallspec gate; retain exactspec approval and scenario evidence.

## Parent-owned post-archive publication and continuation

After final archived policy success, parent commits/pushes to existing PR143 branch, attaches it and verifies all required remoteCI on that exact commit. ExactheadCI is a required continuation gate, not a prearchive implementation task. After it passes, parent advances51 from verified50 and reconciles the preserved approved planning SHA/unchangedsemantic pins before implementation. Continue51–54 stacked draftPRs in dependency order. Do not merge/deploy.
