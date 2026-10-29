# Catalog amendment verification and conformance — 2026-10-29

Base2768cd00bec17b681e8342a4a7c51e3750bd28ec, PR233 remains draft. Approved addenda and root input pins remain frozen with delegated approval. Main remainsb350b5cc; no integration/rebase or PR229 edit.

## Completeness

Amendment tasks4.1–4.3 complete for component publication;4.4 requires publication plus exact-head CI observation and remains pending. Original diagnostic tasks2.2/2.3 and3.x remain open; native attribution/correction is not established. Protected archive-only gate stays failed. This is an investigation component, never whole-change completion or archive readiness.

## Correctness and traceability

- Reuse scenario: contracts.test.ts two block-local predecessor bindings each serve the unchanged count and pinned hash. Static AST inlining preserves all20 assertion expressions, with outside-case content byte-identical; expansion calls17→15, DSP/event projection calls2→1 each. Pure baseline profiling retained in serial-diagnostic.md/checkpoint references demonstrates unchanged input and identical paired outputs,17/19 clone operations per predecessor. The removed duplicated work is36 clones and3868 internal serializations;17.3MB describes serialized representations, not measured RAM savings. No resolver/projection function or shared input changed.
- Independent/invalid-input scenario: current-catalog independent first/second expansions and all historical hashes/counts, raw pins and malformed/drift/repeated-reference controls remain unchanged. Focused file and full units plus contracts subset pass with those assertions. No fixtures/catalogs, runtime helpers, dependencies, timeout/profile/worker config or production changes.
- Windows scenario: PENDING exact published-head Windows evidence. Local Linux runs do not satisfy Windows acceptance. Existing Windows5000ms failure remains in run39598451825/job119216545488. If it recurs, preserve failure and return for review; no retry-until-green or scope expansion.

## Checks and limitations

- Frozen install passed147 packages without lock changes; typecheck and lint passed323 files.
- Initial concurrently launched focused run failed2/44 (catalog6410ms, readiness6490ms); preserved. Authorized once-per-tree serial experiment passed44/44 baseline and changed; catalog3242ms vs2280ms. Order/cache/resource confounds remain; no causal Windows claim.
- First broad units FAILED64/passed991/skipped15: native headless absent from new checkout, plus known PID1 zombie visibility. Diagnostics distinguish those preparation failures from the catalog case, which passed. Preserved before addressing prerequisites.
- Built only current-base debug headless and release package, reusing generated Cargo cache; original artwork debug/release binaries preserved under /tmp/opencut-pr233-preserved-binaries. No workspace-wide Rust build launched. An existing owned child-subreaper launcher, documented by the original approved diagnostic, changes only its own child adoption/reaping; it does not modify tests, assertions, timeout or native behavior. Prepared units passed1055, existing optional skips15 (83 passed files,10 skipped files);58 adopted descendants reaped.
- Source integration passed43/43; assembled packaged smoke passed40/40. Both use current-base native binaries and same unchanged acceptance, under owned reaper; each reaped8 children.
- Full unchanged contracts:check PASSED529 Rust cases across40 result groups plus705 TypeScript cases in46 files;60 adopted descendants reaped. No aggregate command weakened or bypassed.
- Python canonical runner reused existing isolated venv only after exact combined-lock marker comparison: unittest12/12 and pytest12/12 pass. No model downloads or dependency changes.
- Rust formatting passed. Full workspace Rust/Clippy not repeated locally: current remaining disk1.2GiB, historical ENOSPC retained. Prior same-base CI native inputs and Rust1.97.0 are unchanged by this test-only amendment: Ubuntu correctness job119216545676 in run39598451825 passed Rust formatting, strict workspace Clippy, workspace Rust tests and Python. This historical evidence is not new-head CI; exact-head CI remains required and will be reported separately. No full local Rust-suite pass claimed.
- Strict all-spec validation79/79 passed. Protected Moon ran538 policy tests and79 strict items, then FAILED solely with unarchived change isolate-windows-preview-disposal-safety. This expected rejection remains a failure and must not be archived away.

## Coherence / verification skill assessment

Using openspec-verify-change for the explicitly selected isolate-windows-preview-disposal-safety change: scoped implementation follows approved design, all Linux amendment assertions pass. CRITICAL before archive: incomplete native attribution/correction tasks, new-head Windows scenario, and protected archive-only gate. These remain disclosed under the explicit diagnostic draft-publication sequencing approval; no archival, formal PR approval, merge or deployment is permitted. Publication is limited to existing draft PR233, with no description/comment retry or readiness change. The initial failed executions and serial experiment remain preserved.

## Local full logs and SHA256
- `/tmp/opencut-pr233-frozen-install.log`: `03dcfb9f6c07ebcd80600697e0f89d1683d48d968ccb7cf4a1c69efe7c88335c`
- `/tmp/opencut-pr233-typecheck.log`: `8366207267355d3e3d5bf3bf6e8c94c5f93f6078c34f08973fa2b38cdda6cc92`
- `/tmp/opencut-pr233-lint.log`: `1941e5049a98b7e6d13625b7f2fde8da972a1769b711e5462e34b992b5d938ff`
- `/tmp/opencut-pr233-focused-contracts.log`: `dc58efc965245a0107c0fed643e9f1c45bdfe66e1c6bd13d5e7a8f41662559cf`
- `/tmp/opencut-pr233-full-units.log`: `3dd64290f7fc8e37ed98b7b8b667b6472e2be1ac10ac086010b48d13f8170e51`
- `/tmp/opencut-pr233-full-units-prepared.log`: `05b548ae3e26481deb74ab55797ff6881d9bb09446c3cba2ee58c631f45f1d5f`
- `/tmp/opencut-pr233-source-integration.log`: `dfb66a18f35b60a48b70adb5757c5563585b10b8d9766f824aa3761e2d04d08d`
- `/tmp/opencut-pr233-packaged-smoke.log`: `6bcfb2360f2b066d10b2af515cc6af635c013bf9acc5a9529be3b1a65b8600ef`
- `/tmp/opencut-pr233-contracts-check.log`: `6e7ee5859998a6aebc7bce49df00c29143b7620a60a91358f9c320cba001d06f`
- `/tmp/opencut-pr233-python.log`: `21e2690e5b554a4cba6113d11929f5ede8be489fc49a4b1bc664797adbdca903`
- `/tmp/opencut-pr233-rust-fmt.log`: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `/tmp/opencut-pr233-strict-final.log`: `335aabf2cbe54f2e2acc76722b8a0afde84b35713f2b854a8170313a8d0dd517`
- `/tmp/opencut-pr233-protected-moon.log`: `13446e637253711fe706087077e82eca502258115c2e922244aa5191c9d101e8`
