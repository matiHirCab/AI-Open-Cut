## Why

Issue #182 needs one usable editable hero/device card after #181 established a bounded native benchmark. Existing components, text/managed-asset slots and animation channels already support this slice; a discoverable concrete recipe and end-to-end evidence are missing.

## What Changes

- Ship versioned `native-hero-card-v1` data through the existing read-only `create_motion_graphics` prompt, discoverable through existing prompt listing/retrieval.
- Describe and exercise a bounded native component with stable named children, fixed paints, existing title/image slots, two independent instances and persisted compatible entrance animation.
- Verify actual MCP customization, alias/batch failures, retained resources/history/reopen, and independent native frame/short animated-range evidence through existing source/packaged platform gates.

## Capabilities

### New Capabilities

- `native-hero-card`: discoverable original native card recipe, independent customization, atomic lifecycle and native verification.

### Modified Capabilities

None. Existing prompt identity/arguments and tool/resource/capability schemas remain unchanged; this adds deterministic guidance/data content to its existing behavior.

## Impact

Bridge read-only prompt content/static recipe, documentation and shared source/packaged/native MCP tests. Core remains sole owner of component/slot/animation validation, persistence and rendering. No public structural contract, capability identifier, schema, migration, dependency, renderer or provider changes. Existing canonical catalogs, digest pins and protected workflows/limits remain unchanged. Main `50ec842c` includes dependency #181 (externally merged PR193) and separate doctor fix PR194; neither is reimplemented.

## Non-goals

No palette bindings (#183), motif/team pack (#184/#185), budget diagnostics (#186), renderer optimization/precomposition (#187–#189), broad desktop controls (#190), new executing tool, asset service or full acceptance pack (#191). #179 preparation progress and doctor fixes remain separate. Original/synthetic artwork only; no desktop configuration/build/render access.
