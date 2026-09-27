# Agent context efficiency

## Purpose

Define project-scoped agent defaults, reversible local plugin controls, and efficient context handling while preserving contributor safeguards and honest verification.

## Requirements

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

### Requirement: Reversible local plugin availability
OpenCut SHALL retain project-local disable overrides for the nine existing local-marketplace entries for Canva, Figma, Vercel, Sites, documents, spreadsheets, presentations, PDF, and template-creator, preserve other plugins and global settings, and document local re-enablement. Documentation and verification MUST distinguish configured local overrides from observed desktop availability. Unsupported suppression of remote/workspace-managed plugins at project scope SHALL be recorded as an accepted limitation without claiming complete plugin removal.

#### Scenario: Default plugin selection
- **WHEN** project-local plugin configuration is inspected
- **THEN** the nine existing local-marketplace keys have enabled set to false and no unrelated plugin or global setting is overridden

#### Scenario: Explicit re-enablement
- **WHEN** a contributor follows the documented local re-enablement procedure for a needed local-marketplace installation
- **THEN** its local override is set to true for a fresh task, subject to higher-priority controls, without changing global configuration or promising to control a separate remote installation

#### Scenario: Managed override
- **WHEN** a remote/workspace-managed plugin remains available despite its corresponding local-marketplace override
- **THEN** verification records the observed availability and accepted project-control limitation without inventing remote keys, editing managed/global policy, uninstalling plugins, or claiming omission succeeded

#### Scenario: Static or CLI-only evidence
- **WHEN** static configuration tests or a standalone CLI prompt show the expected local overrides or plugin absence
- **THEN** the report identifies that evidence source and does not present it as proof of remote desktop removal or actual cross-project configuration inheritance

#### Scenario: Unsupported complete-removal claim
- **WHEN** contributor documentation claims the nine local overrides guarantee removal of all corresponding desktop plugins
- **THEN** documentation regression validation rejects the unconditional claim

### Requirement: Focused context handling
Contributor guidance SHALL require targeted requirement discovery, reuse of already-read unchanged instructions, concise command results backed by full local logs, and reuse of passing check evidence only while its relevant inputs remain unchanged and no explicit rerun is required.

#### Scenario: Routine requirement discovery
- **WHEN** an agent investigates current behavior
- **THEN** it locates relevant files and requirement headings before reading relevant sections and excludes archived changes from routine searches

#### Scenario: Historical evidence or changed instructions
- **WHEN** history is needed or prior instructions changed or are no longer in context
- **THEN** the agent reads the needed archive evidence or refreshes the relevant instructions; validators retain full archive access

#### Scenario: Long command output
- **WHEN** a check produces lengthy output
- **THEN** full output is retained in an uncommitted local log and the reported result includes command, exit status, summary, relevant failures, and log location

#### Scenario: Check evidence becomes stale
- **WHEN** relevant inputs, toolchain, environment, or new evidence invalidate a passing check, or a rerun is explicitly required
- **THEN** the agent reruns that check and does not reuse stale evidence to claim completion

### Requirement: Preserved safeguards and honest verification
The change MUST preserve the full OpenSpec approval, scenario coverage, verification, archival, architecture, compatibility, and required validation obligations. It MUST leave generated skills, protected checks, application behavior, and unrelated work unchanged and report missing or blocked evidence explicitly.

#### Scenario: Guidance consolidation
- **WHEN** duplicated workflow explanation is consolidated
- **THEN** every existing mandatory safeguard and required check remains in force and discoverable

#### Scenario: Unrelated active change blocks readiness
- **WHEN** the archive-only gate rejects another active change
- **THEN** the result identifies that dependency without modifying or archiving the other change or claiming full completion

#### Scenario: Usage or fresh-task evidence unavailable
- **WHEN** automated verification cannot observe fresh task settings, plugin availability, or comparable before-and-after usage
- **THEN** the report distinguishes static tests from pending manual checks and does not claim measured savings or successful runtime verification

### Requirement: Ordered verification and final merge readiness
Contributor guidance MUST distinguish implementation conformance from final merge readiness. Required implementation checks and OpenSpec conformance verification SHALL precede synchronization and archival. A protected-gate rejection caused only by this active change SHALL be recorded as expected before archival, never as gate success. After archival the unchanged protected gate MUST pass before completion is declared; any other substantive or policy failure remains blocking.

#### Scenario: Only this change blocks the pre-archive gate
- **WHEN** all other required checks pass and the complete protected-gate result rejects only this active change
- **THEN** implementation conformance verification and synchronization/archival can proceed, with final merge-readiness validation still pending

#### Scenario: Another pre-archive failure
- **WHEN** a substantive check fails or the protected gate rejects something besides this active change
- **THEN** verification reports that failure and archival does not proceed under the expected-rejection rule

#### Scenario: Successful final gate
- **WHEN** this verified change is synchronized and archived and the unchanged protected gate succeeds
- **THEN** final merge readiness is established without removing a check or changing attestation semantics

#### Scenario: Failed final gate
- **WHEN** the protected gate fails after archival
- **THEN** the report retains the failure and does not declare completion or fabricate a successful attestation

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
