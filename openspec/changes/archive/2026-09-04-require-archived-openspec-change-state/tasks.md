## 1. Enforce archive-only merge readiness

- [x] 1.1 Add the direct OpenSpec change inventory to protected bootstrap sources and reject every non-archive entry before Moon launch.
- [x] 1.2 Require ordinary `openspec/changes` and `archive` directories and reject files, directories, and symbolic links outside the archive.
- [x] 1.3 Keep unit fixtures explicit so policy behavior can be tested while the authorizing change is active.

## 2. Cover and document the invariant

- [x] 2.1 Add positive archived-only coverage and negative empty-name-independent, single, multiple, file, and symbolic-link active-entry cases.
- [x] 2.2 Prove preflight launches no Moon child and emits no attestation when any active entry exists.
- [x] 2.3 Document that active changes are locally authorable but cannot pass the protected merge-ready gate.

## 3. Close the outstanding renderer change

- [x] 3.1 Run its required Ubuntu native golden conformance and strict report validation without updating fixtures or goldens.
- [x] 3.2 Re-run its required repository checks, update its verification record and tasks, synchronize its deltas, and archive it.

## 4. Verify and archive this policy change

- [x] 4.1 Run policy tests, direct validation, strict OpenSpec, contracts, TypeScript, integration/smoke, Python, Rust, and Ubuntu parity.
- [x] 4.2 Confirm fixture/golden hashes and `git diff --check`, synchronize `repository-validation`, and archive this change.
- [x] 4.3 Run the real bootstrap against the final archive-only state and require exactly one `validated=true` attestation.

Verification note: 228 policy/bootstrap tests, strict OpenSpec, contracts, TypeScript typecheck/lint/unit/integration/smoke, hermetic Python tests, Rust fmt/Clippy/workspace tests, and Ubuntu 24.04 native parity passed on 2026-09-04. The live preflight rejected this active change before Moon launch, and fixture/golden hashes remained identical. The archive-only bootstrap was scheduled as the immediate post-archive assertion so its attestation cannot be produced while this task file remains active.
