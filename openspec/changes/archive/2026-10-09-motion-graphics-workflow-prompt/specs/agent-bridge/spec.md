## ADDED Requirements

### Requirement: Complete motion graphics workflow prompt
The bridge MUST advertise canonical additive prompt `create_motion_graphics` with required nonempty string projectId and request arguments. Retrieval MUST return deterministic user-role textual guidance without invoking editing, media or provider operations. Existing prompts, tool schemas, protocol1 and project schema44 MUST remain compatible.

#### Scenario: Discover and retrieve a complete workflow
- **WHEN** a source or packaged MCP client lists prompts and retrieves create_motion_graphics with valid arguments
- **THEN** it receives guidance covering capability/state inspection, typed atomic batches with creation aliases, component template slots, marker references, supported presets, audio-enabled range preview, artifact revision review, explicit export approval, undo/redo and reopen verification

#### Scenario: Reject invalid prompt arguments
- **WHEN** required arguments are absent, empty or non-string
- **THEN** retrieval rejects them without editing the project or invoking a provider

#### Scenario: Recover through canonical edit semantics
- **WHEN** the workflow encounters missing references, invalid input or revision conflicts
- **THEN** its guidance directs schema validation and reference resolution before editing, atomic rollback verification on failure, and state refresh and replanning on conflict rather than blind retry

#### Scenario: Preserve predecessor discovery
- **WHEN** the additive prompt is removed using the explicitly tested successor projection
- **THEN** every original canonical MCP identifier and predecessor digest remains unchanged
