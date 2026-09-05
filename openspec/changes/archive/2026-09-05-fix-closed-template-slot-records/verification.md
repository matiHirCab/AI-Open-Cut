# Verification: fix-closed-template-slot-records

## Summary

Implementation and automated conformance checks pass. Artifact approval was received from the user on 2026-09-05. The user explicitly approved the completed contract review on 2026-09-05 in response to the final review request. Living specs were synchronized, the change was archived, and the final Moon gate passed. This change is complete.

| Dimension | Assessment |
| --- | --- |
| Completeness | 16/16 tasks complete; completed review, synchronized specs, archive and final Moon validation recorded |
| Correctness | All three modified requirements and their new and retained behavioral scenarios have automated evidence |
| Coherence | Shared public-Zod structural wrapper; unchanged core semantics, safe override maps, published schemas, protocol 1 and schema 12 |

## Requirement and scenario evidence

| Requirement / scenario | Named evidence |
| --- | --- |
| Closed typed slot definitions and values: Reject unknown fields throughout closed slot records | `canonical_closed_slot_records_reject_unknown_own_fields` in crates/editor-core/tests/template_slots.rs; `rejects canonical closed slot records with complete nested paths` in apps/agent-bridge/tests/contracts.test.ts; native protocol test below |
| Preserve valid records and special map identifiers | `preserves and validates every canonical special override key`; `matches runtime slot fixtures and closed typed values`; `canonical_all_kinds_roundtrip_defaults_overrides_history_and_reopen`; `special_slot_keys_preserve_required_values_and_atomic_errors`; retained `group_opacity_defaults_overrides_history_and_failures` and effective Transform2D unit test |
| Retained Round-trip all eight kinds / Validate absent and invalid values / Preserve special slot identifiers / Validate special-key values without dropping entries | Existing canonical valid/invalid slot tests, exact native persistence/history tests, bridge special-key/prototype tests and `verifySlotRegressionWorkflow`; unknown IDs still reach core and return ITEM_NOT_FOUND |
| Typed template slot workflows: Reject malformed records in real standalone and batch workflows | `closed_slot_records_fail_native_decoding_without_mutating_history` in apps/headless/tests/protocol.rs; `verifyClosedSlotWorkflow` in apps/agent-bridge/tests/component-workflow.ts, invoked by both source and packaged tests |
| Preserve nested validation and schema contracts | `rejects canonical closed slot records with complete nested paths` covers slot, definition request/response, instance response, component-create request and batch paths; `delegates closed record parsing without changing types or JSON schemas` verifies delegate output, type assignment, base errors, input/output schema equality and rich-text leaf errors under __proto__ |
| Retained Run real slot workflows / Propagate atomic slot failures / Preserve override maps through real transports | `verifyComponentWorkflow`, `verifySlotRegressionWorkflow`, native protocol slot lifecycle test and existing core atomicity tests cover all-eight-kind inputs, aliases, locks, revisions, failed later operations and undo/redo/reopen |
| Canonical runtime template slot evidence: Compare exhaustive closed-record rejection evidence | Raw JSON `regressions.closedRecords`: 56 cases = 14 record locations × four unknown keys. Every case is rejected as a slot/default; the 44 applicable value cases also run as overrides. TypeScript asserts the complete location/key matrix, own-property presence, nested paths, unmodified input bytes and unchanged prototypes; Rust reads with include_bytes/from_slice |
| Verify compatibility before completion | Registered MCP input/output structural catalog parity; direct pre-change slot/value input/output JSON Schema comparison; unchanged MCP catalog SHA-256; all required checks below. Completed designated review was explicitly approved on 2026-09-05 |
| Retained Compare native and bridge evidence / Govern special-key and group-opacity regressions | Canonical Rust/TypeScript consumers, source/packaged workflows and retained group-opacity tests pass; previous completed review records and both archives are preserved |

## Correctness and design review

