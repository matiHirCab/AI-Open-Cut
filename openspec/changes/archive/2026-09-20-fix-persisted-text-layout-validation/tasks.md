## 1. Artifact approval

- [x] 1.1 Obtain and record explicit approval of proposal, design, both delta specifications and this task list before implementation (2026-09-20: "yes").
- [x] 1.2 Record scenario-to-test traceability for P1-P3, E1-E3 and R1, retaining the original fourteen acceptance scenarios and prior corrective regressions as compatibility obligations.

## 2. Core persisted layout validation

- [x] 2.1 Add red P1/P2 regressions for current, hidden root, undo and redo layouts with negative/excessive tracking, unusable padded bounds and missing fitting bounds; assert exact errors and complete authoritative byte/inventory equality.
- [x] 2.2 Invoke canonical style validation for layout-enabled root text in the shared scope validator; preserve document validation and layout-absent behavior; pass P1/P2.
- [x] 2.3 Add R1 frame/range/export tests using existing fake process/artifact adapters, including invalid destinations; prove no inspection, workspace allocation, writes, publication or process execution.

## 3. Core structural decoding errors

- [x] 3.1 Add red E1/E2 matrices for null layout/nested fields, wrong types, unknown fields and enums through Project, History and retained add/update/component draft payloads during schema-20 migration and schema-21 reopen; compare all authoritative files.
- [x] 3.2 Add a dedicated strict TextStyle.layout deserializer and private anchored marker classification in CoreError's JSON conversion; strip the marker from public errors and preserve unrelated conversions.
- [x] 3.3 Verify propagation through buffered project and internally tagged operation/item decoding and add controls against substring-based or marker-like-input misclassification; pass E1/E2.
- [x] 3.4 Cover P3 valid legacy/advanced reopen, stale draft compatibility, unchanged layout-absent behavior, old-schema layout restrictions, future rejection and unrelated persisted error codes.

## 4. Transport and documentation

- [x] 4.1 Add native headless and MCP assertions for E3 exact INVALID_ARGUMENT and retryable false on persisted layout failures, with unrelated-error controls; exercise the relevant MCP workflow in integration and packaged smoke.
- [x] 4.2 Update supporting documentation and verification evidence with actual scenario mappings and check results; preserve public fields, schema version, catalogs, encoding and both prior archives.

## 5. Required implementation verification

Run all builds and executable-dependent suites sequentially. Use RUSTUP_TOOLCHAIN=1.97.0. For native tests configure OPENCUT_FFMPEG_PATH and OPENCUT_FFPROBE_PATH to the verified temporary FFmpeg 7.1.1 bin directory, and OPENCUT_TEST_FONT_PATH to crates/editor-core/tests/fixtures/fonts/DejaVuSans.ttf. Use the existing working temporary CLI cache for pinned OpenSpec commands. Capture full logs outside the repository and record every exit and failure.

- [x] 5.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace -- --test-threads=2` from the repository root.
- [x] 5.2 With native tools configured, run `cargo test -p opencut-editor-core native_rich_text_render_conformance -- --nocapture` and `cargo test -p opencut-editor-core --test font_resolution -- --nocapture`; verify native execution rather than unconfigured skips.
- [x] 5.3 From apps/agent-bridge run sequentially `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration` and `bun run test:smoke`.
- [x] 5.4 From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts`.
- [x] 5.5 Run `git diff --check`, `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `bunx @moonrepo/cli@2.3.3 run root:openspec-validate`; only rejection naming this active change is expected before archival, not a gate pass.

## 6. Conformance and archival

- [x] 6.1 Apply openspec-verify-change after passing implementation checks; resolve all code/spec/design/task/test mismatches and record limitations without treating prior evidence as proof.
- [x] 6.2 Apply openspec-sync-specs and openspec-archive-change to synchronize and archive this correction separately, preserving both previous archives.
- [x] 6.3 Require passing post-archive `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; record final evidence and report any unresolved failures explicitly.
