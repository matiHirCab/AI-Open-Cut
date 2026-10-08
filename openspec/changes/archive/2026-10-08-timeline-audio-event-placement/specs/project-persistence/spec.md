## ADDED Requirements

### Requirement: Atomic schema41 semantic audio provenance adoption
Core SHALL adopt schema41 from1..40 current and retained undo/redo together under existing locked prepared generation transactions. Source40 populated soundDefinitions and source39+ buses/routes MUST be retained exactly; only source<40 libraries and source<39 buses SHALL receive prior empty/default adoption. Old media MUST omit optional audioEvent metadata without a new audible change. Source<41 event metadata, malformed current41 provenance or unknown future schema MUST fail before authoritative rewrite. Source-matched drafts and complete existing recovery/publication fault rules MUST retain all previous generation/resource bytes on rejection before the durable commit point and preserve/recover the exact complete target generation after durable commit.

#### Scenario: Upgrade populated40 and all older retained states
- **WHEN** current40 with definitions/routing and source1..40 undo/redo are opened or successfully edited
- **THEN** every snapshot reaches41 deterministically without erasing source40 libraries or source39+ routing and old media output remains unchanged

#### Scenario: Reject legacy forgery future and failed adoption
- **WHEN** a pre41 source contains event provenance, a future/malformed source is opened or an invalid/stale placement/draft/publication request fails
- **THEN** adoption is not published and current/history/draft/resource generations remain exact
