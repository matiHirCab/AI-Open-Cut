## MODIFIED Requirements

### Requirement: Inward dependency direction
Editor-core modules MUST follow the complete documented allowed-dependency matrix from orchestration and infrastructure adapters toward domain models and canonical rules, and repository architecture checks MUST reject every undocumented internal edge expressed through direct, grouped, nested, aliased, or multiline Rust imports as well as forbidden outward imports and duplicated owner implementations.

#### Scenario: Detect an inverted dependency
- **WHEN** any editor-core owner imports another internal owner outside its allowed matrix using any supported Rust import form, or a domain/planning owner imports transport, presentation, provider, environment-configuration, FFmpeg-process, artifact-publication, or direct managed-file concerns contrary to the ownership map
- **THEN** an automated architecture check fails with the owner and violated boundary

#### Scenario: Detect responsibility duplication
- **WHEN** `renderer` reimplements scene input enumeration or command execution, or `store` reimplements asset garbage collection
- **THEN** an automated architecture check fails even if the duplicated code compiles

#### Scenario: Review a boundary exception
- **WHEN** a proposed change cannot follow an existing allowed dependency edge
- **THEN** repository review rules require an ADR update and boundary-test matrix update before the new edge is accepted
