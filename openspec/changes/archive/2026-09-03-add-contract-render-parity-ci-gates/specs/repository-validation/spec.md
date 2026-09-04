## ADDED Requirements

### Requirement: Stable motion-graphics foundation status
Repository validation MUST publish one stable aggregate foundation status that depends on both the dedicated contract-parity and render-parity statuses and cannot succeed when either dependency fails, is cancelled, or does not run.

#### Scenario: Both parity boundaries pass
- **WHEN** the dedicated contract-parity and render-parity statuses both succeed
- **THEN** the aggregate motion-graphics foundation status succeeds and is available as the single branch-protection target

#### Scenario: One parity boundary does not pass
- **WHEN** either dedicated parity status fails, is cancelled, or does not run
- **THEN** the aggregate motion-graphics foundation status does not succeed

### Requirement: Automated CI gate policy validation
The repository's pinned validation workflow MUST structurally verify the stable parity job identities, their dependency relationship, the authoritative contract and render commands, deterministic render configuration, strict report validation, and publication of the validated report path.

#### Scenario: Validate the required gate structure
- **WHEN** repository validation reads a workflow containing every required job, dependency, command, deterministic setting, and report publication rule
- **THEN** the CI gate policy check succeeds without executing application behavior

#### Scenario: Detect a weakened gate
- **WHEN** a required parity job, dependency, command, deterministic setting, report validator, or exact report upload is removed or changed incompatibly
- **THEN** the CI gate policy check fails with the missing invariant before the workflow change can be accepted
