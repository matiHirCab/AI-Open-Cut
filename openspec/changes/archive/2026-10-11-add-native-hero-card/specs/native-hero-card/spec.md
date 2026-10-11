## ADDED Requirements

### Requirement: Discoverable versioned read-only hero recipe
The bridge SHALL expose deterministic native-hero-card-v1 example data in the existing create_motion_graphics prompt, retaining its name/arguments and all existing MCP structural schemas, catalogs, protocol/schema versions and capabilities. Guidance MUST identify its version, supported inputs, concrete ordinary operation sequence, placeholders, canvas/timing/complexity envelope and safe managed-resource requirements. Retrieval MUST perform no edits, imports, rendering or provider calls.

#### Scenario: H1 discover and retrieve through current MCP
- **WHEN** source or packaged clients list prompts and retrieve create_motion_graphics with valid existing arguments
- **THEN** they discover the versioned native card and retrieve the same bounded recipe data without project/provider side effects or changed MCP identifiers/arguments

### Requirement: Editable native card with independent inputs
The recipe SHALL produce one 800x600 ordinary component with seven stable named native children, fixed initial paints, title and managed-image slots, and at most two demonstrated instances/fourteen expanded leaves in a 1920x1080,30fps,1000ms example. Supported title constraints MUST use existing text-slot validation and original reference artwork MUST be synthetic 64x64 image data. Slot replacements MUST affect only the chosen instance, preserving shared definition geometry/paints and the other instance. No whole-card bitmap, opaque SVG, automatic asset fitting or new palette binding SHALL replace native children.

#### Scenario: H2 create two native cards atomically
- **WHEN** an actual MCP client resolves current overlay/managed image placeholders and submits the documented aliased batch
- **THEN** one component and two instances commit once with stable native roles and independently resolved existing slots

#### Scenario: H3 customize one instance
- **WHEN** a client replaces one card's title and managed artwork using existing typed instance operations
- **THEN** only its overrides/output change, shared tracks and the other card remain unchanged, and both inputs remain ordinarily editable

### Requirement: Canonical atomic lifecycle and stored motion
Creation/customization MUST use existing core revisions, aliases, typed failures, history and resource ownership. The demonstrated entrance MUST persist ordinary supported animation channels/keyframes rather than executable recipe names. Missing managed assets, invalid late edits and stale revisions MUST preserve the complete prior project/history/managed resources and publish no partial scene or render artifact.

#### Scenario: H4 reject alias batch tail and missing assets
- **WHEN** a batch contains valid dependent card creations before an invalid trailing edit or missing safe managed artwork reference
- **THEN** the canonical typed failure leaves authoritative project/history/assets and artifact inventory unchanged with no partial component or instance

#### Scenario: H5 preserve undo redo and reopen
- **WHEN** creation/customization is undone, redone and reopened through actual MCP operations
- **THEN** exact native content, instance isolation, managed references and persisted motion are preserved with existing revision semantics

#### Scenario: H6 reject stale customization
- **WHEN** customization uses a stale expectedRevision
- **THEN** retryable REVISION_CONFLICT preserves content/history/resources and guidance requires refresh/replanning

### Requirement: Independent actual native frame and animated range evidence
Required native source and default-packaged verification SHALL exercise the same retrieved recipe at 1920x1080 with actual pinned native tools/font, verify one frame and range [0,200)ms with six frames, and compare independent geometry/paint/artwork/glyph expectations and known hidden/visible animation states. Frame/range comparisons MUST respect existing codec tolerances and read-only rendering MUST preserve authoritative content. Synthetic inputs and separate environment/source/runtime evidence MUST remain attributable; unlike hosts or unmeasured full packs MUST NOT imply portable performance or installed desktop readiness.

#### Scenario: H7 render editable instances through native MCP
- **WHEN** actual source/default-packaged clients create/customize cards, render a frame and short animated range and decode the returned artifacts
- **THEN** dimensions/frame count, independent native role/input samples, truthful temporal variation and frame/range parity pass while project state remains unchanged

#### Scenario: H8 reject blank evidence
- **WHEN** independent expected sample verification receives blank pixels or incorrect artwork/temporal states
- **THEN** verification fails rather than accepting mere file existence or production-derived expected pixels
