# Verification: content-addressed-font-shaping

## Scope and approval

Issue #33 is implemented through this approved change. The user approved the
proposal, including four required faces and the one-time schema-19 layout
transition, on 2026-09-12. This authorizes implementation; it is not recorded as
a separate review of the final canonical artifacts. ADR 0002 and
`.github/CODEOWNERS` designate `@matiHirCab` for that review. On 2026-09-13,
the user explicitly approved the final contract changes in response to the
review request, clearing task 4.2 and authorizing synchronization and archival.

## Completeness and scenario traceability

Paths below are relative to the repository. Every normative requirement has
automated coverage. Combined lifecycle scenarios use both focused font tests
and the existing component/slot/repeater and native rendering suites; the
historical schema-18 pixel oracles have not been recaptured.

| Requirement / scenarios | Implementation | Automated evidence |
| --- | --- | --- |
| Durable bounded font bindings: pin exact bytes and fallback | `src/model/font.rs`, `src/fonts.rs`, `src/assets/fonts.rs` in editor-core | `packaged_faces_match_canonical_contract`; `selector_precedence_root_order_and_fallback_are_deterministic`; `pinned_faces_survive_source_removal_and_history`; `explicit_reset_resolves_while_paint_updates_retain_binding`; MCP `verifyRichTextWorkflow` checks all four canonical hashes |
| Reject unsafe/excessive resources; inclusive limits and missing styles | Core font validator and persistence-backed resolver | `font_bytes_fail_closed` (16 MiB boundary, malformed/collection/variable/no-outline input); `catalog_limits_and_paths_are_canonical` (256-file and 256 MiB boundaries); `discovery_count_and_depth_limits_are_inclusive`; `unsafe_selectors_and_missing_styles_preserve_authoritative_bytes`; `font_resolution_rejects_storage_reported_symlink_escape` |
| Reversible stable editing: preserve bindings after source changes | Core timeline/store preparation and immutable draft steps | `pinned_faces_survive_source_removal_and_history`; `draft_only_fonts_survive_reopen_and_commit_without_sources`; `explicit_reset_resolves_while_paint_updates_retain_binding`; existing split/copy/move/trim, component lifecycle and history regressions |
| Resolve aliases and roll back; stale/missing targets | Core batch validation before font publication and revision check before migration | `invalid_batch_and_stale_revision_publish_no_binding`; `stale_legacy_edit_does_not_activate_fonts`; headless `rich_text_documents_roundtrip_batches_drafts_and_failures`; MCP `verifyRichTextWorkflow` |
| Canonical shaping: multilingual/style context | `src/fonts/shaping.rs` | `kerning_ligatures_and_combining_clusters_have_independent_expectations` (AV glyphs 36/57, 1270-unit kerned A advance; ffi glyph 5044; composed e-acute glyph 171); `paint_boundaries_preserve_shaping_and_newlines_remain_explicit`; `bidi_missing_glyphs_and_repeatability_are_fixed` |
| Wrap at clusters and bound work | Core shaping with bidi L1 adjustment and Unicode line breaks | `work_limits_and_cluster_wrapping_are_inclusive`; ligature fixture with wrap width 1; unknown profile and glyph-zero tests. The 16384-glyph budget is tested at the shared guard because the packaged face plus the independent 4096-byte document cap cannot naturally produce that many glyphs; the guard also checks each shaper result and cumulative emission. The 4096-line boundary is exercised through real shaping. |
| Centralized ownership: retain every owner | Existing `assets` reference/GC policy extended to font catalogs, including drafts | Font history and draft-only retention tests; component/slot/repeater lifecycle and migration suites run with bound text; `discarded_draft_releases_only_unowned_font_content` verifies collection on the next successful project commit |
| Damaged/unsafe pinned content and collection failures | Hash/size/path verification before load/render; common GC warning policy | `tampered_managed_font_fails_closed`; `hidden_font_integrity_fails_before_render_artifacts`; native corruption and catalog path tests; existing storage/GC fault tests, including `ASSET_GC_FAILED` without rollback of committed state |
| Atomic schema-19 activation: mixed state/history/drafts | Core migration, staged immutable faces, journaled draft updates | `schema_18_migration_pins_current_and_retained_history`; `font_and_legacy_draft_activation_recovers_every_publication_phase`; existing mixed-schema component/rich-text migration tests |
| Invalid sources/future versions preserve bytes | Version-aware project deserialization and source validation | `historical_fields_and_native_missing_bindings_fail_without_rewrite`; existing `rich_text_documents`, `components`, `template_slots`, store migration and future-schema regressions |
| Recover font activation interruptions | Same persistence journal with before/after-font checkpoints and draft-update checkpoint | `font_and_legacy_draft_activation_recovers_every_publication_phase` covers both font checkpoints, before/after journal, project, history, draft updates, draft cleanup and journal cleanup; `pending_schema_18_journal_recovers_before_font_migration`; existing transaction tests |
| Shared pinned rendering; every intent after reopen | Verified resource sidecar, canonical glyph runs, outline raster and scene-rate compositing | `native_preview_uses_pinned_glyphs_after_reopen_and_source_removal` compares frame, range, export and draft after source removal, including visible-pixel assertions; `native_rich_text_render_conformance`; component/slot/repeater native suites |
| No artifact for invalid retained/effective state | Complete retained catalog in render preflight, bounded reads and shaping before output preparation | `hidden_font_integrity_fails_before_render_artifacts`; existing invalid-source and artifact-isolation tests; unsupported-profile and shaping bounds tests |
| Advertise compatibility transition | Headless status negotiation, Zod/TS/MCP schemas and canonical catalogs | Headless `canonical_status_requests_negotiate_protocol_version_and_capabilities`; `tests/text-layout.test.ts`; exact MCP registration parity in `tests/contracts.test.ts`; integration and packaged smoke |
| Shared stored rich-text evaluation and styled root animation | Versioned layout keeps document/slot substitution, ancestry and Transform2D precedence | `native_rich_text_render_conformance`; `native_styled_root_animation_conformance` checks independent static oracles at 0/400/800 ms and frame/range/draft/export SSIM >=0.99; component evaluation, groups and Transform2D suites; native audio RMS checks unchanged |

