## 1. Specification

- [x] 1.1 Create and approve the OpenSpec proposal, modified requirement, design, failure cases, compatibility statement, and verification plan for generation durability.

## 2. Durable Generation Publication

- [x] 2.1 Synchronize the validated manifest and every declared retained file, plus required generation directories on Unix, before installation can advance.
- [x] 2.2 Install a new digest durably with Unix rename and parent-directory synchronization or Windows `MoveFileExW` write-through, and fail closed on unsupported targets.
- [x] 2.3 Revalidate and resynchronize existing digest generations, keep all generation-durability failures before pointer commit, and roll back only a generation created by the current invocation with bounded reconciliation as fallback.
- [x] 2.4 Add distinct content-sync and directory-install/sync fault points without changing post-pointer-commit classification.

## 3. Regression Coverage and Documentation

- [x] 3.1 Test content-sync failure with an existing pointer and during first publication.
- [x] 3.2 Test post-install directory-durability failure, immediate rollback, retained-orphan reopen cleanup, and successful durability ordering before pointer commit.
- [x] 3.3 Test reuse and resynchronization of an existing digest without deleting it on failure, and retain all pointer recovery, reconciliation, unknown-path, and checked-in digest assertions.
- [x] 3.4 Update golden fixture documentation to distinguish generation durability, pointer commit, and cleanup recovery without changing schemas or fixture bytes.

## 4. Verification and Archival

- [x] 4.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-targets --all-features`, and the exact headless lifecycle test.
- [x] 4.2 Run the applicable Bun contract, typecheck, lint, unit, integration, packaged-smoke, and hermetic Python worker checks.
- [x] 4.3 Run the mandatory Linux native golden conformance with FFmpeg, FFprobe, and DejaVu configured; validate performance report schema 2 and confirm the selected digest is unchanged.
- [x] 4.4 Run `moon run openspec-validate` and `git diff --check`, apply `$openspec-verify-change`, sync the accepted delta into the living spec, and archive `fix-golden-generation-durability` only after every check passes.
