## ADDED Requirements

### Requirement: Bounded scoped compositing inspection
Desktop MUST inspect authoritative root visual masks, matte, matteOnly, blend mode and ordered seven-effect collections with scoped selection. Component-local occurrences MUST remain read-only; audio-only media eligibility MUST derive from authoritative asset metadata. Group/Instance MUST expose only existing aggregate effects and composition-bounds clip controls, and Repeater owner effects MUST remain unavailable. Existing Image/Video Media, Text, SolidColor, Rectangle, Shape, SVG and Grid MUST expose applicable mask/matte/matteOnly/blend/effect controls; Caption MUST expose blend only; Repeater, Transition and audio-only Media MUST expose no compositing mutation. The inspector MUST explain controlled contiguous aggregate ordering and existing uncontrolled track/z/stable ordering without claiming tree indentation is paint order.

#### Scenario: C1 Inspect eligible leaf owner and scoped audio selections
- **WHEN** selection moves among visual leaves, controlled Group/Instance, Repeater, audio-only media and component-local occurrences
- **THEN** only applicable controls appear, values and scope match the project, and unsupported controls cannot mutate

#### Scenario: C2 Inspect complete retained state
- **WHEN** a selected visual item contains unsupported or undisplayed authored values
- **THEN** the inspector labels retained read-only content and preserves every unedited value

### Requirement: Stable bounded compositing collection cursors
Desktop MUST select masks/effects by immutable authored ID, show collection counts and ordered position, and provide previous/next navigation. Path-command and gradient-stop cursors MUST bind owning mask and source revision. Presentation MUST materialize only selected command/stop details for maximum16-record,4096-command,64-stop collections, using bounded metadata summaries rather than whole compound JSON. Existing legacy presentation helpers MUST NOT incidentally serialize whole masks/effects or entire selected path/gradient collections; representation-only selective serialization MUST preserve every existing legacy Field value/path/kind/update rule and animation selected-key/loop/curve behavior. Existing field-description integration MUST avoid incidental whole mask/effect/path/gradient serialization and retain exact previous Field label/path/kind/value/updateKey semantics, including legacy Text enumeration/layout and Grid fallback. Automated prior-vector equivalence and actual maximum unrelated-collection nonmaterialization evidence MUST supplement output counts. Cursor state MUST be process-local. Add MUST select the new ID; reorder MUST follow the same ID; delete MUST select the surviving record at the old index or preceding/empty record.

#### Scenario: C3 Navigate maximum retained collections
- **WHEN** a selected item contains maximum valid collection/command/gradient-stop counts
- **THEN** bounded details expose selected values and counts and preserve all undisplayed content

#### Scenario: C4 Follow stable IDs across collection mutations
- **WHEN** a selected mask or effect is added, moved earlier/later or deleted
- **THEN** selection follows the specified ID and fallback and empty/end actions remain safe

### Requirement: Faithful typed compositing input
Desktop MUST represent every compositing RGBA component and other floating parameter independently as finite f64 with round-trip precision. Integer fields MUST parse exact u16/u32 representations including seed4294967295; parsing MUST never clamp, round or quantize authored values. Malformed numeric/choice representations, including invalid non-ASCII input, MUST fail safely without calling mutation. Core MUST retain semantic range validation. Existing quantized hex-color fields MUST NOT be used for compositing RGBA.

#### Scenario: C5 Preserve floating precision and integer endpoints
- **WHEN** users apply valid non-byte-aligned RGBA/f64 values and maximal representable particle seed
- **THEN** typed operations preserve exact values and unrelated parameters

#### Scenario: C6 Reject malformed representations without mutation
- **WHEN** input contains non-finite, fractional integer, overflow, malformed choice or Unicode-invalid numeric data
- **THEN** parsing feedback appears without panic, revision/history/resource change or core mutation

### Requirement: Complete selected mask field controls
Desktop MUST edit selected mask channel alpha/luma, operation add/subtract/intersect/exclude, inversion, featherPx, signed expansionPx, transform position x/y/unit pixels/normalized, anchor x/y, scale x/y, rotation/skew x/y degrees and opacity. Solid paint MUST expose independent RGBA. FillRule MUST expose nonzero/evenodd. MoveTo/LineTo endpoints, QuadraticTo control/endpoint and CubicTo both controls/endpoint MUST expose selected scalar coordinates without changing command variant; Close MUST be read-only. Gradient variant/coordinates and selected stop offset/RGBA MUST remain visibly read-only without implicit paint conversion.