`closedSlotRecord` in apps/agent-bridge/src/schemas.ts checks own enumerable keys against a Set before delegating to the existing safe parser. It reports all unknown names, returns the delegate's parsed result and forwards nested issues. `closedSlotObject` derives keys from object shapes. The complete eight-kind union is guarded with exactly type/value, preserving its ordinary discriminated variants; nested document, run, asset, binding and constraint records are guarded separately. Shared consumers cover requests and responses. The existing safe override-map parser and Object.fromEntries reconstruction remain unchanged.

Metadata is derived from the underlying declarations via public Zod APIs. Both input and output slot/value schemas were compared against a pre-implementation snapshot and match exactly. Registered MCP structural catalog tests pass without editing contracts/mcp-surface-v1.json; its SHA-256 before and after is `4F0341ED84EFD93BEB573B588C82636F30A9DFE8E7EF89C91B1574298067BFD2`.

Native production code required no change: existing Serde closed-record decoding rejects the same canonical data. The native transport test executes 200 malformed requests across standalone and batch forms and compares project/history bytes after every request. Each source/packaged workflow also executes those 200 malformed default/override requests, with a valid creation before the malformed operation in batches and a creation alias for slot-definition targets. It asserts structural unknown-key errors and native byte equality after every request, then reopens and compares full state including revision. Existing successful histories and group-opacity behavior stay covered.

Canonical consumers and CODEOWNER coverage include the shared workflow and both callers. Documentation distinguishes closed record fields from open slot-ID maps. No dependency, persisted format, renderer, semantic rule, protocol, error catalog, or published structural schema changes were introduced. The two previous archive directories were not modified.

## Validation results

- PASS `cargo fmt --check --all`.
- PASS `cargo clippy --workspace --all-targets -- -D warnings`.
- PASS `cargo test --workspace` with OPENCUT_FFMPEG_PATH/OPENCUT_FFPROBE_PATH pointing to the established FFmpeg 6.1.1 tools, repository DejaVuSans as OPENCUT_TEST_FONT_PATH and RUST_TEST_THREADS=1. Includes 15 template-slot tests, 17 headless protocol tests, rendering/golden, migration/history and architecture evidence.
- PASS `bun run contracts:check`: typecheck, native headless tests and 19 bridge contract tests on final fixtures/ownership.
- PASS `bun run typecheck` and `bun run lint` (47 files).
- PASS `bun run test`: 83 tests across 14 files.
- PASS `bun run test:integration`: nine tests; final runtime 31.38 seconds.
- PASS `bun run test:smoke`: four tests against the release sidecar and compiled bridge; runtime 11.53 seconds plus packaging.
- PASS `bun run scripts/run-python-tests.ts`: 10 speech and five transcription tests.
- PASS `bunx @fission-ai/openspec@1.5.0 validate fix-closed-template-slot-records --strict` and `git diff --check`.
- PASS direct old/new slot/value input/output schema equality and unchanged MCP catalog hash, independently of registered structural parity.

An initial source integration run exceeded the existing 60-second per-test limit. Redundant reopens were consolidated into one final full-state read while retaining byte equality after every malformed request; Buffer.equals replaces expensive per-byte matcher traversal. The final source and packaged runs pass with the existing timeouts and all 200 requests retained. Initial TypeScript typing and test-helper complexity errors were corrected; final type and lint checks pass.

Five existing Rust tests remain explicitly ignored by the repository: four subprocess helpers exercised by parent tests and one helper for externally captured performance reports. This correction adds no ignored tests and skips no required native suite. Local logs and pre-change snapshots are in ignored target/closed-slot-review/.

## Completion gates

Completed designated contract review was explicitly approved by the user on 2026-09-05 after reviewing the implementation evidence above. The review covers the final canonical fixtures, governed consumers and unchanged published schemas; designated CODEOWNER is @matiHirCab. Both previous archives are preserved.

Living specifications for template-slots, agent-bridge and motion-graphics-contracts were synchronized, and the change was archived at openspec/changes/archive/2026-09-05-fix-closed-template-slot-records. PASS final `bunx @moonrepo/cli@2.3.3 run root:openspec-validate`: 231 policy tests, 16 specifications and CI parity policy validation. The active changes directory contains only archive. Final git diff --check passes and the published MCP catalog hash remains unchanged. No unresolved implementation/specification mismatches or other warnings were found.
