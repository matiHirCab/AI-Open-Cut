## ADDED Requirements

### Requirement: MCP grid rollback reaches domain execution
Source and packaged MCP regression workflows MUST use valid operation identifiers so grid batch rollback exercises editor-core domain execution. A batch creating a grid followed by delete_item for a missing item MUST return non-retryable ITEM_NOT_FOUND with unchanged project content and revision. An otherwise valid stale-revision batch MUST return retryable REVISION_CONFLICT with unchanged state.

#### Scenario: Fail after creating a grid
- **WHEN** a real MCP client submits add_grid followed by a valid delete_item operation targeting an absent item
- **THEN** the response contains ITEM_NOT_FOUND and retryable false, with no grid or revision published

#### Scenario: Reject a stale revision
- **WHEN** a real MCP client submits a valid grid edit batch with an obsolete revision
- **THEN** the response contains REVISION_CONFLICT and retryable true and state remains identical
