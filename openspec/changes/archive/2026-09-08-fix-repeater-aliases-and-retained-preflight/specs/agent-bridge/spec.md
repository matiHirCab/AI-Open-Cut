## ADDED Requirements

### Requirement: Transport parity for aliased repeater replacement
Headless, source MCP and packaged MCP workflows MUST delegate aliased update_item.repeater replacements to editor-core using the existing typed schemas and protocol envelopes. No transport MUST implement its own alias substitution. Existing public shapes, capabilities, errors and retryability MUST remain unchanged.

#### Scenario: Exercise replacement across transports
- **WHEN** native headless, source MCP and packaged clients create and retarget a repeater with earlier source and item aliases
- **THEN** every transport returns equivalent resolved state, revision and alias results, supports undo/redo and reopen, and renders the resulting visible scene

#### Scenario: Preserve transaction failures across transports
- **WHEN** the replacement uses missing or forward aliases or is followed by a failing operation
- **THEN** each transport returns the established core error and preserves authoritative state and history
