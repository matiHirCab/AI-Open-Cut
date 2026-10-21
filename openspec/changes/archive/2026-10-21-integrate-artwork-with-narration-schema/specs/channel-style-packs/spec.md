## MODIFIED Requirements

### Requirement: Portable closed captured native styling
A version-one pack SHALL contain bounded ordinary native component recipes at project component schema version 46, typed slot defaults, exact content-addressed font faces, declared managed-image dependencies, and optional caption-role and narration-profile references. It SHALL contain no executable templates, arbitrary filesystem references, source asset paths, provider configuration or ambient font selectors. Capture/update SHALL require source project expectedRevision and selected components, capture their complete bounded component-reference closure and referenced managed font bytes, and preserve source project/history/drafts/media bytes. Capture SHALL read only explicitly compatible source schemas46/47/48 without recovery, migration or staging; schema46 MUST omit narrationManifests and schema47 and schema48 MUST contain a valid real array. A pending transaction journal, schema45 or older, or future source SHALL fail explicitly and direct the caller to a separate recovery/open/migration action. Capture of selected derived artwork SHALL fail VALIDATION_FAILED before library publication because format1 cannot represent transitive artwork source provenance; unrelated derived assets outside the selected recipe closure SHALL NOT prevent ordinary capture. Portable component schema MUST remain46 for format1 independently of unrelated project manifest migrations. Portable import SHALL accept at most 8 MiB of UTF-8 JSON before deserializing its graph; canonical exported pack data SHALL also fit 8 MiB. Each pack SHALL allow at most 16 component recipes, 1024 total nested timeline items, 32 markers, 32 image dependencies and 32 font faces. Existing stricter native limits SHALL remain authoritative. Unknown fields, repeated raw JSON object keys at any depth (including native slot maps), container nesting deeper than64, duplicate identifiers, incompatible versions, invalid graphs/slots/bindings, audio/global-resource dependencies, unsupported artwork kinds and missing or invalid pinned fonts SHALL fail explicitly before library publication. Imports SHALL create an explicitly named independent copy; no guessed collision rename SHALL occur.

#### Scenario: Capture a caption and scene recipe without changing its source
- **WHEN** an existing project supplies component definitions with typography, color and text slots, nested components, markers and managed image/font dependencies
- **THEN** capture preserves their native semantics and exact referenced font bytes while source project, history, drafts and managed media remain byte-identical

#### Scenario: Pending recovery or legacy capture source
- **WHEN** capture observes a pending transaction journal, including a linked journal entry, or an unsupported older/future source schema
- **THEN** it returns an actionable recovery-required or validation error before any recovery/migration/staging and all source project/history/draft/media/journal bytes remain unchanged

#### Scenario: Reject unsafe or excessive packages
- **WHEN** import contains unknown executable/path fields, future format or component schema, invalid font bytes or digest, duplicate recipe identities, unsupported audio dependencies or an exceeded resource limit
- **THEN** a typed nonmutating failure occurs and no library or project data is published

#### Scenario: Literal slot names remain compatible
- **WHEN** a valid native recipe uses slot names such as __proto__, constructor or toString
- **THEN** capture, export, import and application preserve literal own-key semantics and native slot validation

#### Scenario: Preserve CP1 across schema47 narration adoption
- **WHEN** equivalent compatible source46,47 and48 projects are captured or an existing CP1 componentSchemaVersion46 pack is imported/exported
- **THEN** sources stay byte-identical, native recipes and old package digests stay intact, forged metadata46 and missing/null metadata47 fail closed, and portable componentSchemaVersion47 is rejected
