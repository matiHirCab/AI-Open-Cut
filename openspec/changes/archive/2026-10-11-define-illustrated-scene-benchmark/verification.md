# Verification: define-illustrated-scene-benchmark (#181)

2026-10-11. Approved proposal/delta/design/tasks preceded executable edits under recorded standing scoped authorization. Main baseline is `431a5e38bba964d780cfdbe0fe4ddf287ea21c58`; unmerged #85 is excluded.

| Dimension | Assessment |
| --- | --- |
| Completeness | Six scoped implementation/conformance tasks completed; three requirements and four scenarios covered |
| Correctness | Fixed inventory, independent native visual checks and attributable measurements passed |
| Coherence | Test-only core-owned fixture; existing native entry point, contracts, production behavior and protected policy preserved |

## Scenario traceability

| Scenario | Implementation and automated evidence |
| --- | --- |
| I1 initial slice / later targets | `tests/fixtures/illustrated-scene/recipe.json`, benchmark docs, golden `illustrated_scene::validate`, `illustrated_inventory_is_bounded_and_original`: seven roles, original image/text, fixed bounds and exactly three explicitly unimplemented target inventories |
| I2 native evidence | `Fixture::new` uses public import/edit APIs; `conformance` and `capture` exercise original native shapes/text/media at 0/500/966 ms and range [0,100) ms; probe verifies 1920×1080 and three range frames; eight independent color points plus white glyph presence/layout diagnostics; state equality before/after and reopen |
| I3 drift / required dependencies | `illustrated_inventory_rejects_workload_and_oracle_drift` rejects counts, overlong text, duplicate initial/target roles, overflowing boxes and changed expected colors; blank output fails the pixel verifier; required tools reuse fail-closed native configuration and exact font hash. Wrong container font was actually rejected before using the reviewed bundled font |
| I4 identity / timings / targets | Three fresh Renderer and three repeated frame/range calls per intent plus one warm-up; every output independently verified; source/driver/recipe/runtime binary and host/tool/font identities in `docs/verification/illustrated-scene-benchmark/reference-observation.json`; artifact hashes retained. Post-capture targets derive from 2× largest measured request, rounded to whole seconds: frame 5000 ms, three-frame range 8000 ms. Portable CI timing remains report-only |

## Local checks

All commands completed successfully except the explicitly expected pre-archive policy rejection below. Full uncommitted logs are `/tmp/issue181-*.log`.

- `cargo fmt --check --all`: `fmt-final.log`, exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: `clippy-final2.log`, exit 0; no source warnings. Existing dependency future-incompatibility notice is unchanged.
- `cargo test --workspace`: `workspace.log`, exit 0; 1519 passed counts across 77 result sections, 12 existing ignored cases. Test-only validator review changes were subsequently covered by final release focused tests; production and other suite inputs remained unchanged.
- `cargo test --release -p opencut-editor-core --lib illustrated_inventory`: `inventory-final.log`, 2 passed, exit 0 on final driver, including duplicate target roles/u64 overflow.
- Explicit native `cargo test --release -p opencut-editor-core --lib renderer::golden::native_golden_render_conformance -- --exact --nocapture`: `native-final.log`, exit 0. Existing six feature suites, new illustrated fixture and unchanged canonical capture all passed; illustrated work 43.076 seconds, entire entry point 265.58 seconds. Report/retained artifact paths are in benchmark docs.
- Bridge `bun run typecheck` / `bun run lint`: `typecheck.log` / `lint.log`, exit 0.
- `bun run test --no-file-parallelism`: `unit.log`, 869 passed; 15 existing opt-in skips in 10 files; exit 0.
- Unchanged canonical `bun run contracts:check --no-file-parallelism`: `contracts.log`, all required Rust sections and 607 bridge tests in 38 files passed, exit 0. Forwarded Vitest flag serializes files without removing assertions.
- `bun run test:integration --no-file-parallelism`: `integration.log`, 36 passed, exit 0.
- `bun run test:smoke`: `packaged.log`, default packaged smoke 33 passed, exit 0.
- Hermetic worker runner: `python.log`, 12 unittest + 12 pytest passed, exit 0.
- Pinned strict all-spec validation: `strict.log`, 64 items passed including this active change. Final pre-archive protected Moon command also passed all 64 strict items and policy/bootstrap tests, then rejected only `define-illustrated-scene-benchmark` as unarchived (`prearchive-gate-final.log`, expected exit 1). This is not a successful protected gate.

Container process tests used an external Linux subreaper wrapper because PID 1 does not reap orphaned adopted children. It changes no repository code, assertion or deadline. Native release reference capture ran with no concurrent repository checks.

## Review and limitations

Reviewed the retained frame visually against the declared layout. The primitive/card and target inventories are new independent benchmark evidence; existing goldens, raw digest pins, workflows, renderer guards and canonical report schema were not changed. No public/persisted contract, migration, production dependency or desktop/provider work is introduced.

Reference medians: fresh frame 2231.80 ms, repeated frame 1111.03 ms; fresh three-frame range 3700.23 ms, repeated range 2823.41 ms. Source production code is main; added test-only driver, recipe and executing binary have exact hashes verified against the captured record. Fresh means new Renderer, not flushed OS caches; repeated permits existing cache reuse and is not isolated-stage evidence. Peak process-tree memory for this separate fixture, full-duration export, animated/dense final targets and installed desktop runtime readiness were not measured. Interior samples/layout/glyph presence are narrower than full-image goldens. No universal timing guarantee follows from three samples.

No critical conformance issues remain. Sync plan: append only the three accepted added requirements to `render-regression-fixtures`, preserving existing content; archive this completed change. Post-archive protected validation, publication and exact-head CI are still pending at this report's creation and must be recorded after execution in the PR/external delivery checkpoint. No merge, auto-merge or deployment.
