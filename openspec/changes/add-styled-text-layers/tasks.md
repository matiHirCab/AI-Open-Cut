## 1. Approval and canonical fixtures

- [x] 1.1 Record explicit approval of proposal, design and all delta requirements before implementation.
- [x] 1.2 Add `styled-text-layers-v1` valid/invalid fixtures, Unicode/cluster reference vectors, ownership entries and schema-20 migration fixtures. Trace every delta scenario to planned automated tests; preserve historical contracts. (Governed additive styled text contracts; Grapheme indexed style spans; Atomic schema 20 styled text activation.)

## 2. Core model, validation and persistence

- [x] 2.1 Add optional closed spans/layer types and pinned grapheme segmentation; test Unicode boundaries, cross-run graphemes, style precedence, malformed values and every inclusive bound. (Grapheme indexed style spans; Bounded ordered text paint stacks.)
- [x] 2.2 Add source-version validation and atomic schema-20 migration for current state, mixed undo/redo, components/slots and retained drafts. Test future versions, invalid sources, unchanged reopen and every persistence fault phase. (Atomic schema 20 styled text activation.)
- [x] 2.3 Carry styles through add/update, batch aliases, draft replacement, slots and lifecycle edits. Test standalone/batch/draft success, rollback, missing references, revision conflicts, locked/incompatible tracks, undo/redo and reopen without font lookup. (Reversible indexed text editing.)

## 3. Core evaluation and rendering

- [x] 3.1 Map grapheme ranges and effective styles into evaluated text and complete shaped clusters without breaking paint-only shaping; test bidi, ligatures, explicit false overrides and component/repeater/slot evaluation. (Grapheme indexed style spans; Reversible indexed text editing.)
- [x] 3.2 Implement ordered fill/stroke/shadow rasterization, union masks, deterministic Gaussian blur and checked padded bounds/work limits. Test fractional widths, zero blur, signed offsets, overlap order, alpha, transforms, empty/inherited stacks and allocation-free rejection. (Bounded ordered text paint stacks; Shared bounded styled text rasterization.)
- [ ] 3.3 Add independently reviewed visual fixtures and shared frame/range/draft/export parity tests including audio, animation, transforms, missing/damaged resources and source-font removal/reopen. Prove omitted-stack legacy lossless equality. (Styled text render intent parity.)

## 4. Governed adapters and documentation

- [x] 4.1 Update typed headless input/output unions and serialization fixtures; update bridge Zod schemas, capability/version negotiation and MCP surface fixtures. Keep semantic validation in core and add Rust/TypeScript parity plus MCP integration and packaged smoke scenarios. (Governed additive styled text contracts.)
- [ ] 4.2 Document grapheme indexing, cluster fallback, ordering/inheritance, coordinates, limits, schema migration and compatible simple operations with concrete standalone/batch examples. Obtain the designated contract CODEOWNER review and record evidence. (All delta requirements.)

## 5. Required verification and archival

- [x] 5.1 From repository root run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. Record complete logs, exit codes and scenario-to-test traceability in verification.md.
- [x] 5.2 From `apps/agent-bridge` run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke` and `bun run scripts/run-python-tests.ts`. Report any missing environment/dependency or skipped required suite as a blocker.
- [ ] 5.3 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `moon run root:openspec-validate`. Before archival only rejection exclusively caused by this active change is expected; other failures, including unrelated active changes, block archival and must be reported without changing unrelated work.
- [ ] 5.4 Use `$openspec-verify-change` and resolve every requirement/design/task/code/test mismatch. Every normative requirement needs automated evidence or a documented technical impossibility justification, not merely compilation.
- [ ] 5.5 Use `$openspec-sync-specs` and `$openspec-archive-change` after passing required implementation checks and conformance verification. Then rerun `moon run root:openspec-validate` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; both must pass before declaring completion.





## 6. Approved review corrections

- [x] 6.1 Add failing pixel-equality and work-budget regressions for invisible explicit spans and overridden colors, including mixed faces/colors and RTL.
- [x] 6.2 Share the original legacy painter with mixed segments and group explicit stacks by effective paints and face; verify real stack replacements still change output.
- [x] 6.3 Repeat both reproductions through native previews and refresh required verification evidence. Independent visual/CODEOWNER review and archival remain separate gates.

## Unchecked-task reassessment — 2026-09-18

All five unchecked tasks remain open. Task 3.3 lacks independent visual review, and the full native CI golden suite fails the rule-card semantic snapshot comparison at `renderer/golden/rule_card.rs:334` (generated plans add `paint_layers: None`). Task 4.2 lacks designated CODEOWNER review; PR #120 reports no reviews. Task 5.3 still fails the protected archive-only gate for both active changes. Task 5.4 cannot close until the newly observed golden mismatch and required review evidence are resolved. Task 5.5 has not been performed. Existing passing focused raster/native checks do not establish full native golden success.

## 7. Native golden compatibility correction

- [x] 7.1 Preserve legacy semantic debug plans when optional paints are absent, retain explicit paints in debug output, and test original/moved reviewed rule-card plans without FFmpeg. Run the full native golden suite and affected Rust checks. (Styled text render intent parity; approved user request to resolve archival blockers.)

The rule-card mismatch recorded in the preceding reassessment is resolved by task 7.1. Full native golden conformance and the complete Rust workspace now pass. Tasks 3.3 and 4.2 still require independent visual and designated CODEOWNER review; 5.3–5.5 remain open pending review/conformance completion and protected synchronization/archival gates.
