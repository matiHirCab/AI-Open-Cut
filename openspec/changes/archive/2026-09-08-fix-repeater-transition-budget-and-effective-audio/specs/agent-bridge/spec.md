## ADDED Requirements

### Requirement: Transport parity for effective repeater audio rejection
Headless, source MCP and packaged MCP MUST delegate effective repeater source validation to editor-core with unchanged typed schemas and errors. Transports MUST NOT substitute slot values or classify source audio independently.

#### Scenario: Preserve effective-audio failure across transports
- **WHEN** clients submit a batch or draft whose effective asset slots introduce audio into a repeater source
- **THEN** headless, source MCP and packaged MCP return the established core error and preserve authoritative state, revision and history

#### Scenario: Preserve valid silent-source workflows
- **WHEN** a client submits a valid silent effective source with repeaters
- **THEN** existing aliases, undo/redo, reopening and rendered preview continue to work without public contract changes
