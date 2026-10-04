## ADDED Requirements

### Requirement: Strict nested time-expression contract parity
Native and TypeScript consumers MUST accept the same closed milliseconds and marker time-expression variants. Unknown fields inside either variant MUST be rejected before any mutation or migration publication, including standalone and ordered batch edits. Native/headless request parsing MUST retain existing non-retryable `INVALID_ARGUMENT`; the public TypeScript schema MUST retain strict Zod rejection; MCP MUST retain its existing SDK input-validation `CallToolResult` with `isError:true` and validation text, without a structured core error/code/retryability object and without dispatching a headless edit request. This correction MUST NOT normalize or change those distinct existing transport error representations. Supported fields, marker scope/ambiguity resolution, numeric compatibility, alias behavior, optimistic revisions and error precedence for otherwise valid requests MUST remain unchanged. Persisted marker expressions MUST use the same closed variant decoder and existing malformed-document/recovery error mapping rather than silently discard unknown fields. Canonical version1 fixtures MUST govern positive and negative variants across Rust, TypeScript and public transports; this correction MUST NOT add a new expression variant or permit raw executable expressions.

#### Scenario: Reject unknown members in both expression variants
- **WHEN** standalone or batch input supplies an extra member in a milliseconds or marker time-expression object, including an earlier otherwise-valid aliased batch operation
- **THEN** native/headless rejects with existing non-retryable INVALID_ARGUMENT, the public TypeScript schema rejects strictly, and MCP preserves its existing SDK input-validation isError/text result with no structured core error or headless edit dispatch; no surface publishes a project/history/revision, committed alias or managed-resource change

#### Scenario: Preserve supported timing and failure precedence
- **WHEN** a valid numeric or scoped marker expression is submitted at the current revision, or an otherwise-valid request has a stale revision, missing marker/item, locked track or invalid alias
- **THEN** valid timing retains its existing behavior and invalid operations retain their established typed error, retryability and complete rollback

#### Scenario: Reject malformed persisted expression without rewriting
- **WHEN** current, component-local or retained undo/redo source documents or a committed recovery journal contain a marker expression with an unknown member
- **THEN** existing document/recovery failure mapping rejects the closed shape before authoritative publication/replay and preserves the complete previous durable generation
