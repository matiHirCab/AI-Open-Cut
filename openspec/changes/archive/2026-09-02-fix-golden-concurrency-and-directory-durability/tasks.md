## 1. OpenSpec contract

- [x] 1.1 Propose and approve `fix-golden-concurrency-and-directory-durability` with explicit serialization, complete ancestor durability, and ambiguous-install requirements.

## 2. Golden harness implementation

- [x] 2.1 Add the persistent RAII `fs2` coordination lock and hold it for the complete native golden invocation.
- [x] 2.2 Collect and synchronize every retained-file ancestor through the generation root in deepest-first order.
- [x] 2.3 Track confirmed generation installation ownership and preserve destinations after unconfirmed installation errors.
- [x] 2.4 Update fixture documentation for multiprocess serialization, persistent coordination, and full-tree directory durability.

## 3. Focused regression tests

- [x] 3.1 Add subprocess coverage proving a second reconciler blocks on a live stage and proceeds only after lock release.
- [x] 3.2 Cover overlapping same-digest publication and lock release after pre-commit error and controlled panic.
- [x] 3.3 Cover complete deepest-first ancestor collection and preserve existing durability, rollback, reopening, unknown-path, and Linux-digest tests.

## 4. Repository verification

- [x] 4.1 Run `cargo fmt --all --check`, strict workspace Clippy, workspace tests, and the focused golden tests.
- [x] 4.2 Run headless lifecycle, Bun and Python contract suites, and applicable repository validation commands.
- [x] 4.3 Run multiprocess golden tests on Windows and Linux, plus mandatory Linux native golden conformance with performance report schema 2 and no skip fallback.
- [x] 4.4 Run strict OpenSpec validation and `git diff --check`.

## 5. OpenSpec completion

- [x] 5.1 Verify the implemented change against proposal, design, specification, and tasks.
- [x] 5.2 Synchronize the living `render-regression-fixtures` specification and archive the verified change.
