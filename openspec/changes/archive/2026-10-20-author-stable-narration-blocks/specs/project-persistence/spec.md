## ADDED Requirements

### Requirement: Atomic schema47 narration manifest adoption
Project schema47 SHALL require a bounded narrationManifests array. Supported schema1..46 current state and retained undo/redo generations SHALL deterministically adopt empty manifests without inventing block/profile provenance under existing locked recoverable migration. Existing EditDraft v2 envelopes SHALL remain operation lists without project-schema or manifest fields; preview/rebase/commit SHALL replay their existing operations on a migrated base and enforce final manifest binding validation. Dedicated narration operations SHALL NOT become draft/batch EditOperation variants. Introduced metadata declared on an older schema, null/malformed current metadata, invalid ownership or unknown future schemas MUST fail closed before authoritative publication. Recovery and reopen MUST retain complete consistent manifest/native generations without library/provider access or media changes.

#### Scenario: Migrate current history and draft generations
- **WHEN** supported legacy current/undo/redo generations are opened or edited and existing EditDraft v2 operations replay on their migrated base
- **THEN** project generations adopt empty schema47 manifests atomically, draft envelopes keep their existing version/meaning, and all preexisting native content, intent, media and history behavior is preserved

#### Scenario: Reject forged or future generations unchanged
- **WHEN** introduced manifest metadata appears on an older schema, a current field is null/missing/malformed, ownership is inconsistent or any current/history/draft version is unsupported
- **THEN** the complete operation fails without partially rewriting project/history/draft/media or losing the prior authoritative generation

#### Scenario: Recover interrupted manifest publication
- **WHEN** a valid narration commit is interrupted after its existing durable commit point
- **THEN** ordinary recovery restores the single complete native/manifest/history generation and never a mixed or partially acknowledged block set
