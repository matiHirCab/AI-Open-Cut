## ADDED Requirements

### Requirement: Complete bounded MCP conformance validation
Source and packaged MCP conformance clients MUST retain full published JSON Schema output validation and all workflow assertions. Any allocation correction SHALL be justified by phase/process profiling and preserve invalid-output rejection, format checks, unions, nested constraints and exact schema identity. It MUST NOT skip workflows, disable validation, increase memory limits to hide exhaustion or claim a production leak without evidence.

#### Scenario: Execute complete source and package evidence
- **WHEN** source and packaged clients exercise every existing workflow
- **THEN** all results are schema-validated and all original assertions execute within the supported host's resource budget

#### Scenario: Reject malformed output after an allocation correction
- **WHEN** output violates required fields, additional-property policy, formats, a union branch or a nested constraint
- **THEN** validation still rejects it and clients do not treat it as successful
