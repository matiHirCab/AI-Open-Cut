# Verification: draft font retention and Unicode breaks

## Approval and compatibility

The user explicitly requested implementation of the complete plan after selecting preservation across non-font edits, rejection of ambiguous matches, and correction within layout v2. The change preserves public fields, schema 19, draft v2 and dependency pins. It adds no architecture edge. The narrow separator-rendering correction is documented and approved; no contract artifact or consumer declaration changes are required.

## Scenario evidence

| Requirement/scenario | Implementation and regression evidence |
| --- | --- |
| Reset without selector collision; stable editing after source changes | assets/fonts.rs captures unresolved root/component-local identities. `draft_reset_does_not_capture_another_items_old_binding` covers reset, draft state, reopen and commit. `component_draft_bindings_use_scoped_local_identity` covers equal root/local IDs across two components and component edits/commit. |
| Preserve matched operation fonts | assets aligns old steps before store replaces operations. `draft_reordering_and_non_font_edits_retain_bindings` covers insertion, reordering, removal, text/style changes, source removal, changed defaults and changed selectors. Component test covers local styled text replacement. |
| Reject ambiguous replacement atomically | `ambiguous_draft_matching_is_atomic_and_equivalent_matches_succeed` covers exact-match precedence, distinct hashes, atomic INVALID_ARGUMENT, unchanged owned-file count and equivalent matching. |
| Canonical mandatory breaks | `mandatory_separators_preserve_lines_clusters_and_paragraph_direction` covers all eight separators, three wrapping modes, consecutive/trailing breaks, CRLF across styled runs, global byte clusters, paragraph direction and no synthetic extra line. Existing work-limit tests cover inclusive 4096 lines and cluster wrapping; byte limits prevent 4096 multi-byte separators in one valid document. |
| Shared rendering and unaffected profile behavior | `native_mandatory_separators_agree_across_render_intents` compares an independent explicit-LF oracle and frame/range/draft/export before/after source removal. Original native pinned-font test and exact kerning/ligature/combining/bidi fixtures remain unchanged. |
| Existing revision/integrity/migration guarantees | Existing font_resolution, component, draft/store, schema migration and transaction suites remain required in the full workspace run. |

## Test-first evidence

Before implementation all three new draft tests failed with incorrect retained hashes (`%TEMP%/opencut-fix-red-drafts.log`), and the separator regression failed on CR producing one line rather than four (`%TEMP%/opencut-fix-red-breaks.log`). After implementation the initial three draft tests and all five shaping tests passed. Component and native oracle additions are included in final validation.

## Validation status

Full logs are uncommitted under `%TEMP%/opencut-fix-*.log`. Checks use installed Rust/Cargo 1.93.0, Bun 1.4.0, and explicit FFmpeg/FFprobe 7.1.1 paths with OPENCUT_GOLDEN_REQUIRED=1 and two Rust test threads. The repository declares Rust 1.97.0; testing with that toolchain is not claimed.

| Check | Result / log suffix |
| --- | --- |
| Rust fmt and strict workspace Clippy | Passed; `clippy-final.log` |
| Draft regression suite | 4 passed; `drafts-final-retry.log` |
| Shaping unit suite | 5 passed; `shaping.log` |
| Full Rust workspace | Final rerun passed (exit 0), including all 288 library tests, the four draft regressions, 15 font-resolution tests, all remaining workspace integration suites and 27 headless tests; `workspace-final.log`. |
| Bridge typecheck / lint | Passed; `typecheck.log`, `lint.log` |
| Bridge unit suite | 392 passed; `unit.log` |
| Contracts | Passed headless/Rust consumers, 15 font-resolution tests including native separator oracle, and 328 TS parity tests; `contracts-check-retry.log` |
| MCP integration / packaged smoke | 11 integration and 6 packaged tests passed; `test-integration-retry.log`, `test-smoke-retry.log` |
| Hermetic Python | 10 unittest and 5 pytest passed; `python.log` |
| Strict change validation | Passed |
| Moon while active | 27 spec items passed; exit 1 solely for this and the separate active context-efficiency change; `moon-active.log` |
| Moon after archival | 26 spec items passed; exit 1 solely for the separate active `reduce-agent-context-overhead` change; `moon-postarchive.log` |

The initial contracts attempt failed with Windows error 5 while the shared headless executable was in use; retry passed. The first component regression used duplicate stack-order values, failed source validation, and passed after correcting the fixture. The already-running workspace invocation retained its separately compiled pre-correction test binary and therefore also failed that fixture; a full fresh workspace run was started. Neither issue required a production behavior change. The seven ignored Rust entries remain explicit recapture/report/subprocess helpers, not omitted native conformance.

## Correctness and coherence assessment

Scoped capture excludes retained texts before reducing to existing selector keys. Matching consumes each old step at most once, prioritizes exact structure globally, and rejects differing candidates before publication. Mandatory boundaries are computed from the pinned Unicode library while bidi retains original paragraphs; no persisted or public type changes were introduced. No remaining implementation/design mismatch was found. All implementation checks passed.

All 9 change tasks are complete. The accepted requirements were synchronized to font-resolution and this change was archived on 2026-09-13. The mandatory post-archive Moon invocation ran and failed only because `reduce-agent-context-overhead` remains active; that unrelated change was not modified. Repository-wide merge readiness remains blocked until that change is resolved and Moon passes. The installed-versus-pinned Rust toolchain limitation above remains explicit.
