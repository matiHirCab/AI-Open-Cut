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

## Paired probe review

The first instrumented fixture passed on native Windows in workflow37623651450/job112799854979 at head2fe690eaaeee13fbd37dfcbaaf52765e4a14d15e, including the original descendant cleanup assertion, in2.37seconds. This does not establish why the two original full main runs failed before PID observation. Full CI37623651478 remains in progress; the active-change inventory rejection is expected and is not acceptance.

The approved amendment adds a second Windows test preserving the original batch body byte-for-byte alongside the instrumented fixture. Both retain the same deadlines and exact process-handle assertions. Read-only ToolHelp snapshots expose only the owned worker's descendant names/IDs, cap64 descendants and16384 enumerated records, report snapshot failures and truncation, and never open/terminate unrelated processes. Success reports elapsed startup and owned process evidence; timeout/early terminal events report corresponding evidence. Portable tests cover reversed ancestry, unrelated-tree filtering, process limits and malformed-byte expansion. Focused Linux tests now pass9/9; strict focused Clippy, Windows-target compilation and the22 workflow assertions pass. Actual paired native execution remains pending.

Local full bridge unit execution preserved three failures: default-target headless binary absent due isolated cargo target setup; two Linux process.kill(PID,0) expectations failed and require owned-process inspection before attribution. Standard Ubuntu CI on the prior checkpoint passed all required correctness checks. These results do not substitute for exact final-head acceptance.
