## Why

The motion-graphics roadmap depends on shared decisions about project structure, scene evaluation, rendering, ordering, presets, and schema evolution. Recording those decisions before public-model work begins prevents individual milestones from introducing incompatible persisted or renderer-specific semantics.

## What Changes

- Add ADR 0004 as the normative architecture decision for the motion-graphics initiative.
- Keep root project tracks as the additive composition root and reserve reusable component definitions beside them instead of replacing `Project.tracks`.
- Define one renderer-neutral `EvaluatedScene` seam shared by frame preview, audiovisual range preview, draft preview, and final export.
- Select a hybrid renderer: deterministic Rust rasterization for complex graphics and shaped text, with FFmpeg retained for media decode, audio, composition, and encoding behind replaceable interfaces.
- Make coordinate, timing, transform, mask/effect, alpha/color, and stable layer-order rules explicit.
- Require presets to compile into persisted primitive operations with versioned provenance rather than remain renderer- or MCP-only behavior.
- Require one additive project-schema bump per independently shippable persisted milestone, atomic deterministic migration of current state and retained history, and rejection of future versions.
- Add living OpenSpec requirements covering how later motion-graphics changes must conform to the ADR.

This change is documentation-only. It does not add project fields, operations, capabilities, dependencies, migrations, or rendering behavior.

### Non-goals

- Implement components, graphics primitives, richer text, animation channels, effects, markers, audio events, or a graphics backend.
- Change the current serialized project shape, headless protocol, MCP surface, capability report, error catalog, preview/export output, or contract fixtures.
- Select exact future public type or operation names beyond the architecture boundaries needed to keep later milestones compatible.

## Capabilities

### New Capabilities

- `motion-graphics-architecture`: Normative constraints for the additive root-track model, evaluated-scene seam, hybrid rendering boundary, ordering semantics, preset compilation, and schema-version policy.

### Modified Capabilities

- `editor-core-architecture`: Tighten the existing future `EvaluatedScene` seam by assigning canonical evaluation ownership to editor-core and requiring every render entry point to consume the same immutable evaluated semantics.

## Impact

- Documentation: add `docs/adr/0004-motion-graphics-architecture.md` and link it from the roadmap and relevant architecture documentation.
- Living requirements: add a motion-graphics architecture spec and strengthen the editor-core render-seam requirement.
- Compatibility: no runtime or persisted compatibility surface changes in this decision-only issue; later milestones must update the appropriate canonical fixtures and consumers when they add public or persisted fields.
- Verification: OpenSpec strict validation plus deterministic documentation tests that assert the ADR contains every locked decision and required observable semantic.
