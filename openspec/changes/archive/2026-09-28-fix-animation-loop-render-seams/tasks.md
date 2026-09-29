## 1. Reproduce and cover render boundaries

- [x] 1.1 Add a failing `render_plan` native scalar regression on supported FFmpeg 6 and 8 for a nonzero first keyframe, held segment, exact 12/13/14 ms repeat seam, finite repeat exhaustion, ping-pong turn and round-trip seam, and reflected Bézier/spring samples. Compare each backend value to the core integer sampler (rendering-export: exact seams, held seam, reflected curves).
- [x] 1.2 Add decoded frame, nonzero-start audiovisual range, materialized draft, and export comparisons at equivalent absolute timestamps for finite and infinite visual/audio loops; retain current timing and visual/audio tolerance (rendering-export: later-cycle samples, held seam, unlooped output).

## 2. Correct the owning render translation

- [x] 2.1 Update `crates/editor-core/src/render_plan.rs` so FFmpeg selects loop phase and finite exhaustion in quantized integer milliseconds with the same half-open and reverse rules as `animation::map_loop_time`; preserve pre-loop hold, parameterized curve results, unlooped expressions, and canonical preflight failures. Make 1.1 and 1.2 pass (rendering-export and animation-loops).
- [x] 2.2 Preserve the reviewed unlooped golden semantic plan by omitting the absent loop field from `EvaluatedKeyframe` debug output, with a focused legacy-shape test; retain the field for looped keyframes (rendering-export: preserve unlooped rendering).

## 3. Close issue #41 evidence gaps

- [x] 3.1 Add a serialized schema-24 project with nonempty channel current/undo/redo state to the editor-core migration tests; verify one schema-25 generation, unchanged unlooped evaluation, retained history, failed-source atomicity, and deterministic reopen (existing project-persistence requirement).
- [x] 3.2 Add reviewed maximum-count and malformed loop examples to `contracts/animation-channels-v1.json` and have Rust and TypeScript tests consume the same canonical examples; verify MCP schema shape and unchanged protocol/capability/error metadata with `bun run contracts:check` in `apps/agent-bridge` (existing motion-graphics-contracts requirement).

## 4. Verify, synchronize, and archive

- [x] 4.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the focused native loop tests using FFmpeg 6 and 8. Run the relevant render-parity commands in `docs/ci-parity-gates.md`; capture complete logs and report any platform unavailable locally.
- [x] 4.2 Run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, `bun run test:smoke`, and `bun run contracts:check` from `apps/agent-bridge`. Kokoro Python worker tests are not affected because no worker/provider input, output, inference, or files change; record this scope decision.
- [x] 4.3 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and pre-archive `moon run root:openspec-validate`; resolve failures other than the expected active-change inventory rejection. Use `$openspec-verify-change` to resolve every spec/design/task/test mismatch.
- [x] 4.4 Synchronize the verified delta with `$openspec-sync-specs`, archive with `$openspec-archive-change`, then rerun strict all-spec validation and `moon run root:openspec-validate` until both pass.
