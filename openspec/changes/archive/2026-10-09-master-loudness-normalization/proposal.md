## Why

Issue69/epic8 require configured integrated loudness and true-peak ceilings on the authored complete root mix, shared by preview, draft and export. Existing68 analysis reports original samples but does not author normalization settings or normalize delivered audio.

## What Changes

- Add optional nonnull closed project masterNormalization and one typed audio_master_set_normalization standalone/batch edit/MCP tool. Adopt schema44 atomically for current/history sources1..43; headless protocol1 and all old operations remain valid.
- Evaluate active settings once at root; prepare complete-root bounded two-pass normalization through existing selected audio lowering and private process/workspace ports; lower prepared finite coefficients once after final master balance before range/sample crop/codec.
- Use independently scanned actual48kHz stereo delivered PCM for integrated target/true-peak acceptance, distinguish null integrated short/very quiet content and exact silence, fail safely on infeasible measured targets. Preserve old fixed loudnorm input analysis semantics and explicitly distinguish authored processing from analysis-induced normalization.
- Capture final verified68 contracts before producers; manually author all additive fixtures and every governed consumer; preserve all prior raw/digest/count/numeric oracles. Add native, migration/fault, transport/source/release-package and cancellation evidence beside unchanged checks.

## Capabilities

### New Capabilities
- master-loudness-normalization: bounded authored root controls, atomic edits/schema44, full-root measured preparation and shared rendering.

### Modified Capabilities
- audio-mix-analysis: original PCM means the final authored mix including enabled master processing; its own fixed loudnorm output remains discarded and old absent/disabled cases unchanged.
- agent-bridge: additive typed normalization edit/tool and truthful capability/readiness/version reporting.
- rendering-export: active master preparation shared by every intent before final output crop/codec.
- project-persistence: recognize transaction temporary names before inspecting entry types, preserving overlapping normalization cleanup and all actual transaction errors.

## Impact

Owning core model/validation/timeline/migrations/evaluated_scene/render_plan/render_process/render_artifact/renderer/store orchestration, headless reporting, bridge DTOs/MCP, canonical contracts/pins/consumers/docs/policy and independent tests. Existing ownership/import graph remains unchanged. No raw expressions, public paths, cached/persisted measurements, provider/editor UI changes, network inputs, merges/deployments/issue closures, or unrelated changes. Draft PR targets main, cumulative merge order68→69→70. This proposal requires explicit scoped reviewer approval before producer edits.

Issue69 overlap evidence requires a narrowly scoped owning persistence correction: filter existing UUID-suffixed transaction temporary names before type inspection, retaining actual transaction failure and nonfollowing file ownership semantics. Add project-persistence modified recovery capability; no new dependency/public contract or error code.
