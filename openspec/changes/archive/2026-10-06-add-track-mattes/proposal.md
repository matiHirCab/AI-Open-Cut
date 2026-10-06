## Why

Issue #52 requests alpha/luma track mattes, matte-only visibility and composition-scoped dependency-cycle rejection through public editing and shared rendering. The actual stable #51 implementation is schema33 with active static/animated masks and MCP digest `803bf5954ebd4cb47be98dd87b4994e6d261eae20693199c0f569f535452f170`. This approved change is reconciled to that source/contracts and approved deltas; #51 is verified/archived at `5f456618b9335a5f87519c5c4375a49e7f52c2d3`; every captured catalog and MCP pin has been independently matched to that exact commit. #52 was promoted as `add-track-mattes` from that verified implementation branch; explicit parent and independent reviewer approvals, later mechanism amendments and the equal-tree externally merged parent ancestry adjustment are recorded in approval.md.

## What Changes

- Add typed optional `matte:{sourceId,channel}` and boolean `matteOnly` to eligible visual leaves, authored through existing standalone/batch `update_item` and component create/update payloads. Eligible stored DTOs use optional non-null references; ineligible stored DTOs use absent-only matte and false-only matteOnly; shared updates retain nullable clear and core final eligibility.
- Validate directed target-to-provider dependencies independently of paint order, within root or one component definition, including hidden/unused content; reject missing references, unsupported targets, cycles and explicit limits atomically.
- Execute alpha or premultiplied-linear Rec.709 luma coverage at the existing post-transform, pre-recipient-opacity matte stage. Providers are isolated premultiplied planes, independent of destination/background and ordinary draw visibility suppression by matteOnly.
- Specify component/repeater occurrence mapping, timing, nested motion blur and immutable shared EvaluatedScene dependencies; preflight all work/live memory before rasterization or publication.
- Migrate schema33 to34 current/history atomically with defaults; guard premature fields in older source envelopes and retained-base drafts, preserve media/fonts/provenance and existing journal behavior.
- Update canonical contracts, headless/MCP schemas, capability readiness and a tightly reviewed catalog digest transition, docs and independent analytic/native conformance evidence.
- Strengthen the existing exact render-parity command body with mandatory native matte core and MCP witnesses, separated by a default headless build; preserve all existing protected commands, environment, sequence, timeouts and budgets, with additive tampering regressions.

## Capabilities

### New Capabilities

- `track-mattes`: closed typed references, scoped DAG, visibility, isolated provider interpretation, bounded shared rendering.

### Modified Capabilities

- `project-persistence`: atomic schema34 defaults/adoption and raw source guards.
- `contract-governance`: additive fields, canonical matte catalogs and model/render readiness.
- `rendering-export`: common evaluated dependency semantics across frame/range/draft/export.
- `linear-light-compositing`: activate matte stage without stale identity-only promises.
- `motion-graphics-architecture`: concrete scoped dependency and coordinate semantics.
- `repository-validation`: mandatory native matte CI execution and exact fail-closed command guards.

## Impact

Core model/timeline/component validation/migration/store and evaluated_scene/render_plan/render_artifact owners; typed headless/MCP consumers and canonical ownership fixtures. No new renderer expressions, paths, new mask/effect/blend kinds, matte animation properties or group/component-instance isolation surface. Existing mask animations and provider/recipient visual animation remain effective. Projects with every matte=None and every matteOnly=false bypass new rendering work and retain exact output. A matteOnly leaf without any references still suppresses direct drawing. Implementation is authorized by the exact specification and amendment approvals recorded in approval.md; all remaining verification and archival gates remain mandatory.

## Proposed canonical evidence

The independently reviewed external `proposed-track-mattes-v1.json` supplied the pre-implementation representation, limits and independent numeric/DAG/raw rejection cases. The approved checked-in authority is now `contracts/track-mattes-v1.json`, with its governed consumers and CODEOWNER registrations. Historical approval manifests remain unchanged.

The active text measurement mechanism is clarified in glyph-memory-certification.md: source-derived shaping scratch, actual cache/measured/finalized glyph capacities, and a fail-closed admitted font-family lookup all join the existing shared live-byte limit. Public fields, font formats, numerical output, glyph limits and the complete no-matte route remain unchanged. Its exact approval and subsequent platform/memory-scope clarifications are recorded in approval.md.

## Mandatory native CI verification

The independently approved mandatory-native-ci-plan.md strengthens the existing render-parity leaf without changing its six-key environment, earlier commands, steps, dependencies, timeouts, report/benchmark/cache-restoration behavior or aggregate duration budget. The exact workflow and validator require core native matte execution, a default headless build and the dedicated MCP matte test in that order. Actual local core/native and policy evidence does not assert pending exact-head remote CI success.
