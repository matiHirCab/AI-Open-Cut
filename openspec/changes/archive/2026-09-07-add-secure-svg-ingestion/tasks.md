## 1. Approval and canonical contracts

- [x] 1.1 Obtain explicit approval of proposal, design, SVG subset/limits and delta scenarios before editing implementation files; record approval evidence in proposal.md (2026-09-07 user approval).
- [x] 1.2 Add contracts/svg-ingestion-v1.json with closed normalized document fields, source grammar, numerical limits, capability IDs and valid/invalid fixtures; update contract ownership, headless/MCP catalogs and persisted schema fixtures before consumers. Trace all six delta requirements and obtain @matiHirCab contract-owner review.

## 2. Core model, validation and migration

- [x] 2.1 Inspect and pin a structural XML parser that meets the approved no-resolution and bounded-work constraints; add core acceptance, namespace/escape, DTD/entity, script/handler, URL/font, unsupported-feature and every-budget boundary tests for Fail-closed static SVG ingestion and Explicit SVG geometry and complexity.
- [x] 2.2 Implement core-only normalization into closed versioned SVG model records with duplicate-preserving decoding and shared semantic validation, including hidden definitions, drafts and history; prove no parser/render resource access and no source-bearing diagnostics.
- [x] 2.3 Add schema-15 migration fixtures and implement version-only 14-to-15 migration for current/undo/redo state under existing recoverable transactions. Test supported legacy versions, foreign/future SVG, invalid history and interruptions against Canonical persisted SVG and migration.

## 3. Core editing and evaluation

- [x] 3.1 Implement add_svg and ordered batch aliases in timeline/store with existing revision checks, track/parent restrictions and atomic results. Add facade tests for every Transactional SVG timeline operations scenario, including stale revision precedence and later-operation rollback.
- [x] 3.2 Integrate SVG with applicable generic edits, component definitions/instances and drafts; test transform/keyframe restrictions, split/duplicate/delete, stacking, parenting and exact undo/redo/reopen behavior; reject unsupported patches/audio/transition use.
- [x] 3.3 Evaluate SVG into bounded document-ordered subdraws with viewport clipping and existing composed transforms/clocks/opacity. Extend the established raster path and add independent geometry/pixel tests for Shared evaluated SVG rendering and public evidence. Update ADR 0003 and architecture tests if a new owner edge is necessary.

## 4. Headless and bridge consumers

- [x] 4.1 Extend typed headless standalone/batch unions, result serialization and readiness capabilities; add protocol tests consuming canonical fixtures and proving typed errors, aliases and compatibility.
- [x] 4.2 Extend bridge structural Zod schemas, headless contract, timeline_add_svg registration and capabilities with thin delegation to core. Add shared-fixture parity tests and actual MCP lifecycle workflows to integration and packaged smoke suites.

## 5. Documentation and conformance

- [x] 5.1 Document accepted/rejected SVG syntax, defaults, coordinate/timing/order semantics, normalization, security budgets, capability detection, inline request/batch examples and backup-only downgrade. Keep proposal/design/contracts and implementation synchronized.
- [x] 5.2 Add reviewed SVG native render fixtures with independent geometry/pixel expectations, synthetic audio, nested/retimed components and lifecycle states; compare frame, range, draft and export at approved tolerances, with malicious/oversized input failing before artifact/process work.

## 6. Verification and archival

- [x] 6.1 Run root `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; record results and resolve failures.
- [x] 6.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; include SVG fixture tests in contracts:check and record results.
- [x] 6.3 From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts` for existing hermetic provider regressions. From root run `cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact` with the required native mode and configured tools/fonts from .github/workflows/bun-ci.yml, retaining all legacy fixture gates; missing dependencies or skipped required checks block completion.
- [x] 6.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; use openspec-verify-change and produce a requirement/scenario-to-test verification report with actual commands/results, resolving every mismatch.
- [x] 6.5 Use openspec-archive-change to synchronize accepted requirements into living specs and archive the verified change. Then run `moon run root:openspec-validate` through the ordinary protected policy; active changes intentionally block this gate before archival. Report all failed/skipped checks and do not mark implementation complete until required validation succeeds.
