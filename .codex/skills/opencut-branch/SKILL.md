---
name: opencut-branch
description: Create a requested local OpenCut Git branch with GitHub CLI context and feat, fix, or chore naming.
---

# OpenCut local branches

Use when the user asks to create a local branch. Do not create a branch merely because implementation work begins. Creating or changing a branch does not approve an OpenSpec change or authorize implementation.

1. Inspect `git status --short --branch`, the current branch, the requested base, and existing branch names. Preserve every existing edit. If the checkout has pre-existing edits, the base is uncertain, or a proposed name collides, stop and ask for the specific choice; do not stash, reset, overwrite, or discard work. Use the user's requested base when provided; otherwise use the repository's verified default branch. If that base is unavailable locally, ask before creating anything.
2. Use `gh repo view --json nameWithOwner,defaultBranchRef` for GitHub repository context. If an issue is linked, use `gh issue view <number> --json number,title,url` to verify its number and subject. If `gh` authentication fails, say so and use only independently verified local or user-provided context; never present an unverified issue as GitHub-verified.
3. Choose `feat/` for a feature, `fix/` for a defect, or `chore/` for maintenance, docs, or skills. Add `issue-<number>-` only for a verified linked issue. Form a short lowercase kebab-case topic from the task, such as `fix/issue-123-preview-crash` or `chore/add-agent-skills`. Never use a `codex/` prefix.
4. When the checkout is safe and the name is unused, create the local branch with `git switch -c <name> <base>`. Verify the branch and report its name and base. Do not create or push a remote branch unless separately requested.
