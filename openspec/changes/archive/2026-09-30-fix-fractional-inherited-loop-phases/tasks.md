## 1. Approval and failing regressions

- [x] 1.1 Obtain and record explicit approval of proposal, all three delta specs, design and tasks before implementation. Preserve both earlier archives and unrelated worktree changes.
- [x] 1.2 Add independent scalar/backend regressions for fractional repeat interiors and seams, ping-pong turns, finite exhaustion and nonzero first-key offsets. Map to Deterministic item-local loop evaluation / Preserve fractional repeat phase / Preserve fractional turns and finite completion.
- [x] 1.3 Add the 0.995-rate native four-intent parent reproduction plus representative nested component/stagger/signed-copy phases and independent expectations. Confirm failure before fixing. Map to Independent fractional visual loop conformance / Reproduce the fractional parent seam across intents / Compose nested fractional phases.

## 2. Core correction and documentation

- [x] 2.1 Correct fractional visual loop mapping in editor-core animation/render planning with explicit precision selection; reuse existing curve compilation and endpoint/clamping rules, preserve integer and independent audio behavior. No transport rules or new dependency edges. Map to all fractional loop scenarios and Preserve existing integer and independent audio sampling.
- [x] 2.2 Retain passing integer-loop, zero-offset, curves, bounds, audio, controller-delay, aliases, draft/history/migration and contract regressions; verify no publication/error/schema drift. Map to Preserve compatibility and accurate verification and Shift a zero-offset ranked controller source.
- [x] 2.3 Update current animation/inherited/repeater guides and ADR 0004's stored-versus-derived clock wording; correct the living repeater scenario through approved delta synchronization only. Preserve historical archives. Map to Preserve compatibility and accurate verification and corrected Deterministic additional-copy semantics.

## 3. Verification and lifecycle closeout

- [x] 3.1 Run cargo fmt --check --all; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace -- --test-threads=1. Capture complete external logs and resolve failures.
- [x] 3.2 With compatible FFmpeg/FFprobe 7.1.1, pinned DejaVu Sans and OPENCUT_GOLDEN_REQUIRED=1, run cargo test -p opencut-editor-core --lib native_loop -- --nocapture and cargo test -p opencut-editor-core --lib native_inherited_timing -- --nocapture. Include new fractional fixtures and existing native integer/curve/audio evidence; required absent tools/font must fail explicitly. Do not update canonical goldens or tolerances.
- [x] 3.3 From apps/agent-bridge run bun run typecheck; bun run lint; bun run test:unit; bun run contracts:check; bun run test:integration; bun run test:smoke. From root run the existing hermetic Python interpreter with PYTHONPATH=apps/kokoro-tts on -m unittest apps/kokoro-tts/test_worker.py. Capture full external logs; reuse unchanged valid evidence only as AGENTS.md permits.
- [x] 3.4 Run pinned bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive and pre-archive moon run root:openspec-validate. Inspect complete results; only this active change may explain protected-gate rejection. Use $openspec-verify-change, map every changed scenario to actual tests and check evidence, and resolve mismatches before closeout.
- [x] 3.5 Use $openspec-sync-specs and $openspec-archive-change on this verified follow-up only; rerun moon run root:openspec-validate and pinned strict all-spec validation and require both to pass. Report unresolved checks and limitations accurately; no commits or pushes.
