## 1. Stage Model and Process Instrumentation

- [x] 1.1 Add crate-private typed renderer stage observations with finite non-negative durations, explicit work counts, intent identity, stage-definition version, and non-additive semantics.
- [x] 1.2 Instrument evaluation, zero-or-more native raster preparation, resource/filter-plan construction, and end-to-end production execution at their editor-core owning boundaries without changing public renderer results or errors.
- [x] 1.3 Build bounded decode-only and composite-to-null benchmark commands from the production `RenderPlan`; prove input order, source intervals, filter script, mappings, intent interval, and path safety match the production command.
- [x] 1.4 Add injected success/failure tests proving deterministic stage ordering, zero-work raster reporting, cleanup, and no downstream observations or publication after invalid input, missing references, revision conflicts, or process failure.

## 2. Benchmark Report and Fixture

- [x] 2.1 Replace schema-2 performance data with a strict schema-3 report containing environment identity, stage-definition identity, per-intent evaluation/raster/filter/decode/composite/encode/end-to-end measurements, work counts, aggregation rules, and peak process-tree memory.
- [x] 2.2 Validate exact frame/range/export intent coverage, finite non-negative values, coherent work counts, declared units, one warm-up, three measured samples, median timings, maximum memory, process-tree scope, and non-additive stage semantics.
- [x] 2.3 Run benchmark sampling only after the measured captures agree under the existing semantic, filter-graph, frame, audio, and timing conformance rules; retain report-only behavior with no performance threshold.
- [x] 2.4 Capture and atomically install a reviewed immutable golden generation with the schema-3 report, updated hashes/revision, and no machine-local paths or unrecognized residue.

## 3. Lifecycle, Documentation, and CI

- [x] 3.1 Extend deterministic fixture tests for success, invalid input, missing references, stale revision, undo/redo, and reopen behavior, proving benchmark observation never mutates project state or history.
- [x] 3.2 Update renderer fixture documentation with stage definitions, overlap/non-additivity, zero-work semantics, commands, environment comparison identity, atomic update review, and interpretation guidance.
- [x] 3.3 Update the motion-graphics implementation plan and required Linux artifact validation for schema 3 while retaining explicit FFmpeg, FFprobe, and font dependency gates.

## 4. Verification and Completion

- [x] 4.1 Run focused stage command/model/report validation, failure-path, native golden conformance, lifecycle, and temporary recapture tests with explicit rendering dependencies.
- [x] 4.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [x] 4.3 Run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, and `bun run test:smoke` from `apps/agent-bridge`, plus relevant hermetic Python worker tests.
- [x] 4.4 Run `moon run openspec-validate`, `git diff --check`, and inspect the diff for unintended public/persisted contract changes, unsafe paths/resources, generated workspaces, or unrelated edits.
- [x] 4.5 Apply `$openspec-verify-change`, resolve every mismatch, and archive with `$openspec-archive-change` so the accepted delta updates the living specification.
