## Why

Issue #55 cannot expose the planned exposure/contrast/saturation controls: the reserved color_adjustment effect has no typed active payload or renderer. Gaussian blur, glow, tint and vignette already exist; retain their verified semantics and add the missing independently checked positive Gaussian public-native witness.

## What Changes

- Add a closed color_adjustment record with stable id, exposureStops[-8,8], contrast[0,2], saturation[0,2], all finite and required. Apply exposure→contrast about linear0.18→Rec.709 saturation→final clamp in straight linear RGB, preserving alpha and zero-alpha invariants.
- Activate project schema36 with deterministic complete-generation migration; reject new records in sources below36 and future schemas. Preserve current/history/drafts/resources/provenance, existing staging/journal semantics and no-rewrite reopen.
- Expose additive parameterized_effect_models_v1 editor and parameterized_effects_v1 rendering discovery through existing status, typed existing edits, standalone/alias batches and exact canonical cross-language parity. No tool/operation count or protocol-major change.
- Add one canonical parameterized-effects-v1 catalog and exact36→35 current/predecessor projections; retain every prior pin, ordered-effect fixture and nested field. Existing ten catalogs update only their top-level current marker.
- Add independent color/alpha/order/native frame/range/draft/export/audio/lifecycle and nonzero Gaussian witnesses, exact bounded work and fail-closed retained-source evidence. Extend mandatory core/default-headless/MCP CI commands additively after existing ordered-effect commands, preserving every prior guard and deadline.

## Capabilities

### New Capabilities

None: visual-effects remains the single normative effects owner.

### Modified Capabilities

- visual-effects: fifth typed leaf effect and linear-light color-control semantics, bounds and independent evidence.
- project-persistence: schema36 activation, source-matched drafts, complete staged adoption/recovery and resources.
- contract-governance: governed parameterized effects authority, exact current36 markers/MCP discovery and composed historical proofs within unchanged5000ms single test.
- rendering-export: independent positive color-control and Gaussian native evidence across shared intents.
- repository-validation: mandatory actual native color-control core and MCP execution and additive negative guards.

## Non-goals and compatibility

No directional blur, screen flash, particles, group/instance effects or clipping (later issue56), desktop compositing controls (issue57), new effect animation-channel names, brightness alias, raw expressions, new resource/path fields, backend algorithms for existing effects, widened budgets/tolerances/timeouts, new operations or tools. Existing simple behavior remains pixel-identical; the fifth closed typed variant and discovery are additive, with required schema36 adoption and unchanged protocol1. Historical exact digest pins remain immutable. Unavailable-base drafts follow the predecessor policy rather than invented source assumptions.

## Impact

Core models/validation/migrations/store preflight/evaluated-scene budgets/local raster; headless status and typed bridge consumers; canonical catalog/ownership/CODEOWNERS and exact fixtures/projections; independent Rust/TS/native/protocol/MCP tests, docs and protected CI command policy. No new dependencies or cross-layer domain validation.

## Verified base and dependency

Issue54 is closed, PR147 externally merged. Branch55 was created from exact verified016586571e8f49d59aa19062fd59076a017ff128 then fast-forwarded to origin/main857b8539eac743593b60c97724ae58a97fed0fc8, retaining three subsequent merge commits with byte-identical source tree. Prior all11CIrun37508868548 is exact-head evidence; merged history itself introduces no source change. No unrelated edits were present or discarded.
