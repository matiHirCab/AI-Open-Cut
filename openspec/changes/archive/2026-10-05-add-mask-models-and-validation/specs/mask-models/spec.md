## ADDED Requirements

### Requirement: Closed painted path mask records
Core MUST accept a mask as exactly required id, source, channel, operation, inverted, transform, featherPx and expansionPx fields. Source MUST be exactly type=path, canonical VectorPath path and canonical Paint paint. Channel MUST be alpha or luma; operation MUST be add, subtract, intersect or exclude; inverted MUST be boolean. Every nested record MUST retain its existing strict object/enum representation, required fields and unknown-field rejection. Actual raw JSON core/headless Serde decoding MUST reject duplicate fields where original keys are observable; parsed MCP/Zod objects SHALL NOT claim detection of duplicate original wire keys unavailable after parsing. ID MUST contain 1…128 UTF-8 bytes and be unique within its owning item. Feather MUST be finite in[0,128] and expansion finite in[−128,128] local pixels. Path, paint and Transform2D MUST pass their existing canonical validators. No layer, path string, SVG, filesystem/network resource, backend expression or unrecognized source kind SHALL be accepted.

#### Scenario: Accept meaningful alpha and luma descriptions
- **WHEN** valid masks use alpha/luma, each operation, inversion, a nonidentity transform, bounded feather/negative expansion and differently colored solid/gradient paint
- **THEN** strict decoding and core validation preserve every field and paint value exactly without treating the two channels as aliases

#### Scenario: Reject malformed and unsafe descriptions
- **WHEN** any required field is missing, null, unknown, positional or unsupported, or raw JSON at the core/headless boundary contains duplicate fields, or values/topology/IDs exceed a canonical bound
- **THEN** decoding rejects malformed wire structure or core returns nonretryable INVALID_ARGUMENT without authoritative or resource writes

### Requirement: Bounded stored mask ownership
Nonempty masks MUST be supported only on image/video media, text, solid-color, rectangle, shape, SVG and grid leaves in root/component tracks. Other item kinds including audio MUST reject nonempty masks with INVALID_ARGUMENT. Every item MUST contain at most 16 masks; each composition at most 4096 masks/65536 mask path commands; each project at most 16384 masks/262144 mask path commands. Counts MUST include hidden tracks/items and every unused component definition once, before identity normalization, with checked arithmetic. Existing VectorPath command 4096/coordinate ±1000000 and Paint stop 64 limits MUST apply. Current state, retained snapshots and each replayable draft candidate MUST be validated independently; repeated instances SHALL NOT multiply stored definition counts.

#### Scenario: Enforce inclusive stored bounds
- **WHEN** a valid item/composition/project has exactly each named limit or one additional otherwise valid record/command
- **THEN** boundaries succeed and overflow fails before publication without dropping hidden/unused content

#### Scenario: Preserve scoped identity and target eligibility
- **WHEN** two independent eligible items use the same mask ID or an unsupported item receives a nonempty stack
- **THEN** item-local reuse succeeds and the unsupported target fails with INVALID_ARGUMENT without changing state/history

### Requirement: Transactional ordered mask edits
Existing update_item MUST accept optional masks. Omission MUST preserve the stack, a supplied array MUST replace it in declared order, [] MUST clear it, and null MUST reject. Existing atomic timeline batches MUST resolve a created owning item's alias before the same update; no mask-specific creation alias or new operation SHALL be required. Existing component create/update payloads MUST accept masks on local eligible leaves. Successful edits MUST preserve IDs/order/values in one optimistic revision and existing undo/redo/reopen/draft behavior. Missing target MUST retain ITEM_NOT_FOUND and stale revision MUST retain retryable REVISION_CONFLICT; all failure paths MUST retain atomic state/history/draft/resource rollback.

#### Scenario: Author and reorder through standalone and alias batch edits
- **WHEN** a client creates a visual and sets two masks through standalone update_item or creates @leaf then sets/reorders its stack in one batch
- **THEN** returned/persisted state exposes exact authored order and undo/redo/reopen reproduce the proper generation

#### Scenario: Preserve omit clear and atomic failure semantics
- **WHEN** an omitted masks edit, [] clear, valid draft edit, stale edit, missing target or malformed later batch edit is submitted
- **THEN** omission/clear/draft semantics remain deterministic and failures preserve authoritative bytes, revision, aliases and resources with existing stable errors

### Requirement: Explicit inactive rendering interpretation
Mask metadata MUST describe owner-local painted coverage. Pixel positions MUST use the owner's post-crop/clip pre-effect local raster basis; normalized positions MUST use its width/height; anchor MUST use untransformed path geometry bounds. Alpha MUST describe painted alpha and luma MUST describe Rec.709 weighted premultiplied linear RGB; transparent paint contributes zero. Transform opacity MUST describe one coverage multiplication. Inversion MUST describe B→1−B within the owner-local source raster domain, without reviving source crop/clip removal or extending source support. Future add/subtract/intersect/exclude coverage MUST have definitions A+B−A*B, A*(1−B), A*B and A+B−2*A*B, with initial coverage 0 for first add/exclude and 1 for first subtract/intersect, and empty-stack identity 1. Positive expansion MUST denote growth and negative contraction; feather MUST denote local Gaussian sigma. These definitions MUST be retained as metadata semantics only in this milestone; masks SHALL NOT affect any renderer output or activate animation targets. Pixel sampling, expansion rasterization and cumulative mask raster budget MUST be approved/tested with #51 before rendering support is advertised.

#### Scenario: Distinguish authored model support from rendering readiness
- **WHEN** an eligible visual has a nonempty alpha/luma stack with nonidentity parameters
- **THEN** its metadata round-trips while frame/range/draft/export continue using identity masks, and clients can detect model support without a claim of mask rendering
