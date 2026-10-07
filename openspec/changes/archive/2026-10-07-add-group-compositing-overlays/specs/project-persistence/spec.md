## ADDED Requirements

### Requirement: Atomic schema37 aggregate and overlay activation
Schema36 and all prior supported generations MUST migrate current state, every retained undo/redo snapshot and component definition under the existing project lock and complete staged/journal/resource owner to37, adding no controls by default and preserving revision, IDs, provenance, media hashes/bytes and clocks. Source<=36 MUST reject screen_flash/particle_overlay, non-null stored clip and nonempty Group/ComponentInstance effects before migration/adoption, including hidden/unused/component/history state. Structurally scoped guards MUST not interpret unrelated text/metadata as introduced fields. Source37 MUST validate every closed field/resource/work constraint; future versions MUST fail closed. Matching staged generations MUST publish atomically; second valid37 reopen MUST not rewrite bytes. Existing historical source guards MUST remain unchanged.

#### Scenario: Migrate complete authentic generations without controls
- **WHEN** authentic36 and historical current/history/component state with managed resources reopens
- **THEN** complete37 adoption preserves semantic/revision/resource/inventory identity and undo/redo; deterministic second reopen preserves every authoritative byte

#### Scenario: Reject premature malformed or future generations
- **WHEN** new clip/kinds/group effects appear in<=36 or invalid37/future current/history/components, including hidden/unused objects
- **THEN** owning guards reject before publication with complete filesystem/resource snapshots unchanged and ordinary matching text accepted

### Requirement: Source-matched aggregate drafts and complete recovery
Every source-matched draft candidate MUST undergo the same source-version/current model, resource and sampled-work preflight before complete adoption. Unavailable-source drafts MUST retain the existing preserved/conflict disposition without enabling controls. Newly clip-bearing geometry-only edit candidates MUST use canonical admission rather than bypassing raster/resource preflight. All applicable staging rename, journal publication, current/history/component/resource/hash/raster inspection and draft replay failure points MUST preserve or recover one complete valid generation; authoritative state/history/resources MUST never mix generations. A failed later standalone/batch/draft operation MUST preserve revision, undo/redo, drafts and managed bytes exactly. Existing complete resource owner SHALL remain sole authority.

#### Scenario: Validate matched drafts and preserve unmatched disposition
- **WHEN** source-matched current/history/component draft replays clip/overlay/aggregate controls or an unavailable source remains retained
- **THEN** premature/malformed/excess work rejects atomically, valid controls adopt completely and unmatched drafts retain existing conflict behavior

#### Scenario: Recover all applicable interrupted boundaries
- **WHEN** an injected staged/journal/rename/resource/hash/draft/geometry-only raster failure interrupts37 adoption or edits
- **THEN** complete inventory snapshots prove atomic old/new valid generation or recoverable typed failure without resource loss or authoritative partial output
