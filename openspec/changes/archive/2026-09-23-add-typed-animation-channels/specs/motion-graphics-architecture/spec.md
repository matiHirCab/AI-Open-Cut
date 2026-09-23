## ADDED Requirements

### Requirement: Explicit channel activation boundary
Editor-core MUST own channel compatibility, validation, and evaluation. A cataloged channel without implemented scene and renderer semantics MUST not be accepted into a project. Headless, bridge, and desktop layers MUST submit typed data and translate core results without parallel property validation. Activation MUST preserve one evaluated behavior across frame preview, range preview, and export.

#### Scenario: Reject an inactive channel before side effects
- **WHEN** a caller submits a cataloged channel whose target property is scheduled for a later milestone
- **THEN** core rejects it before revision commit or render artifact creation
