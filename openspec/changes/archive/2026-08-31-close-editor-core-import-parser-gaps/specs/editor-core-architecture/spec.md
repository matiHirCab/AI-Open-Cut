## MODIFIED Requirements

### Requirement: Inward dependency direction
Editor-core modules MUST follow the complete documented allowed-dependency matrix from orchestration and infrastructure adapters toward domain models and canonical rules, and repository architecture checks MUST reject every undocumented internal edge expressed through direct, grouped, nested, aliased, relative, qualified, or multiline Rust imports and paths in every production item, regardless of its position around test-only items, as well as forbidden outward imports and duplicated owner implementations.

#### Scenario: Detect an inverted dependency
- **WHEN** any editor-core owner imports or references another internal owner outside its allowed matrix using a crate-root path, a root-reaching relative path, a top-level or nested use group, an alias, or another supported parsed Rust path form in any production item
- **THEN** an automated architecture check fails with the owner and violated boundary

#### Scenario: Exclude test-only dependencies without truncating production
- **WHEN** an owner contains a test-only item and additional production items before or after it
- **THEN** the architecture check excludes only the test-only subtree and still enforces every production dependency in the file

#### Scenario: Detect responsibility duplication
- **WHEN** `renderer` reimplements scene input enumeration or command execution, or `store` reimplements asset garbage collection
- **THEN** an automated architecture check fails even if the duplicated code compiles

#### Scenario: Review a boundary exception
- **WHEN** a proposed change cannot follow an existing allowed dependency edge
- **THEN** repository review rules require an ADR update and boundary-test matrix update before the new edge is accepted
