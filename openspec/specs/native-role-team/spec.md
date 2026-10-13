# native-role-team

## Purpose

Provide bounded editable native role badges and grouped team examples through the existing read-only motion prompt and core operations, preserving independent instance values, transactional history and governed native evidence.

## Requirements

### Requirement: Discoverable read-only role and team recipe
The existing create_motion_graphics prompt SHALL expose deterministic native-role-team-v1 example JSON and guidance in its existing single message, retaining all preceding guidance, motif data and final hero JSON. Retrieval MUST leave projects/resources/providers unchanged. Existing public catalogs, structural schemas, identifiers, error meanings, protocol/project versions and dependencies MUST remain unchanged.

#### Scenario: T1 retrieve from source and package
- **WHEN** actual source or freshly assembled default-package MCP clients list and retrieve the existing prompt
- **THEN** the role/team recipe is identical to the canonical fixture, the original hero/motif readers remain valid, and retrieval has no project/provider side effects

### Requirement: Bounded independently editable badge composition
The recipe SHALL create one 800x600,1000ms shared badge definition with the seven native hero roles (title renamed role_label) and two original sun-motif roles in stable paint order. It MUST expose a required role-label text slot accepting 1 through24 Unicode scalars, a required project-managed image slot, seven hex6 solid-color palette slots preserving existing paint alpha, and one boolean sun-ray-hidden slot. Defaults and explicit override-map reset MUST use existing slot semantics. Its fixed original artwork MUST be synthetic64x64 pixels with explicit4x intrinsic scaling; no automatic fitting or flattened badge SHALL be introduced.

#### Scenario: T2 customize independent members and reset
- **WHEN** two shared-definition occurrences receive different valid role labels, managed artwork, palettes and visibility, and one subsequently clears its overrides
- **THEN** each occurrence resolves its own values/defaults without changing the other occurrence or the shared native geometry, and reset restores the defined defaults

#### Scenario: T3 accept label boundary and reject invalid inputs
- **WHEN** 1-scalar and24-scalar labels versus empty/25-scalar labels, wrong typed values or missing image references are submitted
- **THEN** valid bounds succeed, existing canonical typed failures reject invalid input atomically, and project/history/resource bytes remain unchanged after failure

### Requirement: Fixed team counts grouped placement and authorable reveal
Guidance SHALL support exactly1 through4 demonstrated members using explicit row and grid operation lists on a1920x1080,30fps canvas, without runtime count/layout inference or changing core limits. Members MUST use stable member_1 through member_4 aliases and paint order, one ordinary parent group and fixed local placements at0.5scale. Row positions MUST be(100+450i,390); grid positions MUST be(480+560(i mod2),160+400 floor(i/2)), for i0..3. Both layouts MUST retain nine visual leaves per badge, at most36 expanded visual leaves, one nonvisual group and at most four existing compatible root transform.opacity channels. Each channel MUST fade from0 to1 over100ms starting at80i milliseconds, preserving editable stored timing and canonical interpolation.

#### Scenario: T4 build every supported count and layout
- **WHEN** explicit row/grid examples with1,2,3 and4 members are submitted through existing batch operations
- **THEN** each commits one complete grouped placement with stable order/aliases, exact declared positions and nine leaves per member without a new scene format or solver

#### Scenario: T5 move the group and sample stored motion
- **WHEN** the parent is moved and member channels are evaluated at0,40,80,120,200,280 and500ms
- **THEN** relative member placement remains grouped and opacity matches independent clamped linear ramps (t-80i)/100, with exact endpoints and unchanged shared definitions

### Requirement: Canonical transactional lifecycle and managed resources
Creation, customization, group placement and channels MUST delegate to existing core validation, locks, revisions, aliases, resource ownership and history. The documented create batch MUST be one revision/undo step; any separate prior asset import MUST remain explicit. Late invalid edits, unresolved aliases, missing managed assets, locked tracks and stale revisions MUST reject with existing codes/retryability and preserve complete pre-request project/history/resources and artifact inventory. Undo, redo and reopen MUST preserve native roles, exact member identities/order/values, parent relationships and channels.

#### Scenario: T6 reject complete batch on failure
- **WHEN** a valid construction prefix precedes an invalid tail, unresolved alias or absent artwork, or the target track is locked
- **THEN** the canonical error is returned without publishing a partial badge/group/member or changing saved bytes/resources

#### Scenario: T7 reject stale customization
- **WHEN** customization uses a stale expectedRevision
- **THEN** retryable REVISION_CONFLICT preserves complete authoritative state and guidance requires refresh and replanning

#### Scenario: T8 preserve history and reopen
- **WHEN** creation and customization are undone, redone and reopened through actual core and MCP operations
- **THEN** exact definitions, independent values, grouped member placement and authorable channels are restored under existing revision semantics

### Requirement: Independent actual native output and preserved governance
Required source and default-package native tests SHALL render retrieved minimum and maximum examples, a500ms frame, a changed-instance frame and a silent range[0,400)ms containing12frames at30fps. Independent literal interior/exterior, artwork, palette, text and temporal expectations MUST establish native output and frame/range agreement within existing PNG3/encoded8 channel tolerances. Blank, wrong-size, wrong-paint and wrong-temporal evidence MUST fail. Read-only rendering MUST preserve authoritative state. Complete canonical/ownership/policy checks MUST retain all predecessor categories/pins/consumers/negative controls and strict equality guards; new fixture consumers MUST have omission and failure-masking rejection evidence. Exact published-head CI MUST pass before completion, with source/package/environment attribution and no installed-desktop or portable-performance claim.

#### Scenario: T9 render minimum maximum and customization
- **WHEN** actual source/default-package clients render one and four members, customize one maximum-team member and decode frame/range artifacts
- **THEN** expected native dimensions/counts, geometry/managed-image/role/palette/visibility samples and staggered states pass independently, other members remain unchanged and project/history/resources remain unchanged by rendering

#### Scenario: T10 reject false native evidence
- **WHEN** independent verification receives blank/truncated pixels, wrong paint or incorrect staggered states
- **THEN** verification rejects them without deriving expectations from production output or widening established tolerances

PNG verification MUST retain exact authored interior samples for all roles at the existing3-channel tolerance. Lossy encoded range verification MUST retain authored broad-interior color and temporal samples at the existing8-channel tolerance, plus independent presence controls for the narrow accent and rays whenever a member is fully revealed. The fixed accent and ray regions MUST exhibit authored yellow-versus-neutral contrast; omission of either role MUST fail verification. Thin-feature encoded RGB fidelity SHALL NOT be claimed. Sample positions and presence thresholds MUST come from authored geometry/colors rather than adapting to decoded output.

#### Scenario: T11 preserve cumulative governance and exact-head checks
- **WHEN** the cumulative predecessor union and new consumers are verified and malformed canonical commands omit/mask a new consumer
- **THEN** every old owner/pin/negative remains effective, malformed commands fail unchanged strict guards, and completion requires all protected jobs at the published remote head
