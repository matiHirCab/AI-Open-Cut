## ADDED Requirements

### Requirement: Governed additive alignment marker contracts
The canonical speech-alignment-markers-v1 fixture SHALL govern closed policies, selection/naming/timing bounds, core/headless edit, MCP standalone and batch schemas, optional generated-speech insertion policy and speech_alignment_markers_v1 capability. Native, TypeScript, live MCP and workflow consumers MUST agree. Protocol 1, schema 38, prior stable errors, provider contracts and existing result shapes MUST remain compatible. Manual additive synchronization MUST capture the exact #61 predecessor raw catalogs/expanded MCP digest, remove only reviewed additions before evaluating that predecessor and every existing historical proof, and retain all unrelated drift negatives and existing contract gate consumers. CODEOWNER review and all required exact-head CI MUST pass before issue completion.

#### Scenario: Govern every public surface
- **WHEN** complete contract parity and integration/smoke inspect the new policies and actual registered tools
- **THEN** canonical positive/negative fixture cases agree across core/headless/Zod/MCP/batch/speech insertion and the new capability is visible

#### Scenario: Preserve all predecessor semantics and negatives
- **WHEN** the reviewed additions are projected from current contracts or unrelated historical contract content is tampered with
- **THEN** the exact #61 predecessor and every older proof remain valid, while unrelated tampering is rejected without weakening the protected gate
