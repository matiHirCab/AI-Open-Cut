## ADDED Requirements

### Requirement: Project-scoped agent defaults
OpenCut SHALL configure GPT-6 Astra with medium reasoning and medium planning in project-local settings without changing global configuration, other projects, trust settings, or unrelated inherited settings.

#### Scenario: Fresh OpenCut task
- **WHEN** a fresh task loads the trusted OpenCut project configuration without a higher-priority override
- **THEN** its configured model is gpt-6-astra and its reasoning and planning effort are medium

#### Scenario: Another project
- **WHEN** a task loads a different project
- **THEN** the OpenCut overrides do not apply and its existing settings remain unchanged

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
