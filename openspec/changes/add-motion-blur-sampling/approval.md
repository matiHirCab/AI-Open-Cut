# Bounded delegated OpenSpec decision

2026-10-01: Reviewed proposal.md, design.md, specs/motion-blur-sampling/spec.md and tasks.md against issue44 and issue43 ownership/lifecycle rules. Strict change validation passed. Approved these artifacts under the user's explicit permission to approve necessary issue44 OpenSpec specs. Scope is only issue44; no unrelated proposals approved.

This approves specification workflow decisions, not publication, account/security changes, PR131 merging, or a completed implementation. Designated cross-language CODEOWNER implementation review remains pending and must be obtained before archival. Base PR131 remains open at pinned e1010bb97174d44a5d19c61a90514bf2336845ca.

2026-10-01 reconciliation review: Approved the bounded design/tasks verification
follow-up after reviewing it against the four issue44 requirements and the
independently approved issue43 correction. Strict change validation passes.
Approval covers local reconciliation after terminal prerequisite CI, explicit
RGBA composition/exact-frame measurement with unchanged numeric oracles, and
stronger existing null/malformed/limit failure evidence. It introduces no new
feature or contract meaning. Mandatory final checks remain pending. Original
issue44 commits are preserved; publication and designated contract-owner
implementation review are still separate and unapproved.

2026-10-01 deterministic fallback amendment: Reviewed the complete bounded
proposal/design/delta/tasks amendment and its native diagnostic evidence. Strict
change validation passes. Approved under the user's issue44 OpenSpec authority:
extend the existing private expression-thread guard to the inherited instantaneous
affine branch, and include the unchanged final YUV420 conversion in the independent
analytic oracle. All numeric oracles, tolerances, CI budgets and ordinary/sampled
thread selection remain unchanged. Required verification must use the unmodified
real executable. This approval does not authorize a further PR131 update,
publication of issue44, or designated-owner implementation approval.
