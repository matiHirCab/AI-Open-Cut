# Verification: add-vector-paint-primitives

Date: 2026-09-06. Scope explicitly approved by the user before implementation.

## Assessment

| Dimension | Result |
| --- | --- |
| Completeness | All five requirements and ten scenarios implemented and covered; all 14 tasks complete, contract review approved, living specification synchronized, change archived, and final Moon gate passed |
| Correctness | Strict production Rust types and pure validation agree with mirrored TypeScript schemas on 214 canonical cases |
| Coherence | Reference-free primitives remain in core; ADR 0003 and its architecture matrix include the new private owner; no persisted or transport activation |

No unresolved implementation/spec mismatch or finalization blocker remains. All required checks passed.

## Scenario traceability

All focused Rust evidence is in crates/editor-core/tests/vector_primitives.rs; TypeScript evidence is in apps/agent-bridge/tests/vector-primitives.test.ts. Both consume contracts/vector-primitives-v1.json.

| Requirement / scenario | Implementation and automated evidence |
| --- | --- |
| Strict vocabulary / round-trip typed primitives | vector.rs strict Serde structs/enums and validate methods; canonical_vector_primitives; each valid TypeScript catalog case asserts exact parsed and serialized value equality |
| Strict vocabulary / reject malformed and unsafe values | Closed structs/tags, finite coordinate bounds and INVALID_ARGUMENT; shared missing/wrong/unknown/unsafe fixtures; nonfinite_constructed_values_are_rejected_everywhere; nonfinite_nested_paints_and_all_command_coordinates; TypeScript recursive numeric-leaf mutation |
| Colors and gradients / accept all paints and boundaries | Paint::validate and VectorColor::validate; solid/linear/radial fixtures, transparent/opaque channels, min/max coordinates/radius, 2/64 stops |
| Colors and gradients / reject ambiguous gradients | Paint::validate; duplicate/reversed/missing endpoint stops, coincident points, nonpositive/oversized radius, malformed stop/color and 65-stop fixtures |
| Strokes and radii / accept boundaries | Stroke::validate and CornerRadii::resolve; all cap/join combinations, dash limits and offsets, radius boundaries; radii_use_one_independent_scale_and_preserve_input; radius_resolution_checks_each_limiting_side |
| Strokes and radii / reject invalid collections | Odd/oversized/nonpositive dashes, width/miter/radius limits; nonfinite tests and invalid rectangle dimension tests |
| Paths / accept open and closed paths | VectorPath::validate iterative state machine; move-only, open/closed, multiple subpaths, every command and both fill rules in shared fixtures |
| Paths / reject grammar and complexity | Shared empty, draw-before-move, close-before-draw, repeated-close, draw-after-close, missing-control and invalid-control cases; path_limit_and_preflight and TypeScript 4096/4097 test |
| Parity / shared fixture evidence | canonical_vector_primitives; rejects_malformed_catalog_wrappers; TypeScript malformed wrapper/identifier/duplicate-ID checks; contracts:check explicitly executes native vector and TypeScript vector suites |
| Deferred activation / preserve workflows | Existing workspace tests cover migrations, future-version rejection, optimistic revisions, atomic batch failures, undo/redo/reopen and native preview/range/export; headless protocol, source integration and packaged smoke suites pass |

Each of the ten named specification scenarios has evidence. Colors/stroke/fill sampling rules are documented contractual semantics for #28, not claims that this prerequisite implements a rasterizer.

## Compatibility and review

No change to model.rs, migrations.rs, evaluated_scene.rs, renderer code, headless request/response unions, MCP surface/capability catalogs, stable errors, or legacy color strings. Schema remains 13. Reference-free values add no missing-reference, revision, alias, or migration behavior. Existing workflow tests provide regression evidence; new activation tests belong to #28.

The additive contract owner and actual consumers are registered in contract-ownership-v1.json and .github/CODEOWNERS. The new vector owner imports only facade errors and serde, with no outward dependency. No new library dependency is introduced.

## Validation results

- PASS: cargo fmt --check --all.
- PASS: cargo clippy --workspace --all-targets -- -D warnings, repeated after final focused-test refinements.
- PASS: cargo test --workspace, including native golden/render, migration, history, architecture, desktop, and headless protocol tests. Existing six ignored helper/maintenance entry points retain their status: four process helpers are invoked by parent tests; review-only reference recapture and externally supplied performance-report validation are not required conformance cases for this change.
- PASS: cargo test -p opencut-editor-core --test vector_primitives, seven tests including all canonical cases and final catalog-wrapper refinements.
- PASS: bun run contracts:check: native headless and vector suites plus 238 TypeScript contract/vector tests.
- PASS: bridge bun run typecheck, bun run lint, and bun run test; final run passed 302 tests in 15 files.
- PASS: bun run test:integration, nine tests.
- PASS: bun run test:smoke, four packaged tests after rebuilding.
- PASS: hermetic Python worker runner, ten speech tests and five transcription tests.
- PASS: pinned OpenSpec strict change validation.
- PASS: git diff --check.
- PASS: root:openspec-validate through pinned @moonrepo/cli@2.3.3 after approved archival, exit code 0. Workflow normalization, 231 policy tests, all 19 living specifications, and the final CI parity policy gate passed. The earlier active-change inventory blocker is resolved without changing policy.

Local diagnostic logs under ignored local-data/vector-*.log are not deliverables. Initial test-number normalization and TypeScript optional-index diagnostics were corrected and rechecked.

## Finalization

The user explicitly approved the completed canonical contract and consumer implementation on 2026-09-06 in response to the designated contract-owner review request. This is separate from the earlier proposal approval. All five requirements and ten scenarios are synchronized into openspec/specs/vector-primitives/spec.md. The change is archived at openspec/changes/archive/2026-09-06-add-vector-paint-primitives/. The final Moon gate passed and all 14 tasks are complete. No commits, pushes, or issue-closing actions were performed.
