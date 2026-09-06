# Verification: fix-component-migration-validation

## Completeness

The approved migration correction is implemented and verified. All eight tasks are complete, the delta is synchronized, and the change is archived. No implementation or scenario mismatch remains. One modified requirement has four scenarios; no scenario requires a manual-only coverage exception.

## Correctness and traceability

| Atomic schema 13 instance activation scenario | Evidence |
| --- | --- |
| Migrate retained history and reopen | Existing component/store mixed-history tests and new `source_schema_valid_transforms_preserve_content_and_reopen`: schemas 11/12/13, default transforms, transform2d, schema-13 legacy transforms, mixed history, content equality and byte-stable reopen |
| Reject invalid and interrupted migration | Existing root-instance rejection, unknown-version and `supported_migrations_recover_every_publication_phase` tests retained |
| Reject forbidden source transforms across current and history | `source_schema_transform_rejection_preserves_current_and_history`: 48 cases (2 schemas × 3 locations × 4 transform fields × 2 visibility states), INVALID_ARGUMENT/non-retryable and unchanged project/history bytes; `source_schema_transform_failure_preserves_all_input_documents`: 6 typed clone-atomicity cases |
| Preserve valid transform migration and schema-13 behavior | Six positive integration cases in `source_schema_valid_transforms_preserve_content_and_reopen`; full native component evaluation regression suite |

The rejection test first failed on the uncorrected implementation because opening returned a successfully migrated schema-13 project. It passed after the guard was added.

## Coherence

The private guard in `migrations.rs` executes before source versions 11–12 are changed and traverses all definition tracks/items without visibility or reachability filtering. Existing clones protect caller inputs; unchanged store validation/transaction flow protects files. Schema 13, legacy normalization, root-instance rejection and unsupported-version handling remain unchanged. No public contract shape or version changed.

An initial implementation called the validation owner from migrations. The architecture test rejected that forbidden dependency; the guard was moved into the migration owner and the design updated to match ADR 0003. Strict Clippy and the architecture test pass after correction. No ADR or architecture-test allowance was changed.

## Checks

- Focused source-schema tests: PASS (3 tests).
- Native component-evaluation tests with FFmpeg 7.1.1 and OPENCUT_GOLDEN_REQUIRED=1: PASS (9 tests).
- Rust formatting and workspace all-target strict Clippy: PASS.
- TypeScript typecheck/headless tests/canonical parity (`bun run contracts:check`): PASS (20 contract tests).
- Bridge lint: PASS; unit tests: PASS (84); MCP integration: PASS (9); packaged smoke: PASS (4).
- Hermetic Python worker: PASS (10 unittest + 5 pytest).
- Strict OpenSpec validation: PASS (18 items before archive).
- Full workspace Rust rerun with native golden coverage: PASS.
- `git diff --check`: PASS.
- Final archive-only `moon run root:openspec-validate` (pinned Moon 2.3.3): PASS, 17 specs plus policy checks. The sandboxed attempt could not download the configured toolchain plugin; the authorized network-enabled rerun passed.

Diagnostic logs are ignored under `target/migration-fix-*.log`. The first workspace run failed the ownership edge check described above; its final rerun uses the corrected owner placement. The expected pre-fix regression failure is resolved. Native runs use the existing supported FFmpeg/ffprobe 7.1.1 paths and checked-in DejaVuSans test font. The five existing ignored Rust helpers/benchmarks are unchanged; owning tests exercise required helper subprocesses.

## Final assessment

Completeness: 8/8 tasks, 1/1 requirement and 4/4 scenarios covered. Correctness: rejection, preservation and compatibility tests pass. Coherence: ADR ownership respected, no new dependency edges. No unresolved critical issues, warnings or required skipped checks. Archived on 2026-09-06 after user-authorized verification and synchronization.
