## ADDED Requirements

### Requirement: Reviewed golden evidence for shared render routing
The canonical shared-render guarantee MUST be backed by a required native golden suite that renders a fixed non-empty EvaluatedScene through frame preview, audiovisual range preview, and final export, compares each entry point with reviewed visual, audio, timing, semantic-plan, and normalized-filter-graph references, and proves failures occur before render or publication side effects.

#### Scenario: Exercise every production output intent
- **WHEN** required native CI evaluates and renders the canonical fixture at its declared frame and interval
- **THEN** frame preview, range preview, and export all consume the same semantic scene, satisfy the documented SSIM, decoded-audio RMS, and one-frame timing tolerances, and match the reviewed semantic and filter-graph evidence

#### Scenario: Fail before fixture output on invalid work
- **WHEN** the canonical fixture is varied to contain invalid timing, a missing media reference, or a stale expected revision
- **THEN** the existing typed failure occurs before renderer execution, temporary or golden-reference writes, artifact publication, or project/history mutation
