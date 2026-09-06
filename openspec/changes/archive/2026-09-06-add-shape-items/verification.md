# Verification: add-shape-items

Verified on 2026-09-06 using the openspec-verify-change workflow. User approval of the proposed scope and contracts was received before implementation.

## Assessment

Implementation conforms to the approved geometry, editing, migration, transport and rendering requirements. No implementation/specification mismatches remain. Final contract-owner review was approved by the user on 2026-09-06. Living specifications are synchronized, the change is archived, and the post-archive Moon gate passes. All 17 tasks are complete; no verification blockers remain.

| Dimension | Evidence |
| --- | --- |
| Completeness | Implementation tasks 1–4 complete; regression and conformance checks below pass. All 17 tasks complete, including approved contract review, archival and the successful post-archive policy check. |
| Correctness | Seven normative requirements mapped to automated evidence below. |
| Coherence | Models/validation/migration/editing remain in editor-core; evaluator compiles bounded geometry; renderer consumes evaluated sources; headless and MCP translate typed operations. Existing architecture tests pass. |

## Requirement and scenario evidence

| Requirement / scenarios | Implementation and automated evidence |
| --- | --- |
| Closed bounded shape geometry: every geometry; invalid shapes; anchor/clipping | `model/shape.rs` defines the closed union and reuses vector validation. `tests/shape_items.rs::canonical_shapes_roundtrip_and_reject_at_documented_stage` and `shape_numeric_and_collection_boundaries` consume all seven valid and eighteen invalid canonical fixtures. `evaluated_scene/shapes.rs` tests independently assert analytic quadratic/cubic extrema, rounded-radius scaling, clockwise stars and empty paths. `shape_anchor_is_independent_of_stroke_padding_and_offscreen_origin` checks an asymmetric polygon with a noncentral anchor and two stroke widths. Native fixture includes a partially clipped polygon. |
| Canonical shape evaluation and bounded raster work: occurrences; expensive geometry; exact vector semantics | `evaluated_scene/shapes.rs` enforces bounded adaptive flattening, analytic anchors, surface limits and dash expansion budgets before rasterization. `shape_occurrences_preserve_clocks_opacity_identity_and_aggregate_bounds` checks repeated component occurrence identity, retimed clocks, parent opacity and the inclusive 1,048,576-segment limit versus overflow. Existing nested component evaluation tests cover the shared traversal. Raster unit tests independently assert linear-light premultiplied gradients, pad extension, transparent radial colors, nonzero/evenodd fill, open paths, fill-before-stroke, all caps/joins, signed dash phase, subpath reset and miter fallback. |
| Transactional shape editing: aliases; atomic failures; lifecycle; definitions | Core `tests/shape_items.rs` covers aliases, parent/z-order, geometry/style replacement, omitted/null semantics, stale revisions, missing references, locked tracks, later batch rollback, move/trim/split/duplicate/hide, undo/redo/reopen, legacy keyframe activation and split rebasing, persisted drafts and scoped components. Hidden invalid unused definitions reject without rewriting. Audio edits reject shapes; existing generic lifecycle/ordering suites remain green. |
| Atomic schema 14 shape activation: mixed migration; malformed snapshots; recoverable history | Core fixture tests migrate source schemas 1–13 and mixed nonempty current/undo/redo histories, reject old-schema shapes in current/undo/redo, hidden unused definitions, invalid shapes and future versions with byte preservation. Existing schema-zero/future/reference tests remain green. `supported_migrations_recover_every_publication_phase` now includes schema 13 at all five publication phases; `supported_migration_before_journal_failure_preserves_generation` includes schema 13 before journal publication. Legacy migration expectations use the current schema constant. |
| Shared complete shape rendering: every intent; side-effect-free failure; legacy compatibility | `renderer/golden/shapes.rs::native_shape_render_conformance` verifies frame/range/persisted-draft/export using seven shapes, gradients/transparency, dashes/strokes, asymmetric transforms, clipping, parent opacity, components, evenodd holes and legacy overlap. It compares exact evaluated plans, native decoded SSIM >= 0.99, non-silent aligned PCM RMS <= 0.0001 and timing within one frame. Independent rectangle/background pixel oracles and the separate vector unit oracles guard against coordinated drift. `invalid_and_expensive_shapes_fail_before_artifact_io_for_all_facades` asserts zero workspace/output/process effects. The required native gate also passes the existing rule-card and flat audiovisual goldens without changing their references. |
| Typed discoverable shape workflows: real clients; errors; partial readiness | Canonical shape, headless, MCP schema/annotation and ownership catalogs are synchronized. Rust protocol and TypeScript fixture tests check every shape and error acceptance stage. The shared `tests/shape-workflow.ts` runs against source and packaged MCP clients: discovery, all standalone variants, aliased edits, preview/export jobs, typed failures, byte preservation, conflicts, locks, undo/redo and reopen. The transport smoke renderer is the existing fake process adapter; real pixels/audio are verified separately by the native gate. Missing-renderer status preserves `shape_items` and omits `shape_rendering`. |
| Governed parity and deferred activation: shared fixtures; legacy workflows | Vector primitive catalog remains `core_primitives_only`; shape activation has its own catalog/capabilities. Rust/TypeScript vector parity suites pass unchanged. Documentation distinguishes primitive ownership from schema-14 activation. |

