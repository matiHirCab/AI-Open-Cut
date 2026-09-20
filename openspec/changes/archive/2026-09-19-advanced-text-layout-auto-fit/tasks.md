## 1. Approval and canonical fixtures

- [x] 1.1 Record explicit user approval of proposal, design and delta specs (2026-09-19: "Approve"). Final contract conformance review remains in task 5.4.
- [x] 1.2 Add canonical layout/diagnostic examples and invalid boundaries, schema-21 migration fixtures and additive capability reporting to the owned catalogs; update contract ownership consumers. Covers Closed opt-in text layout and Keep public contracts compatible.

## 2. Core model and persistence

- [x] 2.1 Implement optional layout types/defaults and core validation with automated Opt in and inherit existing styles, Validate all boundaries and locations, and Reject unusable padding boxes scenarios.
- [x] 2.2 Implement locked schema-21 migration for current state/history and applicable drafts, with Migrate history atomically and preserve legacy pixels tests, future-version rejection and injected pre/post-commit recovery evidence.
- [x] 2.3 Preserve standalone/batch alias/draft style replacement and lifecycle semantics; test Edit through aliases and lifecycle operations and Reject stale missing and incompatible edits, including font ownership rollback.

## 3. Core layout and rasterization

- [x] 3.1 Implement pinned cluster tracking, wrap modes and line boxes; independently test Track and wrap multilingual clusters across styles/bidi/mandatory separators.
- [x] 3.2 Implement box alignment, rounded solid backgrounds and effect-aware raster extents; test Position and paint a bounded text block with fractional geometry and all alignment variants.
- [x] 3.3 Implement exact descending integer fitting and cumulative scene work accounting; test Resolve each fit mode and exact ties, Report unavoidable overflow, and Bound fitting work and reject missing bounds without destination side effects.
- [x] 3.4 Carry a shared resolved layout and diagnostics through core evaluation/render paths after animation/substitution/expansion; test Verify layout across every render intent and Reopen without original fonts and fail safely with independent decoded golden evidence.

## 4. Public consumers and documentation

- [x] 4.1 Update typed headless, bridge Zod/MCP and diagnostic consumers of canonical fixtures; add native/bridge contract parity and standalone/batch/draft MCP integration and packaged smoke coverage for the new style fields, diagnostics and invalid cases.
- [x] 4.2 Document all observable fields, units, defaults, fallback precedence, fitting/overflow rules, complexity limits, diagnostics, schema migration and compatibility; record scenario-to-test names and actual check evidence in this change.

## 5. Verification and archival

- [x] 5.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` from the repository root; record full logs/exit status and resolve all failures.
- [x] 5.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration` and `bun run test:smoke`; from apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts`. Record full logs and resolve failures; unavailable required checks block completion.
- [x] 5.3 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `moon run root:openspec-validate`; before archival only rejection caused by this active change is expected, and every other failure blocks archival.
- [x] 5.4 Use `$openspec-verify-change` to verify every requirement/scenario against code, tests, design and completed tasks; resolve mismatches and record evidence limitations.
- [x] 5.5 Use `$openspec-sync-specs` and `$openspec-archive-change` after passing implementation checks and verification; rerun `moon run root:openspec-validate` and strict all-spec validation, requiring both to pass before declaring completion.
