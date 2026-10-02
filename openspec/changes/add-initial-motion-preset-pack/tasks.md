## 1. Approval and dependency checkpoint

- [ ] 1.1 Obtain explicit approval of the exact roadmap-oriented parameter/phase table, opacity flash/positional shake, enabled MotionBlur collision behavior, loop defaults/invisible radar reset, schema30 and blur-sensitive provenance rules; record attributable approval. Do not implement before this task completes.
- [ ] 1.2 Resolve issue45 review concerns through parent; verify accepted prerequisite head, rebase this branch and preserve the unmerged dependency status until #45 actually closes. Parent must confirm scope before any issue46 publication.

## 2. Canonical contracts and persistence

- [ ] 2.1 Add manually reviewed canonical positive/negative expansion, provenance, parameter, collision, status and schema30 fixtures; update ownership/affected catalogs and cross-language expectations before consumers. Preserve scalar outputs and old meaning.
- [ ] 2.2 Core: add strict union shapes/object-only duplicate-preserving decoding (including nested MotionBlur arrays/duplicates), property-membership provenance validation and schema30 migration; test current/components/undo/redo, premature/future schemas, malformed state and every journal fault phase. Trace to atomic schema30 and descriptive provenance requirements.

## 3. Core compilation and mutation

- [ ] 3.1 Core: add pure bounded pack vector expansion with checked integer times, closed endpoint validation and independently fixed sample oracles for all entries and boundary cases.
- [ ] 3.2 Core: implement atomic multi-identity collision/replace preserving positions and appending output order; cover partial channel/blur collision, legacy collision, enabled blur/raster budgets, loops, target/reference/locks, revision/aliases, final safety budgets, existing drafts and bytes.
- [ ] 3.3 Core: cover source lifecycle, raw setter/blur change/reconciliation/copies/split/duration edits, exact undo/redo and retirement/reopen without dispatch. Preserve scalar and midpoint regressions.

## 4. Transport and rendered conformance

- [ ] 4.1 Headless: update typed union/capability/status and raw-wire tests against canonical fixture, preserving codes/retryability for standalone and alias batches.
- [ ] 4.2 Bridge: update typed input/Zod/tool documentation/structural MCP catalog and digest; add old/new positive and negative contract tests plus all-five source/packaged smoke workflows. Keep semantics in core.
- [ ] 4.3 Core rendering tests: compare all-five compiled/manual primitives in frame/range/draft/export at key boundaries, odd times, loop seams/final finite cycles, invisible radar reset, slam shutter/flash/shake and fractional inherited clocks with native FFmpeg/FFprobe and approved font; retain visual/audio tolerances and read-only state.
- [ ] 4.4 Update user docs and exact requirement/scenario/test ledger; obtain @matiHirCab CODEOWNER review of canonical artifact and governed consumers before completion.

## 5. Required verification and acceptance

- [ ] 5.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; run focused pack, migration and existing scalar/midpoint tests. Capture full logs and required native coverage environment; do not count opt-out native cases as render evidence.
- [ ] 5.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke`, and `bun run scripts/run-python-tests.ts`. Provider behavior is unchanged; hermetic Python worker regression remains mandatory, no live provider needed.
- [ ] 5.3 Run required native render/cache/worker and all three rules-screen PR shards exactly as .github/workflows/bun-ci.yml requires; record optional weekly scope separately and do not equate local timings with GitHub budget evidence.
- [ ] 5.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; run `moon run root:openspec-validate` prearchive and inspect that its only expected rejection is this active change. Resolve every other failure.
- [ ] 5.5 Invoke $openspec-verify-change and resolve completeness/correctness/coherence gaps; obtain owner implementation acceptance. Then invoke $openspec-sync-specs and $openspec-archive-change only for this verified change.
- [ ] 5.6 Run postarchive `moon run root:openspec-validate`, strict all-spec validation and `bun --config=/dev/null --no-env-file run scripts/run-ci-policy.ts`; verify clean scoped commits and dry-run push. Publication requires parent scope confirmation, then track exact-head CI to terminal and protected duration audit. Do not merge/deploy or claim issue45 closed by this change.
