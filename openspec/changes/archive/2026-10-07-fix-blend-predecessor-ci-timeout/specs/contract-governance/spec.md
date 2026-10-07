## ADDED Requirements

### Requirement: Bounded historical blend negative-control rejection
Historical blend conformance MUST reject malformed or missing approved blend nodes read-only before processing unrelated large successor-schema subtrees. It MUST preserve the complete valid historical projection, every capability/literal/addition proof and pinned complete MCP digest. All29paths,87malformed cases,29missing cases, source non-mutation assertions and the existing5000ms deadline MUST remain enforced. Catalog bytes, public/persisted surfaces, production rendering and CI settings MUST remain unchanged.

#### Scenario: Reject malformed additions before unrelated predecessor work
- **WHEN** any unchanged approved blend node is malformed or missing in current successor-schema input
- **THEN** the existing exact rejection and source non-mutation checks pass without first reading unrelated expensive predecessor subtrees and without increasing the5000ms deadline

#### Scenario: Retain complete valid predecessor evidence
- **WHEN** the current valid catalog or unrelated catalog drift passes the early blend-node check
- **THEN** the entire historical projection still executes, the original complete digest distinguishes valid input from drift, and all current/predecessor catalog pins and production inputs remain exact
