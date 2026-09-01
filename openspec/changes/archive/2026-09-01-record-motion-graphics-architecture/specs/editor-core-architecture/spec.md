## MODIFIED Requirements

### Requirement: EvaluatedScene-compatible render seam
The render-planning boundary MUST provide a single inward seam owned by editor-core through which motion-graphics work can supply immutable renderer-neutral `EvaluatedScene` semantics to frame preview, range preview, draft preview, and export. Every render entry point MUST consume the same canonical evaluation semantics, and the renderer facade MUST NOT inspect persisted project tracks/items, resolve hierarchy or timing, compile presets, or reconstruct logical input/resource/layer ordering outside that seam.

#### Scenario: Introduce EvaluatedScene later
- **WHEN** the renderer-neutral evaluated representation is implemented
- **THEN** all render entry points can substitute it at the planning boundary without changing renderer orchestration, persistence ownership, process execution, artifact storage, or public project semantics

#### Scenario: Render every entry point through one evaluation
- **WHEN** frame preview, range preview, draft preview, or final export renders a fixed immutable project revision
- **THEN** each entry point obtains hierarchy, timing, primitive expansion, layer ordering, graphics, effects, and audio instructions from the same editor-core evaluation semantics rather than reconstructing them in the renderer