## Correctness and coherence review

- Font semantics remain inside editor-core. Headless supplies immutable
  configuration; the bridge translates typed requests/results. ADR 0003 and its
  architecture matrix include the pure `fonts` owner and exact dependency edges.
- Four licensed DejaVu Sans 2.37 faces are embedded into the binary; no runtime
  font download or ambient lookup is used for pinned text. SHA-256 values are
  asserted against `contracts/text-layout-v2.json` in Rust and packaged MCP tests.
- Bindings are preserved on content/paint changes, and explicit selector changes
  invalidate them. Revision conflicts precede font migration side effects.
- `component_replacement_preserves_bindings_until_selector_changes` verifies
  replacement after source removal, omitted binding metadata, and selector changes.
- Explicit null font fields are rejected in historical projects and drafts;
  `legacy_draft_rejects_explicit_null_font_fields` supplements the historical
  source rejection scenarios. Network roots fail before discovery enumeration.
- Current/history/draft source validation finishes before font publication.
  Immutable unowned bytes may survive an interrupted pre-journal publication;
  common managed GC collects them after a successful project commit. Recovery
  never journals a reference before the associated immutable bytes are present.
- Preflight verifies even hidden/unused retained fonts and uses bounded file
  reads. Shaping cache keys include profile/binding/document/layout settings;
  integrity checks precede cache use. Repeated glyph outlines share allocations.
- Glyph images are converted to the scene frame rate before animation, fixing
  the parity mismatch found by the independent animation oracle.
- Glyph media inputs are sorted by item ID before plan construction.
  `pinned_glyph_plans_match_across_intents_and_repeated_preparation` proves exact
  filter-graph/input equality across repeated frame, range and export preparation.
- Historical rich-text/rule-card fixtures remain schema-18 regression evidence;
  current schema-19 glyph/layout behavior has separate conformance coverage.
- Public declarations remain hand-authored. Canonical MCP snapshots were
  synchronized for reviewed field changes; parity tests only read/compare them.

## Validation evidence

Executed on Windows with Bun 1.4.0 and installed Rust/Cargo 1.93.0. The repository
declares Rust 1.97.0; a run on that pinned toolchain is not claimed. Native tests
use FFmpeg/FFprobe 7.1.1 and the reviewed DejaVu fixture, with
`OPENCUT_GOLDEN_REQUIRED=1`. System FFmpeg 9.0.1 rejects the repository's existing
`-filter_complex_script` invocation, so it was not used for conformance.

| Check | Result |
| --- | --- |
| `cargo fmt --check --all` | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed |
| `cargo test --workspace` with required native configuration | Passed with two test threads, including native conformance and the process sampler. Subsequent edits passed all six affected component/text suites, contract suites, the exact render-plan test and all 14 font-resolution tests including native intent parity. Final formatting and strict Clippy passed. |
| `bun run contracts:check` | Passed: headless, governed Rust suites including font resolution, and 328 TS parity tests |
| `bun run typecheck` / `bun run lint` | Passed |
| `bun run test:unit --maxWorkers=2` | Passed: 392 tests |
| `bun run test:integration` | Passed: 11 tests |
| `bun run test:smoke` | Passed: 6 packaged tests, including exact embedded hashes and draft commit |
| Hermetic Python worker runner | Passed: 10 unittest tests and 5 pytest tests |
| OpenSpec 1.5.0 `validate --all --strict --no-interactive` | Passed: 26 items in the final validation |
| Moon 2.3.3 `run root:openspec-validate` | Post-archive run exited 1: all 26 spec items passed, but archive-only policy rejects the separate active `reduce-agent-context-overhead` change. Full log: `%TEMP%/opencut-font-postarchive-moon.log`. |

The seven ignored Rust entries are explicit capture/report helpers and
subprocess helpers, not omitted native conformance tests. Parent tests invoke
the process helpers where required; reference recapture is intentionally not run.

## Outstanding gates

Final contract review is approved. All four delta specifications are synchronized,
and this change was archived on 2026-09-13. The post-archive Moon check ran and
failed only because the separate active change remains. Task 5.6 stays open
until that repository-wide gate passes; no implementation work remains for #33.

The final workspace also contains the separate active change
`reduce-agent-context-overhead`. It is outside issue #33 and was left untouched;
the archive-only Moon policy requires that change to be resolved independently.

## Final assessment

Completeness: 22 of 23 tasks complete; all nine delta requirements have automated
evidence. Correctness and coherence review found no remaining implementation
mismatches. Only the repository-wide post-archive Moon gate (task 5.6) remains blocked by
the separate active change. The toolchain warning
above remains: validation used installed Rust 1.93.0 rather than pinned 1.97.0.
