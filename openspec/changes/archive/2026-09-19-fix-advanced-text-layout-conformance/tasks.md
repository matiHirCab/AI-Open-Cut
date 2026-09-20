## 1. Approval and traceability

- [x] 1.1 Obtain explicit approval of proposal, design, both delta specs and this task list; record approval before editing implementation.
- [x] 1.2 Map corrective scenarios G1-G2, W1-W2, D1-D3, B1-B3 and R1-R2 and the original fourteen scenarios to concrete regression tests and independent expected results in verification.md.

## 2. Core logical geometry

- [x] 2.1 Add failing geometry regressions for G1-G2: negative shadow, stroke, overhang, fractional bounds, every anchor, rotation/scaling, animation and component/repeater placement; preserve exact legacy assertions.
- [x] 2.2 Carry private logical dimensions and raster-to-local offsets through evaluated text geometry and compose them consistently in static and animated sampling, including ancestor transforms; pass G1-G2.

## 3. Core retained draft validation

- [x] 3.1 Add failing D1-D2 schema-20 and schema-21 fixtures covering creation/update styles, invalid tracking, invalid bounds/padding and nested component text; snapshot authoritative file bytes and inventory.
- [x] 3.2 Extend canonical typed retained-draft traversal to reuse validate_text_style before publication, retaining existing document and paint validation; pass D1-D2.
- [x] 3.3 Establish D3 valid/stale/legacy draft compatibility and unchanged revision-conflict behavior without replaying drafts.

## 4. Core exact width arithmetic

- [x] 4.1 Add failing W1 dimension round-trip and W2 word/cluster, bidi, mixed-face, exact and narrowly smaller boundary tests with independent metric expectations.
- [x] 4.2 Implement advanced-only canonical accumulation and word-break width restoration; use logical widths for diagnostics/fitting while preserving bidi placement and exact legacy arithmetic; pass W1-W2.

## 5. Core shared work accounting

- [x] 5.1 Add B1 direct production-ceiling equality, excess and checked-overflow tests and B3 newline-only accounting tests without changing limits.
- [x] 5.2 Encapsulate the private budget with a test-only smaller limit; exercise actual expanded-scene sharing for B2 and pass boundary tests.
- [x] 5.3 Add R2 preflight tests proving rejection precedes destination inspection, artifact allocation and publication, including invalid destinations and unchanged inventories.

## 6. Render and transport conformance

- [x] 6.1 Extend native and bridge regression evidence for R1 with exact matching diagnostics and decoded frame/range/draft/export visual, audio and timing comparisons at existing tolerances.
- [x] 6.2 Verify standalone, batch aliases, components, slots, repeaters and typed headless/MCP parity; preserve exact legacy fixtures and all existing public fields, schema version, capabilities and encoding settings.
- [x] 6.3 Update supporting documentation and scenario-to-test evidence without modifying the original issue-35 archive; investigate and resolve mismatches rather than treating prior verification.md as proof.

## 7. Required verification on a stable tree

Use Rust 1.97.0 through RUSTUP_TOOLCHAIN. Set OPENCUT_FFMPEG_PATH and OPENCUT_FFPROBE_PATH to the verified temporary FFmpeg 7.1.1 binaries and OPENCUT_TEST_FONT_PATH to the pinned DejaVuSans.ttf fixture for native checks. Run builds and executable-dependent suites sequentially. Capture complete logs outside the repository, inspect failures and record each command and exit.

- [x] 7.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` from the repository root.
- [x] 7.2 Run `cargo test -p opencut-editor-core native_rich_text_render_conformance -- --nocapture` and `cargo test -p opencut-editor-core --test font_resolution -- --nocapture` with the native environment; confirm native cases executed, not skipped.
- [x] 7.3 From apps/agent-bridge run sequentially `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration` and `bun run test:smoke`.
- [x] 7.4 From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts`.
- [x] 7.5 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `bunx @moonrepo/cli@2.3.3 run root:openspec-validate`; inspect the protected gate and confirm any pre-archive rejection is caused only by this active change, not claim it as a pass.

## 8. Conformance and archival

- [x] 8.1 Apply openspec-verify-change after implementation checks pass; verify every normative requirement and scenario, review ownership and compatibility, and resolve all code/spec/design/task/evidence mismatches. Report unavailable checks and block completion if required evidence is missing.
- [x] 8.2 Use openspec-sync-specs and openspec-archive-change to synchronize and archive this corrective change separately, preserving the original archive.
- [x] 8.3 Run `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` after archival; require both to pass and record final evidence and remaining limitations accurately.
