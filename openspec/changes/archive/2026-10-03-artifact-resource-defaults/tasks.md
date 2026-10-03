## 1. Specification and canonical contract

- [x] 1.1 Independent spec review; record delegated approval, compatibility impact and CODEOWNER distinction.
- [x] 1.2 Add version-2 artifact content-policy fixtures, ownership, narrow MCP catalog updates and parity evidence.

## 2. Bridge implementation and tests

- [x] 2.1 Implement metadata projection, safe explicit binary reader, job-owned resource registration and bridge capability discovery.
- [x] 2.2 Add tests for every artifact-resources requirement/scenario; update source/packaged smoke and client migration documentation.
- [x] 2.3 Independent implementation review including active PR #135/#136 and #59 lane compatibility; fix findings and retest.

## 3. Verification and delivery

- [x] 3.1 Run cargo fmt --check --all, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace; bridge bun run typecheck, bun run lint, bun run test, bun run contracts:check, bun run test:integration, bun run test:smoke; relevant hermetic Python unittest suites.
- [x] 3.2 Run strict pinned OpenSpec validation and moon run root:openspec-validate before archive; verify conformance with openspec-verify-change and record evidence.
## 4. Post-verification lifecycle and delivery

After implementation verification, synchronize and archive only this change with openspec-sync-specs/openspec-archive-change, then rerun the protected Moon and strict all-spec gates. These ordered lifecycle actions are tracked in conformance.md, outside the implementation checklist so their prerequisites can be verified before executing them.

Commit the verified implementation, push the isolated branch, create a draft PR, and record exact-head terminal CI and the delivery bundle. Report limitations; do not merge or deploy. Delivery evidence is recorded in the PR and local delivery report without changing the tested head to store later CI status.
