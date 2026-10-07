## Why

Issue #57 requires usable desktop mask, matte, ordered-effect and blend controls plus parity through existing core/headless/MCP APIs. Verified #56 already supplies those public semantics; desktop currently exposes none of these collections.

## What Changes

- Add bounded selected mask/path/gradient inspection, faithful typed scalar edits and ordered mask authoring with explicit valid defaults; preserve legacy fields while removing incidental whole-item serialization from presentation.
- Add ordered seven-effect authoring, every parameter and exact integer/RGBA fields; expose seven leaf blend modes, matte set/channel/clear/matteOnly, and Group/Instance bounds clip/effects.
- Refactor existing field-description serialization to consume only actual legacy fields/selected animation data, preserving exact Field vectors while avoiding incidental whole mask/effect/path/gradient expansion.
- Bind drafts to authoritative scoped identity/revision/collection/command/field; preserve all unrelated values, use existing core mutations and retain core failures, locks, targeted-channel rejection and history.
- Add a fixture-governed desktop control/default catalog, fresh shared core/headless/MCP lifecycle/failure parity, automated bounded presentation tests and genuine built GPUI acceptance evidence.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- desktop-hierarchy: compositing inspection, collection actions, typed edits, stale input, history and genuine GUI workflow.
- contract-governance: canonical desktop compositing control/default fixture and governed consumer parity.

## Impact

Desktop presentation/session/inspector tests, canonical contract ownership/CODEOWNERS and parity consumers/tests, deterministic desktop fixture and workflow documentation, OpenSpec. Contract-parity CI installs the existing signed desktop link prerequisites and contracts:check runs governed Rust desktop tests; additive package-source/complete-command enforcement and Moon tracked-input controls preserve all prior gates. No new renderer, migration, schema, tool, operation, capability or public payload. Preserve schema37, protocol1, 78 tools and every historical/current MCP pin; seven effects and aggregate semantics are supplied by verified #56. No component-local mutation, path-command retyping/insertion/deletion, gradient mutation, effect retyping, domain validation duplication or preview backend.

## Verified dependency gate

VERIFIED: #56 draft PR150 head abc06b75d491d10ddd5aa3cf7bda0638aa437fe7 passed all11 required CI jobs in run37561714579. Remote main remains944a048c13e58cfa79acc2ac39c865ada396403b; preceding external merges are preserved. Branch #57 from that implementation and target its branch with a stacked draft PR; #58 follows only after #57 exact-head CI. Preserve subsequent external commits/merges. Do not merge or deploy.
