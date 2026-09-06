# Verification: fix-vector-json-representations

Date: 2026-09-06. The user explicitly authorized the complete correction plan before implementation. The original add-vector-paint-primitives archive is preserved.

## Assessment

| Dimension | Result |
| --- | --- |
| Completeness | All nine tasks complete; both requirements and all five scenarios implemented and covered; completed contract-owner review approved, living specification synchronized, correction archived, and final Moon gate passed |
| Correctness | Rust and TypeScript agree on all 288 shared fixtures, including 60 explicit structural counterexamples and 64 valid fixtures |
| Coherence | Private streaming object visitors and strict private field decoders preserve fields, serialization, tags, validators, and module ownership; TypeScript production schemas unchanged |

No implementation/spec mismatch or finalization blocker remains. There are no correctness or coherence warnings. All required checks passed, with the existing parallel Windows sampling limitation documented below.

## Scenario traceability

Rust test references below are in crates/editor-core/tests/vector_primitives.rs. TypeScript references are in apps/agent-bridge/tests/vector-primitives.test.ts. Both consume contracts/vector-primitives-v1.json.

| Requirement / scenario | Implementation and automated evidence |
| --- | --- |
| Canonical JSON representation enforcement / Reject positional records at any nesting depth | vector.rs deserialize_object invokes deserialize_map and only implements visit_map; six record Fields structs use deny_unknown_fields through MapAccessDeserializer. structural_fixtures_fail_before_semantic_validation rejects correctly sized arrays with both from_str and from_value, including color, point, stop, stroke, radii, path, nested paint colors/endpoints/stops, stroke paints, and every path-command point field. The shared fixtures also reject sequence envelopes for every Paint and PathCommand variant, including nested uses. Each TypeScript fixture rejects the same value. |
| Canonical JSON representation enforcement / Reject object-form string enums | LineCap, LineJoin, and FillRule deserialize String and match only exact identifiers. Shared standalone fixtures cover every canonical identifier, its object alternative, and scalar/unknown substitutes; nested stroke/path cases cover object enums. Rust structural tests assert decoding failure before validators. |
| Canonical JSON representation enforcement / Preserve strict object parsing and valid round trips | canonical_vector_primitives compares raw/Value acceptance and successful decoded values, canonical serialization, and round trips. Existing missing/unknown-field fixtures retain their outcomes. reordered_objects_and_raw_duplicate_fields_at_all_depths recursively reverses keys and injects each duplicate object field into raw JSON for every valid fixture, including internal type tags and nested objects. TypeScript valid cases assert unchanged parsed and serialized values. Existing nonfinite tests, path_limit_and_preflight, radii_use_one_independent_scale_and_preserve_input, and radius_resolution_checks_each_limiting_side remain passing. |
| Representation regression evidence / Reject alternate catalog envelopes | Test-only Catalog/Fixture map visitors, required_value fields, and string-only Kind reject positional wrappers, object kinds, and absent required fields. catalog_wrappers_require_objects_and_string_kinds exercises both Rust decoding paths plus raw duplicate fixture fields. rejects_malformed_catalog_wrappers retains metadata/unknown-field/identity checks. TypeScript rejects positional catalog/fixture envelopes, object kinds, missing fields, and contradictory structuralInvalid metadata. |
| Representation regression evidence / Prove representation parity and unchanged behavior | contracts:check runs both consumers over all 288 cases. Original 214 fixtures retain their contents/outcomes; 74 added cases expose alternate representations and standalone canonical values. Existing workspace, headless protocol, integration, and packaged smoke checks exercise state/history/revision/render workflows. No schema, migration, public operation, dependency, or renderer implementation changes belong to this correction. |

## Red-to-green evidence and resolved findings

Before record/string decoder changes, the expanded Rust suite had three failures: canonical fixture acceptance, structural rejection (including Value decoder accepted [1,0,0,1]), and positional catalog wrapper acceptance. Seven tests passed. After the decoder changes those failures passed.

Broader verification independently reproduced Serde accepting internally tagged sequences such as ["solid",{"r":1,"g":0,"b":0,"a":1}] and ["moveTo",{"x":1,"y":2}], including nested occurrences. Ten additional shared fixtures reproduced the failure before the fix. The same outer object visitor now guards private internally tagged Paint/PathCommand decoders. Existing type tags and variants remain unchanged. This fulfills the already-approved object representation restriction; it adds no public behavior.

No serde_json::Value buffering was added to production decoding. MapAccessDeserializer preserves duplicate map entries; raw duplicate tests cover the nested tagged decoding path as well. TypeScript schemas receive parsed values, so duplicate-key rejection is tested in Rust raw JSON rather than claimed for parsed JavaScript objects.

## Check results

- PASS: focused Rust vector and architecture suites, 30 tests (10 vector and 20 architecture).
- PASS: cargo fmt --check --all.
- PASS: cargo clippy --workspace --all-targets -- -D warnings on final implementation.
- PASS: cargo test --workspace -- --test-threads=1, exit code 0, including the complete core, architecture, integration, desktop, headless, and doc-test targets. Two earlier parallel workspace attempts failed the existing renderer::golden::process_tree_sampler_observes_a_child_allocation test (190 passed, one failed, six ignored in the core library), including outside the sandbox. Both an isolated rerun and the complete serial workspace run passed without code changes. The 300 ms child allocation appears sensitive to concurrent process sampling load; no test or threshold was changed or skipped. This timing sensitivity remains a limitation of the existing parallel test run.
- PASS: bun run contracts:check, including native headless/vector suites and 313 TypeScript contract/vector tests.
- PASS: bridge bun run typecheck and bun run lint (51 files).
- PASS: bridge bun run test, 377 tests in 15 files.
- PASS: bun run test:integration, nine tests.
- PASS: bun run test:smoke, four rebuilt packaged tests.
- PASS: bun run scripts/run-python-tests.ts, ten hermetic speech tests and five transcription tests.
- PASS: pinned OpenSpec strict correction validation.
- PASS: git diff --check.
- PASS: root:openspec-validate through pinned @moonrepo/cli@2.3.3 after approved archival, exit code 0. All 231 policy tests, 19 living specifications, and the CI parity policy gate passed.

Diagnostic logs in ignored local-data/vector-fix-*.log are not deliverables. The existing six ignored helper/maintenance entry points retain their status; parent tests invoke the process helpers. Reference recapture and externally supplied performance-report validation remain maintenance entry points, not new correction conformance tests.

## Review and finalization

The designated contract owner is @matiHirCab under .github/CODEOWNERS. On 2026-09-06 the user explicitly approved the completed contract-owner review of the changed catalog and native/test consumers in response to the final review request, separately from implementation-plan approval. Both added requirements are synchronized into openspec/specs/vector-primitives/spec.md. The correction is archived at openspec/changes/archive/2026-09-06-fix-vector-json-representations/ and the final root:openspec-validate gate passed. The original implementation archive remains intact. No commits or pushes were performed.
