## ADDED Requirements

### Requirement: Ordered-effect retained generation conformance without new migration
Existing ordered-effect current/undo/redo/draft generations and already-supported historical source adoption MUST preserve exact effect IDs/order/values and unrelated revisions/order/resources/provenance. Evidence MUST use genuine reviewed schema27-or-later sources where effects are valid and the actual verified predecessor adoption, not an invented schema advance or downlabeling new masked/matted/blended data. Invalid retained effect/target/source data MUST fail existing preflight atomically; valid no-rewrite reopen and source-matched stale/unavailable-base draft behavior MUST remain unchanged. No migration/schema/journal-format change is authorized. Under the explicit reviewed atomic-adoption amendment, effects-bearing edits and retained ordered generations MUST reuse the existing staged composition transaction owner so failed operations cannot publish incidental adoption.

#### Scenario: Preserve both ordered generations through existing adoption
- **WHEN** current and retained history contain opposite valid effects and an applicable old-base draft with managed resources/provenance in a genuine supported source
- **THEN** existing adoption preserves all ordered generations and native semantics, undo/redo/reopen reproduce correct arrays/pixels and subsequent reads do not rewrite state

#### Scenario: Reject invalid retained sources before publication
- **WHEN** any current/undo/redo/component/draft generation has invalid effects/targets/future source or genuine pre27 introduced-effect data
- **THEN** existing stable failure preserves complete authoritative/draft/resources and published-journal recovery evidence retains one complete valid generation without new transaction semantics

#### Scenario: Adopt ordered edits within one existing transaction
- **WHEN** an effects-only edit, alias batch or draft mutation on a supported legacy project fails before the existing journal commit, including a removed or retyped targeted effect
- **THEN** all authoritative/history/draft/managed-resource bytes remain unchanged; after the existing journal commit, recovery yields one complete correct ordered generation without new journal or migration semantics
