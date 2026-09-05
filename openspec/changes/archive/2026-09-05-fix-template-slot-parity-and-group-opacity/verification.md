# Verification: fix-template-slot-parity-and-group-opacity

Reviewed on 2026-09-05 against the explicitly approved proposal, design, tasks and three delta specifications. The original archive is preserved. Both previously reproduced defects now have regression coverage. All implementation checks pass. The user approved completed contract review on 2026-09-05; specification synchronization, archival and the final Moon gate are complete.

## Completeness and correctness

Four modified requirements have implementation evidence and coverage for all thirteen scenarios (six added scenarios plus seven retained scenarios).

| Scenario | Automated evidence |
| --- | --- |
| Round-trip all eight kinds | template_slots.rs canonical_all_kinds_roundtrip_defaults_overrides_history_and_reopen; shared component workflow |
| Validate absent and invalid values | required_optional_unknown_values_and_definition_replacement_are_atomic; canonical_invalid_and_closed_wire_records |
| Preserve special slot identifiers | special_slot_keys_preserve_required_values_and_atomic_errors; contracts.test.ts preserves and validates every canonical special override key; verifySlotRegressionWorkflow |
| Validate special-key values without dropping entries | same native and TypeScript regressions; real workflow covers malformed own keys, missing required values and unknown IDs |
| Resolve local identity and compatible properties | locks_scope_duplicate_writers_and_effective_domain_rules; existing all-kind tests |
| Reject invalid bindings and effective values | effective_asset_and_duration_are_validated_together_and_defaults_remain_validated; existing lock/scope tests |
| Apply group opacity without requiring explicit Transform2D | validation.rs group_opacity_uses_transform2d_and_preserves_other_fields; group_opacity_defaults_overrides_history_and_failures; verifySlotRegressionWorkflow |
| Preserve group opacity failure atomicity | group_opacity_defaults_overrides_history_and_failures covers invalid default/override bounds, stale revisions, locked tracks and byte-identical failed batch rollback |
| Run real slot workflows | verifyComponentWorkflow and verifySlotRegressionWorkflow through source and packaged MCP |
| Propagate atomic slot failures | shared workflow and existing standalone/batch error tests |
| Preserve override maps through real transports | verifySlotRegressionWorkflow; required schema-12 field tests and exact input/output JSON Schema comparison; registered MCP catalog comparison |
| Compare native and bridge evidence | canonical Rust template-slot fixtures and TypeScript contracts suite |
| Govern special-key and group-opacity regressions | new canonical regressions consumed by native persistence/validation tests and TypeScript unit/source/packaged workflows; completed contract review explicitly approved on 2026-09-05 |

## Coherence

The bridge uses public Zod APIs, checks every own enumerable string entry with the existing closed slotValueSchema, prefixes validation paths and reconstructs parsed values through Object.fromEntries. This preserves __proto__, constructor and toString as data without mutating input objects or prototypes. Domain lookup, default precedence, constraints and mutation semantics remain in core. Input/output JSON Schema metadata is derived from the prior record declaration, and registered schema parity passes without changing the MCP surface catalog.

Core applies group opacity to an initialized or existing Transform2D only on a derived clone, just as it does for component instances. Tests assert exact opacity and preservation of every other transform field, legacy identity and stored base tracks. No dependency, protocol, persisted schema, renderer or provider behavior changed. No new ownership edge was introduced.

## Regression evidence and investigation

- Before fixes, the TypeScript regression omitted the canonical __proto__ entry and the native effective-group test observed absent Transform2D instead of the expected opacity-bearing value. Both reproduced the review findings.
- Vitest's JSON module transformation also omitted __proto__ from the fixture input. Tests now parse canonical JSON source bytes with JSON.parse and use JSON imports only for types. This prevents the fixture loader from hiding the regression; native tests read the same canonical bytes.
- An initial parallel native workspace run failed the existing process-memory sampler's child-allocation threshold. The unchanged sampler passed in isolation; the complete suite subsequently passed with RUST_TEST_THREADS=1 to reduce contention. No test or assertion is disabled.
- Existing ignored native helper/maintenance tests retain their documented status; parent tests invoke subprocess helpers as before. Logs in ignored target/ are local evidence.

## Validation results

- PASS cargo fmt --check --all.
- PASS cargo clippy --workspace --all-targets -- -D warnings.
- PASS focused native library tests (177 passed, five existing ignored helpers/maintenance tests) and template-slot tests (14 passed).
- PASS full cargo test --workspace with RUST_TEST_THREADS=1, established FFmpeg 6.1.1/ffprobe and repository DejaVuSans. Native golden, component output parity, migration/recovery, architecture and headless lifecycle tests all passed; no required native test was skipped.
- PASS bun run contracts:check (native headless evidence and 17 TypeScript contract tests).
- PASS bun run typecheck and bun run lint.
- PASS bun run test (81 tests, 14 files).
- PASS bun run test:integration (nine tests with expanded real slot workflow).
- PASS bun run test:smoke (four packaged tests with the expanded real slot workflow).
- PASS bun run scripts/run-python-tests.ts (10 speech and five transcription tests).
- PASS focused strict OpenSpec validation and git diff --check.
- PASS final moon run root:openspec-validate through bunx @moonrepo/cli@2.3.3 after archival: workflow normalization, 231 policy tests, all 16 living specifications and the CI parity gate pass (exit code 0).

## Finalization

All 14 tasks are complete. The user explicitly approved the completed contract corrections in response to the designated CODEOWNER review request. Six new scenarios were synchronized across three living specifications, preserving existing requirements. The verified follow-up is archived at openspec/changes/archive/2026-09-05-fix-template-slot-parity-and-group-opacity/. The final Moon gate passed. No commits or pushes were performed.

MCP surface catalog SHA256 remains 4F0341ED84EFD93BEB573B588C82636F30A9DFE8E7EF89C91B1574298067BFD2 throughout this follow-up; the runtime normalized input/output schema parity tests pass against it.
