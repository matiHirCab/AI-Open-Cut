# Postarchive delivery

These steps remain pending after implementation verification/archive and are not implementation-completion assertions.

- [ ] Review the final scoped diff and applicable CODEOWNERS; obtain actual delegated review, clearly distinguish it from human approval.
- [ ] Verify commit identity and permitted dry-run push, push the isolated issue48 branch and create a draft PR under existing authorization.
- [ ] Verify all mandatory CI checks against the exact published head; retain failures and resolve only approved scoped issues.
- [ ] Report PR/head/checks and any blockers. Do not merge or deploy. After exact-head mandatory CI is green, perform the user-requested independent epic6 audit before MG-M4 work. Then continue dependency-ready canonical issues as stacked draft PRs from the verified prior branch, recording base/head/dependencies and merge order; no issue71 lane changes are authorized here. Stop and report any publication review denial with target and reason.
