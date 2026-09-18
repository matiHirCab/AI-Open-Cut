## ADDED Requirements

### Requirement: Governed additive styled text contracts
The canonical `styled-text-layers-v1` contract MUST declare schema 20, runtime capability `styled_text_layers_v1`, Unicode grapheme profile, ordered layer semantics, closed field shapes, inclusive limits and independent valid/invalid envelopes. Typed Rust/headless requests and responses, MCP Zod input/output schemas, capability reporting and governed contract fixtures MUST agree. Existing simple operations and aliases MUST remain valid; no existing contract identifier, error code or retryability MUST change meaning. Clients MUST be able to distinguish support by capability/schema reporting. Unsupported explicitly requested contract versions MUST fail with INVALID_ARGUMENT before mutation. Structural transport validation MUST NOT become a second implementation of core grapheme indexing, inheritance, reference validation or complexity accounting. The contract ownership catalog MUST identify every affected consumer and designated review owner.

#### Scenario: Observe runtime support and legacy compatibility
- **WHEN** a client queries capabilities and submits legacy simple-text, styled standalone and styled batch/draft requests
- **THEN** support and schema are explicit, all valid requests reach core consistently and serialized responses preserve the governed optional fields

#### Scenario: Reject unsupported and malformed contracts
- **WHEN** clients request unsupported contract versions or submit malformed/over-limit styled inputs through governed surfaces
- **THEN** canonical valid/invalid fixtures establish consistent rejection and stable errors without partial mutation
