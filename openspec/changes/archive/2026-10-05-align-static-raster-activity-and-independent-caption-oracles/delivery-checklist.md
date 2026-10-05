# Separate delivery and hosted acceptance obligations

This checklist is prospective at plan approval, not an attestation of completion. Authoritative actual delivery status lives in the external evidence checkpoint and API/raw-gate proofs. It does not require a commit after hosted checks merely to update implementation tasks.

1. After reviewed conformance, all mandatory local gates and verified sync/archive/postarchive gates pass, commit only scoped reviewed source/spec/docs.
2. Verify exact committed executable-input identities and clean worktree. Create and verify a separate full-history bundle, preserving every earlier bundle.
3. Ordinary push the existing branch/draftPR141 and update the accurate PR description with actual changes, tests, retained failures, reuse limits and review provenance. No forcepush, merge or deployment. Any publication review denial stops the attempted action and is reported with exact target/reason; no workaround.
4. Verify exact corrected published-head all11 jobs and overall run actually succeed. A failed or missing required check blocks acceptance; no blind reruns, fabricated green status, threshold/budget/reference weakening or favorable-environment substitution.
5. Verify PR exact head, draft/open/unmerged status and source/tool evidence identities. Report delegated approvals honestly, with designated human CODEOWNER review separate and pending where unavailable.
6. Keep issue49 held until corrected audit acceptance. Preserve original fb750/c030 failures and all diagnostic artifacts regardless of later success.
