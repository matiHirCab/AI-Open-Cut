## Why

OpenCut has repository-specific contract, rendering, and bridge verification rules, but no focused skills that help an agent find and apply them. Contributors also need a consistent local branch workflow that uses GitHub context without assigning a `codex/` prefix. The web and API apps use Cloudflare Workers, for which a project-scoped reference skill would be useful. The project also pins Astra despite the user's preference to select a model independently, causing the agent-context regression test to conflict with the current checkout.

## What Changes

- Add four project-local Codex skills: `opencut-contract-change`, `opencut-render-regression`, `opencut-bridge-verification`, and `opencut-branch`.
- Install Cloudflare's `workers-best-practices` skill for this project only and record its source and installation method.
- Remove the project-local `model` pin so model selection is inherited, while retaining medium reasoning and planning defaults. Update the living-setting expectation in the focused test and contributor guide after approval.
- Keep the new skills subordinate to the mandatory OpenSpec approval, implementation, verification, and archival lifecycle. Give each a narrow trigger and links to current repository authorities instead of copying entire policies.
- For requested local branch creation, use `gh` to inspect repository and issue context, then `git switch -c`; use `feat/`, `fix/`, or `chore/` with an issue number when available and a short topic. Check the working tree and preserve pre-existing changes. Handle unavailable GitHub authentication without inventing issue context.
- Non-goals: changing application behavior, public contracts, persisted schema, CI policy, global skills or configuration, reasoning-effort defaults, project plugin overrides, existing OpenSpec workflow skills, or the unrelated `compact-mcp-surface-catalog` work.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `agent-context-efficiency`: Add focused, project-scoped agent guidance and safe local branch creation; remove the project model pin while retaining the medium effort settings and contributor safeguards.

## Impact

- New project-local skill files, a project-scoped third-party skill installation, and a focused edit to `.codex/config.toml`, `docs/spec-driven-development.md`, and the agent-context regression test. A focused skill validation test covers mechanically checkable scenarios.
- No public API, cross-language contract, persisted data, migration, or runtime application behavior changes. Existing Git branches and working-tree edits remain untouched by the installation.
- The current unrelated active change prevents a clean archive-only protected-gate result; verification must report that condition without modifying or archiving it.
