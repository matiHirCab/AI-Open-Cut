## Context

The repository has eleven generated OpenSpec skills under `.codex/skills/`, mandatory spec-driven implementation rules in `AGENTS.md`, and project-scoped Codex settings in `.codex/config.toml`. Contract, render, and bridge checks are spread across living specs, ADRs, contracts, and package scripts. The checkout currently has unrelated edits and an active `compact-mcp-surface-catalog` change. `gh` is installed, but its saved token currently fails authentication.

## Goals / Non-Goals

**Goals:** Add narrowly routed, low-context skills for three recurrent workflows; install one upstream Workers skill only in this project; make requested local branch creation predictable and safe; remove the project model pin while retaining medium effort defaults; preserve OpenSpec and unrelated checkout state.

**Non-Goals:** Add a new implementation shortcut, create a branch as part of skill installation, change any application or wire behavior, modify generated OpenSpec skills, change global settings or project effort defaults, or resolve unrelated work.

## Decisions

1. **Project-local skill entrypoints.** Add the four OpenCut skills as concise `SKILL.md` files in `.codex/skills/`. Frontmatter descriptions will distinguish contract change, render regression, bridge verification, and explicit branch creation. Link to `AGENTS.md`, `docs/spec-driven-development.md`, applicable specs, ADRs, and existing commands at use time. Do not embed copies of long policies or add scripts unless repeated deterministic mechanics prove necessary. This keeps their guidance current and avoids a second authority. One broad catchall skill was rejected because it would load irrelevant guidance for ordinary tasks.
2. **Project-scoped upstream skill.** Use the Skills CLI with explicit Codex agent and project scope to install `cloudflare/skills` `workers-best-practices`. Inspect the exact destination and installed content, retain upstream attribution, and avoid `.codex/config.toml` and global locations. If package/network access is unavailable, do not substitute unreviewed copied instructions; report the installation as blocked. A global installation was rejected because the requested scope is this repository.
3. **Local branch flow.** The branch skill triggers on a request to create a branch, not every implementation task. It reads `git status`, current branch, existing names, and the requested base; uses `gh repo view` and `gh issue view` where applicable; then invokes `git switch -c` locally. Map new features to `feat/`, defects to `fix/`, and maintenance/docs/skills to `chore/`; normalize the topic to a short lowercase kebab-case slug. If the checkout is dirty with unrelated edits, the base is uncertain, or the name collides, stop for a specific decision without stashing, resetting, or moving user work. If `gh` authentication fails, use only verified local remote/default-branch information or user-supplied task context and say what was unverified. Remote branch creation and pushing remain separate requested actions. Creating remote refs through `gh api` was rejected because the user selected a local branch flow.
4. **Verification of instruction artifacts.** Run the skill creator's structural validator and a focused automated check of machine-checkable metadata and installation scope. Walk through the four branch scenarios and the three workflow-routing scenarios against the written instructions. Agent adherence cannot be proved by static tests alone; record scenario walkthroughs as manual evidence and do not claim runtime behavior from text checks.
5. **Inherit model selection.** Delete only the `model` key from `.codex/config.toml`; retain `model_reasoning_effort = "medium"`, `plan_mode_reasoning_effort = "medium"`, and all plugin overrides. Update the focused test to require absence of a project model key and the two effort defaults, and revise the contributor guide to describe inherited model selection and the limits of static desktop evidence. Replacing Astra with another fixed model was rejected because it would retain the restriction. Removing effort settings was rejected because the user explicitly chose to keep them.

## Risks / Trade-offs

- **Instruction drift:** Link to living requirements and existing commands; avoid copied requirement text and recheck links during validation.
- **Third-party skill changes:** Review installed files and source before accepting them; do not silently modify upstream content.
- **Branch name or base ambiguity:** Fail without changing the checkout, then request the missing decision.
- **Unrelated active change:** Preserve its files. Report protected-gate rejection due to that change as a separate blocker, not as a passing check.

## Compatibility, Rollback, and Verification

No public or persisted contract, migration, application runtime, or provider protocol changes. Removing the five project-scoped skill installations and restoring the previous project model setting rolls back this agent-guidance change; no user data migration is needed. After approval of the revised artifacts, validate skill structure and model-setting scenarios, run strict OpenSpec validation and all repository-required final checks from `AGENTS.md`, verify conformance with `$openspec-verify-change`, then synchronize and archive only this change when the protected-gate policy permits it. A missing pinned CLI, missing required check, or unrelated active-change gate failure must be reported rather than bypassed.
