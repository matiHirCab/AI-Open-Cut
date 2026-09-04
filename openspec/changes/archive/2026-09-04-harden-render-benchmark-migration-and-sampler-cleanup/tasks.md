## 1. Complete legacy fixture validation

- [x] 1.1 Add strict private schema-2 report types and validate every retained identity, sampling, aggregation, unit, policy, and finite timing field.
- [x] 1.2 Share canonical manifest metadata/reference validation across current and legacy generations while dispatching to the version-appropriate performance validator.
- [x] 1.3 Add valid reconciliation plus malformed report, manifest, schema-pairing, and inactive-cleanup tests.

## 2. Guarantee sampler cleanup

- [x] 2.1 Make process-tree sampler shutdown idempotent, join exactly once, and invoke it from both explicit finish and non-panicking `Drop`.
- [x] 2.2 Add worker-exit evidence for normal completion and panic unwinding while retaining benchmark stage failure cleanup tests.

## 3. Synchronize and verify

- [x] 3.1 Update fixture documentation and living requirements without changing schema 3, stage-definition version 1, or golden references.
- [x] 3.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, native Linux golden conformance/report validation, agent-bridge contract/type/lint/unit/integration/smoke checks, hermetic Python worker tests, and `git diff --check`.
- [x] 3.3 Run strict OpenSpec validation, verify implementation against this change, sync its requirements, and archive only after every check passes.

Verification note: formatting, strict Clippy, workspace tests, bridge contract/type/lint/unit/integration/smoke checks, hermetic Python tests, diff checks, strict OpenSpec validation, requirement traceability, and schema-3 identity cases pass. Ubuntu 24.04 with Rust 1.97.0 and FFmpeg 6.1.1 passed native golden conformance, headless lifecycle parity, and strict external report read-back on 2026-09-04. Fixture and golden hashes were identical before and after verification.

## 4. Enforce canonical schema-3 work counts

- [x] 4.1 Validate the exact revision-3 stage-work matrix for frame preview, audiovisual range preview, and final export without changing schema or stage-definition versions.
- [x] 4.2 Update shared observation fixtures and add strict read-back mutations below and above each nonzero work count, plus multiple nonzero mutations for canonical zero-valued fields.
- [x] 4.3 Re-run all required verification, sync the expanded requirement, and archive only after native Linux conformance and report validation pass.

## 5. Complete schema-3 identity validation

- [x] 5.1 Reject empty and whitespace-only schema-3 `gitRevision` strings while retaining null and arbitrary nonblank string support.
- [x] 5.2 Require the schema-3 `gitRevision` field to be present and add strict file read-back coverage for explicit null, nonblank, missing, empty, whitespace-only, and non-string Git identities.
- [x] 5.3 Re-run all required verification, sync the expanded requirement, and archive only after native Linux conformance and report validation pass.
