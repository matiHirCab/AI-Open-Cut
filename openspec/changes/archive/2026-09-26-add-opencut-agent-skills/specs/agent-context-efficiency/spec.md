## MODIFIED Requirements

### Requirement: Project-scoped agent defaults

OpenCut SHALL leave model selection unpinned in project-local settings so a task inherits its model from the applicable higher-priority or user setting. OpenCut SHALL retain medium reasoning and medium planning defaults in project-local settings without changing global configuration, other projects, trust settings, or unrelated inherited settings.

#### Scenario: Fresh OpenCut task

- **WHEN** a fresh task loads the trusted OpenCut project configuration without a higher-priority override
- **THEN** the project configuration has no `model` key, model selection is inherited, and reasoning and planning effort are configured as medium

#### Scenario: Another project

- **WHEN** a task loads a different project
- **THEN** the OpenCut effort overrides do not apply and its existing model and settings remain unchanged

#### Scenario: Explicit model selection

- **WHEN** the user or a higher-priority setting selects a supported model for an OpenCut task
- **THEN** OpenCut's project configuration does not force Astra or another model in its place

## ADDED Requirements

### Requirement: Focused project-scoped contributor skills

OpenCut SHALL provide project-scoped agent skills for contract changes, render regressions, and bridge verification. Each skill MUST identify its applicable work, locate the current authoritative requirements and checks, and require an approved OpenSpec change before implementation. The skills MUST preserve the mandatory verification and archival obligations and MUST NOT create a parallel source of domain or contract rules.

#### Scenario: Contract-change guidance

- **WHEN** an agent is asked to change a public or persisted contract
- **THEN** the contract skill directs it to the canonical owner and governed consumers, compatibility classification, parity evidence, CODEOWNER review, and the approved OpenSpec change before implementation

#### Scenario: Render-regression guidance

- **WHEN** an agent is asked to investigate or change rendering behavior
- **THEN** the render skill directs it to current rendering requirements, deterministic fixtures, relevant core checks, and approval before implementation

#### Scenario: Bridge-verification guidance

- **WHEN** an agent is asked to verify an agent-bridge or MCP change
- **THEN** the bridge skill selects checks according to affected surfaces, preserves complete required-gate obligations, and records failed or unavailable evidence honestly

### Requirement: Project-scoped Cloudflare guidance

OpenCut SHALL make Cloudflare's `workers-best-practices` skill available to this project for Workers work without changing global skill installation, unrelated plugin settings, or existing project agent defaults. The installed skill MUST be attributable to its upstream source.

#### Scenario: Workers task in OpenCut

- **WHEN** an agent works on the Cloudflare-backed web or API application
- **THEN** the project-scoped Workers skill is discoverable and its upstream source can be identified

#### Scenario: Other projects and settings

- **WHEN** the skill installation is reviewed
- **THEN** global skills, plugin overrides, and unrelated project settings remain unchanged

### Requirement: Safe GitHub-informed local branch creation

OpenCut SHALL provide a project-scoped branch skill for explicit local branch-creation requests. It MUST use `gh` to inspect available repository and issue context, inspect the current branch and working-tree state, and create the local branch with `git switch -c` only after establishing a safe base. It MUST use `feat/`, `fix/`, or `chore/` according to the task and a short kebab-case topic, prefixed by `issue-<number>-` when a verified issue is linked. It MUST NOT use `codex/`, invent an issue number, overwrite an existing branch, discard working-tree changes, or create a remote branch unless separately requested.

#### Scenario: Verified issue context

- **WHEN** GitHub lookup verifies issue 123 for a bug fix and the local checkout is safe to branch
- **THEN** the skill creates a unique local branch named `fix/issue-123-short-topic` using a topic derived from that task

#### Scenario: Task without an issue

- **WHEN** a feature request has no verified linked issue and the local checkout is safe to branch
- **THEN** the skill creates a unique local branch named `feat/short-topic` without an invented issue number

#### Scenario: Dirty checkout or existing name

- **WHEN** pre-existing edits could be mixed into the requested branch, the base is ambiguous, or the proposed name already exists
- **THEN** the skill preserves the checkout and resolves the ambiguity with the user before creating a branch

#### Scenario: GitHub authentication unavailable

- **WHEN** `gh` cannot read repository or issue context because authentication is unavailable
- **THEN** the skill reports that limitation, uses only independently verified local or user-provided context, and does not claim GitHub verification