Paths in the evidence table are relative to `crates/editor-core` unless explicitly identifying the bridge. Native shape references and their recipe are in `crates/editor-core/tests/fixtures/shapes/`; legacy golden references were not updated.

## Executed checks

| Check | Result |
| --- | --- |
| `cargo fmt --check --all` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS: 369 passed across 22 suites; 6 intentionally ignored existing capture/benchmark helpers. These are explicit maintenance helpers, not skipped required conformance. |
| Required native gate: `cargo test --release -p opencut-editor-core native_golden_render_conformance --lib -- --nocapture` | PASS, 60.74 seconds, `OPENCUT_GOLDEN_REQUIRED=1`; existing local FFmpeg/ffprobe 8.1.2 essentials build and checked-in DejaVuSans.ttf. Covers legacy rule-card, flat audiovisual and new shape fixtures. |
| Focused native shape conformance | PASS for all four intents, including a persisted draft; new shape reference inspected visually. |
| Bridge `bun run typecheck` / `bun run lint` | PASS |
| Bridge `bun run test` | PASS: 380 tests in 16 files |
| Bridge `bun run contracts:check` | PASS: headless protocol, Rust vector/shape and TypeScript catalog parity; 316 TypeScript parity tests |
| Bridge `bun run test:integration` | PASS: 10 tests, including shape preview/export jobs |
| Bridge `bun run test:smoke` | PASS: 5 packaged tests, including shape preview/export jobs |
| Worker `bun run ../agent-bridge/scripts/run-python-tests.ts` | PASS: 10 unittest and 5 pytest tests |
| Pinned OpenSpec `validate --all --strict --no-interactive` | PASS: 20 items |
| `moon run root:openspec-validate` via pinned `bunx @moonrepo/cli@2.3.3` | PASS after archival: normalization, all 231 policy tests, strict OpenSpec validation and final CI parity gate policy pass. The earlier active-change rejection is resolved. |
| `git diff --check` | PASS |

The first sandboxed Rust run could not sample a child process; the full run with process access passed. A Windows executable lock during simultaneous Rust/source tests was resolved by running source integration after Rust completion. The long debug native run was stopped and replaced with the same successful optimized conformance test; it is not counted as passing evidence. No test expectations, gates or warning policies were weakened to suppress these environment issues.

## Finalization approval

The user approved the completed public-contract review on 2026-09-06 with “Approve”. This resolves the designated-review blocker recorded during implementation. The six capability deltas have been synchronized, preserving unrelated living requirements. The change is archived at `openspec/changes/archive/2026-09-06-add-shape-items`. The post-archive Moon gate passed on 2026-09-06, completing task 5.5. No code or executable behavior changed during finalization; implementation regression results above remain applicable.
