## ADDED Requirements

### Requirement: Canonical runtime template slot evidence
A versioned runtime slot catalog MUST define exact slot/value/property identifiers, constraints, limits and success/failure fixtures for all eight kinds. Its owning category and all affected headless, MCP, persisted-project and capability consumers MUST be listed in contract ownership and validated against the canonical records. Existing motion-graphics-v1 preparatory records MUST remain fixture_only; documentation MUST map runtime adoption explicitly without pretending their string-only example is the complete runtime union. Protocol version 1 additions MUST preserve old requests and stable error retryability. Rust and TypeScript parity MUST distinguish structural decoding from semantic core validation and cover Unicode scalar bounds, references, effective values and compatibility. Designated CODEOWNER review MUST cover the canonical changes and consumers.

#### Scenario: Compare native and bridge evidence
- **WHEN** canonical valid and invalid runtime fixtures pass through Rust, TypeScript/Zod and real public transports
- **THEN** every consumer reports the documented acceptance stage and error, including exact limits and all value variants