#### Scenario: C7 Apply all selected mask scalar families
- **WHEN** each supported mask field or path-command coordinate is applied
- **THEN** one existing typed update changes only that field and preserves all remaining mask/source/item data

#### Scenario: C8 Retain gradient and command variants
- **WHEN** a gradient mask or Close command is selected or another mask property is edited
- **THEN** read-only details remain visible and exact gradient/commands/stops are retained

### Requirement: Explicit ordered mask authoring defaults
Desktop MUST add only the canonical default mask: first-unused mask-N ID, Path nonzero fill rule with MoveTo(0,0),LineTo(64,0),LineTo(64,64),LineTo(0,64),Close, solid white RGBA, alpha/add/not inverted, Transform2D default position0pixels/anchor0/scale1/rotation0/skews0/opacity1, feather0/expansion0. The UI MUST label the fixed64local-pixel rectangle without asserting full-source coverage. Add/delete/reorder MUST submit complete authoritative masks through existing UpdateItem, preserving IDs and all unedited records. IDs MUST remain immutable; existing records MUST NOT be retyped.

#### Scenario: C9 Add default and preserve ordered masks
- **WHEN** users add, reorder or delete unreferenced masks
- **THEN** first-unused ID/defaults/order match the canonical fixture and unaffected records remain exact

#### Scenario: C10 Reject limits and referenced mask deletion
- **WHEN** adding exceeds core limits or deletion would dangle a retained animation target
- **THEN** core rejection and exact unchanged complete inventory remain visible without silently deleting channels

### Requirement: Complete seven-effect parameter controls
Desktop MUST expose Gaussian radiusPx; Glow radiusPx/intensity/RGBA; Tint RGBA; Vignette amount; Adjustment exposureStops/contrast/saturation; Flash startMs/durationMs/intensity/RGBA; and Particle count/seed/radiusPx/speedPxPerSecond/lifetimeMs/RGBA. Editing MUST retain selected type/ID, original clock metadata, all other effects and animation channels. Owner eligibility and effect-animation restrictions MUST remain supplied by core.

#### Scenario: C11 Apply every effect parameter family
- **WHEN** each supported field of all seven selected effect variants is applied
- **THEN** the existing typed core operation changes only intended parameters with exact numeric/color representation

#### Scenario: C12 Retain unsupported owner and target behavior
- **WHEN** unsupported owners or aggregate effect targets would be requested
- **THEN** desktop offers no unsupported authoring and core failure semantics remain unchanged

### Requirement: Explicit ordered effect authoring defaults
Desktop MUST add first-unused effect-N IDs using canonical defaults: Gaussian radius2; Glow radius2/intensity0.5/white; Tint white; Vignette0.5; Adjustment exposure0/contrast1/saturation1; Flash start0/duration200/intensity0.5/white; Particle count16/seed1/radius2/speed20/lifetime1000/white. White MUST mean RGBA(1,1,1,1). Add/delete/reorder MUST use complete authoritative ordered effects with immutable IDs, no implicit retyping and no changes to unedited records/channels. Meaningful noncommuting ordering MUST remain visible.

#### Scenario: C13 Add all defaults and reorder noncommuting effects
- **WHEN** users add each type and move effects earlier/later
- **THEN** canonical defaults/first-unused IDs and requested stack order are committed with unchanged neighbors

#### Scenario: C14 Reject overflow and referenced effect deletion
- **WHEN** a collection exceeds16 records or deletion leaves an authored effect target
- **THEN** core rejects atomically with unchanged revision/history/inventory and no channel removal

### Requirement: Leaf blend and matte controls
Eligible root leaves MUST expose normal/multiply/screen/overlay/add/darken/lighten blend values, matte source item ID and alpha/luma channel, explicit clear and matteOnly. Creating a matte MUST require explicit source ID with initial alpha; omission MUST preserve the matte and clear MUST use existing explicit null semantics. Desktop MUST delegate provider scope, missing reference, cycle, eligibility and locks to core without fabricating a provider.

