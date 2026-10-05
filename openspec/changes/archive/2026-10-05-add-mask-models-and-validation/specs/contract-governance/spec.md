## ADDED Requirements

### Requirement: Active mask authoring contract and capability
contracts/mask-models-v1.json MUST own active mask identifiers, exact required fields, finite inclusive limits, valid/invalid fixtures and metadata-only status, referencing existing canonical VectorPath/Paint/Transform2D semantics. Its governed Rust/Serde, TypeScript/Zod, headless and MCP consumers MUST agree. The additive mask_models_v1 capability MUST appear in editor status independently of renderer readiness; protocol major 1 and existing operation names/tool counts/annotations MUST remain. Project schema reporting MUST identify 32. Existing active animation-channels, animation-presets, extended-visual-animation, inherited-animation-timing, motion-blur-sampling and initial-motion-preset-pack catalogs MUST report current projectSchemaVersion32, synchronized with the current core and their governed consumers. Only this top-level version marker may change in those six catalogs; every feature, bound, fixture and historical source-version behavior MUST remain unchanged. A narrowly scoped 32→31 marker projection MUST reproduce independently pinned predecessor semantic digests and reject unrelated drift. Strict older consumers pinned to 31 MAY reject schema 32 responses under the existing persisted-version compatibility boundary; unchanged request acceptance SHALL NOT be misrepresented as compatibility with those old response validators. Fixture-only motion-graphics-v1 records SHALL NOT be reinterpreted as active runtime sources. Documentation MUST explicitly state typed alpha/luma masks are accepted/persisted/editable but visually inactive until #51.

#### Scenario: Detect supported mask models through unavailable rendering
- **WHEN** status reports an editor with model support but renderer dependencies unavailable
- **THEN** mask_models_v1 remains discoverable while no mask-rendering support is claimed and existing renderer readiness reporting stays accurate

#### Scenario: Reject cross-language catalog drift
- **WHEN** any fixture, field representation, nested bound, null/default rule, capability or project-version producer differs from its governed consumer
- **THEN** mandatory contract parity fails with the mismatched category before the change is considered complete

#### Scenario: Synchronize current catalog schema markers without feature drift
- **WHEN** schema32 model activation advances current project-version reporting
- **THEN** all six active animation catalogs and current consumers agree exactly on32, their narrowly projected predecessor digests remain exact, and unrelated changes or weakened historical migration guards fail conformance

### Requirement: Reviewed additive mask MCP digest transition
The expanded canonical MCP surface digest MUST advance only after reviewed mask schema/capability synchronization. The exact immediately preceding verified #49 expanded digest MUST remain pinned as predecessor evidence. Removing only the new capability and the masks properties from explicitly approved input/output schema locations, and restoring only enumerated projectSchemaVersion/schemaVersion response/status literal locations from 32 to 31, MUST reproduce the predecessor exactly, including existing schemas and annotations. Projection SHALL NOT hide unrelated matching keys or changes. Canonical storage references MUST retain missing/cyclic/malformed/nonlocal/unused rejection; parity tests SHALL NOT regenerate catalogs or expected digests.

#### Scenario: Prove the additive contract boundary
- **WHEN** reviewed expanded mask catalogs are projected back through the exact authorized fields
- **THEN** the predecessor digest is identical and unrelated schema/capability/annotation mutations fail parity
