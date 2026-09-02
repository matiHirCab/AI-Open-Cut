## 1. EvaluatedScene contract tests

- [x] 1.1 Add focused editor-core tests that construct current flat media, text, solid-color, and rectangle timelines and assert owned renderer-neutral instructions, half-open timing, stable track/item ordering, logical first-use resource order, and source-project immutability.
- [x] 1.2 Add focused audio tests for audio-only and audiovisual media, mute handling, gain automation, fades, voiceover intervals, and ducking facts.
- [x] 1.3 Add failure tests for `ASSET_NOT_FOUND`, non-finite values, invalid/overflowing intervals, and boundary-plus-one cases for every named scene complexity limit, asserting no renderer, filesystem, or artifact adapter invocation.
- [x] 1.4 Add architecture tests that reject persisted-record references, filesystem paths, network/resource locators, renderer expressions, FFmpeg/backend types, prepared files, and artifact destinations in the evaluated-scene module.

## 2. Editor-core scene model and evaluator

- [x] 2.1 Add the crate-private owned `EvaluatedScene` header, layer-order key, logical media-resource table, closed flat visual instruction enum, audio instructions, transition facts, and backend-neutral animation/style value records.
- [x] 2.2 Implement pure evaluation of visible current flat items, deterministic first-use resource ordering, stable layer ordering, logical asset resolution, and resolved audio/ducking facts without mutating or serializing project state.
- [x] 2.3 Add named inclusive constants for 4,096 visual layers, 4,096 logical media resources, 4,096 audio layers, and 10,000 keyframes per property channel; enforce finite values and checked non-empty half-open intervals with existing stable errors before downstream I/O.
- [x] 2.4 Extract only behavior-preserving shared helpers needed by both the new evaluator and the existing production planner; keep all frame/range/draft/export call sites on their current path for issue #13.

## 3. Compatibility and documentation

- [x] 3.1 Update ADR 0004 and the motion-graphics implementation documentation with the concrete scene ownership, field categories, explicit flat limits, logical-resource rule, and issue #13 routing boundary.
- [x] 3.2 Confirm with regression tests that schema version 6, project serialization, deterministic reopen, retained undo/redo snapshots, headless/MCP contracts, capability reports, stable-error catalog, and existing simple rendering behavior are unchanged.
- [x] 3.3 Confirm `contracts/motion-graphics-v1.json` remains `fixture_only` and document why no contract fixture, migration, batch-alias, or capability update applies to this process-local model.

## 4. Verification and closure

- [x] 4.1 Run `moon run openspec-validate` from the repository root. (`moon` was unavailable, so the repository-documented pinned `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` fallback ran successfully: 13 passed, 0 failed.)
- [x] 4.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from the repository root.
- [x] 4.3 Record that TypeScript, Python, MCP integration, migration, and packaged-smoke suites are unaffected unless implementation review reveals a touched surface; if one is touched, run and record its repository-required checks before closure. (No TypeScript, Python, MCP, persistence/migration, provider, integration, or packaging surface changed; workspace contract, compatibility, migration, renderer parity, headless protocol, and error-catalog tests passed through `cargo test --workspace`.)
- [x] 4.4 Use `$openspec-verify-change`, resolve every requirement/design/task/test mismatch, obtain required review, and archive with `$openspec-archive-change` so the accepted delta updates the living specification.
