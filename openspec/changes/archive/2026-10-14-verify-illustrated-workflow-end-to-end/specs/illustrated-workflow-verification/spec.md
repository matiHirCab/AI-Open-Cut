## ADDED Requirements

### Requirement: Original bounded illustrated target fixtures
Verification SHALL author the three original device, sticker/motif, and three-member team targets through existing native MCP operations using original synthetic managed media. Each target MUST preserve #181's named roles and layout envelopes, 1920×1080/30fps/3000ms timeline, samples at 0/1500/2966ms, and inclusive caps. Actual expanded leaves, instances, text scalars, artwork occurrences, distinct active assets, retained stored/history assets, image dimensions, path commands, and animation channels SHALL be recorded independently. ArtworkCount SHALL bound active expanded composition occurrences without asserting a bound on retained storage or changing existing storage guards. Historical recipes and public/persisted contracts SHALL remain unchanged.

#### Scenario: All original targets exist within their caps
- **WHEN** the three fixtures are authored and inspected
- **THEN** all original roles/layouts and exact timing settings are present, their independently measured inventories satisfy the unchanged per-target caps, and smaller inventories are explicitly reported without maximum-envelope claims

#### Scenario: Missing roles or excess inventory is rejected
- **WHEN** a fixture omits an original role, moves an element outside its agreed envelope, changes duration/samples, or exceeds a cap
- **THEN** the verification fails rather than silently accepting a relabeled earlier example

### Requirement: Actual editable MCP lifecycle
Both required source and assembled default-package MCP paths SHALL create/customize the targets, edit native source with observable output change, verify independent instance override isolation, preserve editable state after close/reopen, and exercise atomic late invalid batches, stale revisions, undo/redo, and actual cancellation using existing typed behavior. These checks SHALL preserve existing guards, polling bounds, test entries, and assertions.

#### Scenario: Native edits and overrides survive reopen
- **WHEN** source and instance values are edited through actual MCP and the project is reopened
- **THEN** saved native data and output reflect the source edit, instance overrides remain independent, and supported edits require no manual plates

#### Scenario: Invalid lifecycle operations preserve state
- **WHEN** a batch fails late, a stale revision is submitted, or a render job is cancelled
- **THEN** typed outcomes match existing contracts, failed mutations preserve revision/history/inventory, undo/redo restore expected states, and cancelled work publishes no completed artifact

### Requirement: Independent visual and media evidence
Verification SHALL separately establish semantic roles/layout and manually specified native pixel/gap probes, with omission/corruption negative controls, rather than treating route parity as an independent oracle. Both runtime paths SHALL render the three sample PNGs and [0,3000) range/export at 1080p, verify 90 frames and timing within one output frame, compare range/export SSIM at least 0.99, and compare decoded synthetic audio RMS at most 0.0001 with nonzero energy. Each decoded audio route SHALL additionally agree with the independently authored synthetic PCM and original clock at normalized correlation at least 0.99 and alignment within one 30fps output frame, with silence, omission, and a 40ms wrong-clock negative control.

#### Scenario: All target samples and full ranges conform
- **WHEN** actual native artifacts are decoded and inspected
- **THEN** independent layout/pixel expectations pass at all agreed samples, full-range and export agree within visual/audio thresholds, each decoded audio agrees with the original PCM/clock within the independent correlation/alignment thresholds, and frame counts/timing match the three-second timeline

#### Scenario: Corrupted output does not pass on parity alone
- **WHEN** an expected role/probe is omitted or an independently checked output/inventory value is corrupted, or audio is silent, omitted, or shifted by 40ms
- **THEN** the corresponding oracle rejects it even if two routes could share that omission

### Requirement: Isolated actual desktop interaction evidence
Release evidence SHALL include the current source desktop opening MCP-authored target data in a fresh owned cloud GUI session, actual input events for supported slot editing and lifecycle behavior, settled nonempty screenshots, persisted value/revision transitions, and source/build/environment attribution. Earlier evidence, widget tests, or screenshots alone SHALL NOT establish fresh desktop integration acceptance. Personal installed desktop and unrelated production files/processes SHALL remain untouched.

#### Scenario: Desktop edits the MCP-authored native target
- **WHEN** actual GUI input edits a supported target slot and exercises persistence/history in the isolated current build
- **THEN** the displayed state and persisted native transitions agree, and evidence identifies the exact source/build/session rather than inferring installation capability

#### Scenario: Incomplete GUI observation blocks acceptance
- **WHEN** only model tests, empty frames, old evidence, or unobserved installation information are available
- **THEN** fresh desktop acceptance remains unproven and the limitation is reported explicitly

### Requirement: Attributed release documentation and performance limits
Documentation SHALL provide reproducible usage, supported parameters, actual tested capability matrix, source/package/desktop attribution, dependency outcomes and explicit #188/#189 waiver, and retained #203 optimization evidence. Original #187/#203 performance inputs, numeric thresholds, counterexample, fixed captures, and guard budgets SHALL remain unchanged. New three-second timing observations SHALL NOT be represented as the original acceptance workload, portable performance, or maximum-envelope proof.

#### Scenario: Release evidence distinguishes observed capabilities
- **WHEN** the workflow evidence is published
- **THEN** each capability claim cites the actual tested runtime/build and limitations, usage reproduces existing operations, dependency waivers are explicit, and personal installed desktop capability remains unobserved

#### Scenario: New timings retain the original performance meaning
- **WHEN** new 90-frame workloads are timed
- **THEN** measured inventories and environment are reported separately, original fixed evidence and 5s/8s workloads are retained, and no broader performance claim is inferred

### Requirement: Required verification and exact-head completion
Completion SHALL require meaningful automated scenario coverage, all repository-required local gates without failed/skipped required checks, independent conformance review, synchronization/archive of only this change, final protected strict validation, and all required published-head CI checks with unchanged prerequisite/default-120-minute foundation behavior. Failures and unavailable evidence SHALL be retained and reported; no merge or deployment is authorized.

The inherited native cached-range golden SHALL require the already approved current-main ten-frame preparation progress[0.05,0.1,0.15,0.2,0.25,0.3,0.35,0.4,0.45,0.5] followed by publication1. Its exact cache hit/miss counts, encoded-byte reuse, changed-media invalidation, frozen visual/audio/layout/state checks and missing-input precedence MUST remain enforced; preparation events alone MUST NOT be interpreted as an encoding miss. No renderer, cache limit, reference, tolerance, deadline or policy changes are permitted.

#### Scenario: Verified change reaches draft review
- **WHEN** implementation checks and independent conformance pass
- **THEN** only this change is archived, final protected validation passes, the verified commit is pushed to a draft PR, and exact-head CI evidence identifies all checks and unchanged budgets

#### Scenario: A required check fails or is skipped
- **WHEN** any required local, desktop, or published-head evidence is failed, skipped, or unavailable
- **THEN** completion remains blocked, the exact limitation is reported, and no guard, tolerance, or budget is weakened to obtain acceptance
