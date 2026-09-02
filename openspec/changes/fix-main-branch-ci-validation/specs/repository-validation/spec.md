## ADDED Requirements

### Requirement: Canonical default-branch validation
Repository validation MUST use the repository's canonical `main` branch as the VCS comparison base and MUST execute required gates in a clean default-branch checkout without referencing an absent legacy branch.

#### Scenario: Validate a push checkout on the default branch
- **WHEN** continuous integration checks out a push commit on `main` and invokes the required Moon OpenSpec task
- **THEN** Moon constructs the task graph using `main` as its default VCS revision and runs the pinned normalization and strict OpenSpec validators without attempting to resolve `master`

#### Scenario: Validate a pull-request checkout
- **WHEN** continuous integration checks out a pull request targeting `main` and invokes the same Moon task
- **THEN** the task uses a valid repository revision context and runs the identical pinned validators

### Requirement: Validation configuration compatibility
Repository validation MUST preserve the existing application, public contract, persisted data, and required CI job behavior while correcting default-branch resolution.

#### Scenario: Apply the branch-resolution correction
- **WHEN** the canonical VCS default branch is configured for repository validation
- **THEN** no application runtime, public or persisted contract, migration, test selection, or required CI job is removed or changed
