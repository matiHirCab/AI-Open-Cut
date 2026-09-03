## 1. Correct benchmark isolation and validation

- [x] 1.1 Add an explicit capture mode and prove ordinary conformance invokes no benchmark probes while warm-up and measured benchmark captures retain the required intent/sample matrix.
- [x] 1.2 Validate aggregated reports before use and add strict read-back validation of the exact report-only artifact.
- [x] 1.3 Replace partial CI JavaScript validation with the editor-core strict report-file validator before upload.

## 2. Fail closed for fixture generations and measured processes

- [x] 2.1 Restrict migration validation to revision-2/schema-2 and strictly validate revision-3 selected and inactive generations.
- [x] 2.2 Add malformed-current, supported-legacy, and unsupported-generation tests proving rejection occurs before capture, replacement, or cleanup.
- [x] 2.3 Add stage-aware probe injection and independent encode/decode/composite failure tests proving ordering, early termination, cleanup, unchanged project state, and unchanged selection.

## 3. Synchronize and verify

- [x] 3.1 Update renderer fixture documentation without changing schema 3, stage-definition version 1, or deterministic references unless recapture review requires it.
- [x] 3.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, native Linux golden conformance/report validation, agent-bridge contract/type/lint/unit/integration/smoke checks, and hermetic Python worker tests.
- [x] 3.3 Run `moon run openspec-validate`, verify implementation against this change, sync the delta into the living specification, and archive only after every check passes.
