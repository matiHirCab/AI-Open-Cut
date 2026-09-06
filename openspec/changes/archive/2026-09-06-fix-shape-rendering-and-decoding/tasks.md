## 1. Approval and reproductions

- [x] 1.1 Record explicit approval of these concrete proposal, design, delta specifications and tasks before implementation.
- [x] 1.2 Add failing core/native regressions for Scale-aware shape coverage and Closed bounded shape geometry: magnified ellipse equivalence and fractional anchor equivalence.
- [x] 1.3 Add failing raw-string decoding regressions for Canonical JSON representation enforcement, including nested identical and invalid-first/valid-last duplicate keys.

## 2. Core evaluation and rendering

- [x] 2.1 Implement idempotent effective-magnification density using the composed transform and interval scale maximum; cover nonuniform, skewed, inherited and component transforms.
- [x] 2.2 Implement density-aware bounds, raster coverage, stroke/dash metrics and inverse-mapped paint sampling; compensate static and animated source sampling without changing authored placement.
- [x] 2.3 Correct fractional anchor bounds and restrict one-pixel fallback to zero-extent path axes; cover lines and degenerate paths.
- [x] 2.4 Check density-adjusted per-shape and aggregate allocation budgets before allocation/writes; test exact boundaries and no-side-effect failure for each render facade.
- [x] 2.5 Extend native shape conformance with gradients, strokes, inherited/animated transforms and matching preview/range/export samples; retain all legacy golden references and tolerances.

## 3. Core decoding and compatibility

- [x] 3.1 Introduce duplicate-preserving internal buffered decoding and replace lossy shape-bearing operation/project buffers; preserve numeric types, optional/null semantics and migration preprocessing.
- [x] 3.2 Audit and cover batch, draft, component, current project and retained history paths; prove structural rejection preserves revision, state, history and atomicity through real headless requests and core persistence tests.
- [x] 3.3 Preserve valid schema 14 and supported legacy migrations, stale revisions and canonical serialization; add governed consumer regression evidence without changing wire/catalog shapes.
- [x] 3.4 Update shape documentation and scenario-to-test traceability for all three findings; keep original archive and unrelated working-tree edits intact.

## 4. Required checks

- [x] 4.1 Run root `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` after focused failing regressions pass.
- [x] 4.2 Run `cargo test --release -p opencut-editor-core native_golden_render_conformance --lib -- --nocapture` with `OPENCUT_GOLDEN_REQUIRED=1`, `OPENCUT_FFMPEG_PATH`, `OPENCUT_FFPROBE_PATH`, and the checked-in DejaVuSans font via `OPENCUT_TEST_FONT_PATH`; record tool paths and results.
- [x] 4.3 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, and `bun run contracts:check`.
- [x] 4.4 After Rust checks finish, from apps/agent-bridge run `bun run test:integration` and `bun run test:smoke`; from apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts`.
- [x] 4.5 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `git diff --check`; record every result, required skip/failure and scenario mapping in verification.md.

## 5. Verification and archival

- [x] 5.1 Use openspec-verify-change to compare requirements, design, implementation and automated evidence; resolve every discrepancy before declaring completion.
- [x] 5.2 Use openspec-archive-change to synchronize deltas and archive this verified follow-up without modifying the original archive.
- [x] 5.3 Run `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` after archival; report any failed or skipped required check as blocking completion.
