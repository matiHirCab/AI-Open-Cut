## ADDED Requirements

### Requirement: Explicit source-grounded local planning
The standalone version-1 local CLI SHALL accept an explicitly approved source-timed transcript for one existing managed audio/video asset, resolve its identity, integrity, revision and duration through existing core-backed reads, and return a read-only bounded plan. It MUST reject stale/missing/mismatched sources and MUST NOT infer original source times from movable timeline captions, read project documents directly, mutate project/history, invoke providers, upload media or install models.

#### Scenario: Plan an approved managed source
- **WHEN** a caller supplies an approved source transcript matching the current project revision and managed asset
- **THEN** the plan records the exact project/asset/revision/content identity and source intervals while authoritative project/history/media bytes remain unchanged

#### Scenario: Reject stale missing or unapproved source
- **WHEN** approval is absent, core rejects the asset, or source identity/revision/duration changes during planning
- **THEN** no successful plan is emitted, existing core errors remain recognizable and no inference or mutation is attempted

### Requirement: Bounded complete transcript provenance
Input MUST be at most 4MiB with at most 128 chunks, 4096 segments and 32768 optional words. Source duration MUST be positive and at most 24 hours. Each chunk MUST declare its source offset/duration and provider/model/maximum-duration provenance, fit that declared provider limit, and together chunks MUST explicitly cover the complete source without silent truncation. Segment/word timestamps SHALL be original chunk-relative times with bounded text and unique identifiers; invalid or ambiguous timing MUST fail explicitly. These supplied provenance claims SHALL not be represented as independently verified inference.

#### Scenario: Handle long source chunks explicitly
- **WHEN** an approved long source is supplied as complete ordered chunks within declared provider bounds
- **THEN** candidate evidence retains chunk/provider provenance and converts checked offsets into exact source ranges without invoking the provider

#### Scenario: Reject truncation and resource overflow
- **WHEN** chunks omit source coverage, exceed provider duration or resource limits, or contain invalid/duplicate/nonordered timing
- **THEN** planning fails before successful output and reports an input or limit error instead of silently dropping content

### Requirement: Editable candidates with honest evidence and context
Planning SHALL deterministically group complete transcript segments around a caller target duration, emit at most 32 candidates and reject an exceeded requested candidate bound rather than truncate. It MUST retain original segment quotes/times/provenance separately from caller-supplied hook/CTA/rationale and include adjacent source context. Human edits SHALL select explicit source boundaries and editorial suggestions without rewriting transcript evidence. Plans MUST expose cut-through-segment, excluded-context, target-duration, chunk-boundary and candidate-overlap warnings as applicable. Every editorial suggestion and every candidate's semantic/context completeness MUST remain subject to human review; no suggestion SHALL be labeled fact-checked, source-verified or guaranteed engaging.

#### Scenario: Preserve negation context and separate claims
- **WHEN** source segments contain negation or uncertain claims and a caller supplies a hook/CTA/rationale
- **THEN** original evidence and neighboring context remain exact and distinct from editorial text, with explicit semantic-context and unsupported-claim review warnings

#### Scenario: Correct boundaries and expose overlap
- **WHEN** a caller edits candidate boundaries within the source, including a mid-segment cut or overlap with another candidate
- **THEN** the plan records the exact requested range, retains full original evidence with inclusion flags and exposes boundary/context/overlap warnings without assembling or mutating media

### Requirement: Governed additive interface and isolated ownership
The new CLI category MUST have one versioned checked-in canonical fixture, ownership-index entry and actual consumer coverage in the unchanged required contracts:check command. New ownership additions MUST be independently pinned and exactly invertible before every existing historical proof; malformed additions, missing coverage and unrelated drift MUST remain detectable. Existing MCP/headless/provider/persisted schemas, stable errors, narration behavior and #213 SHALL retain their meanings. Shared integration-file conflicts with future #208 SHALL be documented without retrieving or reconstructing its WIP.

#### Scenario: Verify canonical and predecessor parity
- **WHEN** fixtures, CLI validators and protected contract checks run
- **THEN** valid/invalid cases and current ownership agree, only the exact approved addition projects away, all unchanged predecessor pins/negative controls remain enforced and no new MCP/headless/provider behavior is registered

#### Scenario: Demonstrate real local workflow
- **WHEN** actual CLI execution plans and edits original synthetic managed media, then exercises stale/missing/invalid inputs
- **THEN** native core resolution and bounded CLI behavior are observed with exact output/source oracles and unchanged authoritative hashes; required implementation, OpenSpec and exact-published-head aggregate checks pass before completion