#### Scenario: C15 Apply blend matte channel clear and matteOnly
- **WHEN** users choose each blend, set an explicit matte provider, change channel, clear and toggle matteOnly
- **THEN** existing typed updates persist intended values and preserve unrelated properties

#### Scenario: C16 Reject invalid provider graphs
- **WHEN** the provider is missing, out of scope or introduces a cycle
- **THEN** the exact core failure remains visible and the complete prior state is unchanged

### Requirement: Explicit aggregate clip controls
Root Group/ComponentInstance MUST expose composition_bounds clip set and explicit clear through existing typed UpdateItem. Setting clip or nonempty effects MUST use verified #56 controlled isolation; removing clip MUST preserve effects and omission MUST preserve clip. Desktop MUST NOT offer aggregate masks, matte, matteOnly, non-normal blend, owner shutter or effect-target animation activation.

#### Scenario: C17 Set and clear isolated owner clip
- **WHEN** users set/clear a Group/Instance clip while retaining an effect collection
- **THEN** only clip changes, verified aggregate semantics remain shared and unrelated effects/children remain exact

#### Scenario: C18 Preserve owner compatibility
- **WHEN** uncontrolled and component-local owners are inspected or unrelated fields are edited
- **THEN** no implicit clip/effect activation or unsupported mutation occurs

### Requirement: Compositing draft identity and action guards
Desktop MUST bind drafts to scoped selection, authoritative revision, animation cursor, compositing selected mask/effect IDs, command/stop cursor and exact field path/updateKey. Navigation, collection actions, selection, reset and successful refresh/history/edit MUST discard obsolete drafts. Every mutation and collection button MUST honor busy/needs_refresh. Async completion MUST NOT restore input or selection belonging to another context. Conflict MUST require explicit refresh without automatic retry.

#### Scenario: C19 Discard obsolete drafts across contexts
- **WHEN** users change collection/command/field/selection/reset/history/refresh while input exists
- **THEN** stale input cannot apply to another field, ID, revision or occurrence

#### Scenario: C20 Guard pending and conflicting actions
- **WHEN** a mutation is in flight or an external edit causes a revision conflict
- **THEN** collection actions and Apply remain guarded, exact failure is shown and explicit refresh restores authoritative state

### Requirement: Core-owned compositing mutation history
Desktop MUST use existing typed core operations with displayed expected revision, preserve every unedited authored record, clock, target and metadata, reload on success and publish no optimistic domain state. Core MUST own semantic limits, locks, references, cycles, complexity and persistence. Failure MUST retain code/message/retryability and complete project/history/resource inventory. Reset MUST not mutate. Undo/redo/reopen MUST reconstruct exact committed compositing values without saved UI cursors.

#### Scenario: C21 Preserve full state on valid edits and history
- **WHEN** supported edits are applied, undone, redone and reopened
- **THEN** each authoritative state matches exact intended values and retained metadata

#### Scenario: C22 Preserve failure inventory and explicit refresh
- **WHEN** core rejects invalid bounds, locked/missing items, dangling targets or stale revisions
- **THEN** no partial edit/history/resource change occurs and the returned failure remains visible

### Requirement: Demonstrated compositing desktop workflow
The change MUST include a successful desktop build and observed actual GPUI workflow against a deterministic project. Evidence MUST record environment/commands/controls and authoritative transitions for mask add/edit/command navigation/order/delete, all seven effects and typed parameters/order, blend, matte/channel/clear/matteOnly, Group/Instance clip/effects, invalid numeric/semantic input, reset, selection, undo/redo and external conflict/refresh. Session tests MUST supplement this genuine GUI evidence. Placeholder preview MUST be labelled accurately and production media claims MUST derive from configured native tests. Unavailable required GUI execution MUST remain an acceptance blocker.

#### Scenario: C23 Exercise actual built compositing controls
- **WHEN** the built affected application is used against the documented deterministic project
- **THEN** required controls, transitions and failure feedback are observed and recorded with accurate limitations

#### Scenario: C24 Distinguish GUI state from production rendering
- **WHEN** desktop preview is displayed while native parity fixtures are executed
- **THEN** documentation identifies the placeholder and reports media evidence only from actual production renders
