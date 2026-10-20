# Authorized delivery checklist

Status before publication: prepared; no remote branch or PR mutation yet. User authorization covers branch publication and draft PR, with no merge or deployment. Execution and terminal exact-head outcomes will be recorded in PR metadata after observation, without rewriting verified source just to record CI.

1. Finish all required local implementation checks, prearchive protected/strict validation, OpenSpec conformance review, approved delta synchronization/archive and final protected/strict validation.
2. Stage only three private test/fixture files, two living spec additions and this own archived change. Verify no unrelated files, workflows, contracts or governance hashes changed. Commit on fix/windows-ci-fixture-readiness based on verified main756b7b39.
3. Publish only that branch without force or mutation of peer branches. Independently verify the remote head/tree against the staged and checked local content.
4. Create a draft PR targeting main from the prepared reviewable body at /tmp/opencut-ci-pr.md, with completed observed validation and links to infrastructure evidence. Keep exact-head CI pending until fetched.
5. Monitor all required published-head check runs/workflow conclusions until terminal. Distinguish code/test failures from provisioning infrastructure, disclose any failed/skipped/blocked required gate and retain observed URLs. Do not rewrite or rerun PR228.
6. Update this PR's metadata with actual published SHA and terminal outcomes. Deliver links/evidence/blockers; do not claim main repaired before incorporation. No merge, auto-merge or deployment.

This checklist separates postarchive publication/CI acceptance from implementation tasks so the required archive-only gate can be validated before publication. Prepared checklist completion does not mean future remote operations or CI outcomes have already happened.
