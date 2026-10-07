# Diagnostic review and pending acceptance

Reviewed the complete diagnostic change against its delta requirements on2026-10-07. New behavior is confined to headless test fixtures/evidence and a separate focused Windows workflow. All original production files, public/persisted contracts, the two existing CI workflows,10-second startup deadline, exact-child handle/liveness assertions and5000ms post-crash wait remain unchanged. Terminal error/result evidence cannot become passing cleanup evidence. New fixture files are owned by the existing tempfile directory; output is bounded16KiB/source and32 events with explicit labels.

Portable tests cover actual missing/unreadable files, exact typed failure text, multibyte file/stderr/event truncation and event-count bounds. The focused workflow test checks main-only PR targeting, read-only permissions, pinned Bun/Rust matching.prototools, the exact Windows fixture selection, diagnostics visibility, and absence of failure suppression/conditional skips. It uses the existing repository bunfig.toml with dotenv loading disabled and never invokes Moon before policy validation.

Passing preparation checks:

- `cargo fmt --check --all`.
- `cargo clippy -p opencut-headless --test render_worker -- -D warnings` (log:/tmp/opencut-windows-diagnostic-clippy.log).
- `cargo test -p opencut-headless --test render_worker`:6/6 Linux tests, including3 new portable evidence tests (log:/tmp/opencut-windows-diagnostic-linux-tests.log). The Windows-only cleanup scenario is not executed on Linux.
- `cargo check -p opencut-headless --test render_worker --target x86_64-pc-windows-msvc`:passes with pinned1.97.0 Windows std target (log:/tmp/opencut-windows-diagnostic-crosscheck.log). Cross-check is compilation evidence, not Windows startup/process proof.
- `bun --config=bunfig.toml --no-env-file test scripts/windows-renderer-startup-diagnostic.test.ts`:1/1,22 assertions.
- Strict pinned all-spec validation passes (log:/tmp/opencut-windows-diagnostic-spec-validation.log).
- `git diff --check`; no diff in existing required CI workflows.

Full repository suites, actual Windows evidence, cause-specific correction, independent final conformance review, sync/archive, final protected policy and exact-head required CI remain pending. This investigative draft must not be described as a resolved baseline or merge-ready. Its active change intentionally blocks protected merge readiness until correction/conformance are verified and archived. Original failed Windows logs remain preserved in the original issue60 checkout's blocker dossier.
