# Independent concrete specification review

Status: **delegated specification acceptance** for the revised separate-operation design.

Reviewed proposal, design, tasks and both delta specifications against root AGENTS.md, docs/spec-driven-development.md, ADR 0002 and the current renderer/headless declarations. This review is delegated specification review under the user's standing authorization; it is not human CODEOWNER acceptance, implementation verification, merge approval or deployment approval.

## Compatibility finding resolved

The initial design would have applied 7680x4320 limits to the existing numeric range API. Existing headless RenderPreviewRange uses u32 dimensions (apps/headless/src/main.rs); Renderer::render_preview_range rejects zero dimensions but has no upper dimension limit (crates/editor-core/src/renderer.rs); evaluate_flat_project likewise only requires positive dimensions (crates/editor-core/src/evaluated_scene.rs). Newly applying project-settings bounds there would narrow an existing public contract.

Re-read proposal, design, both delta specs and tasks after correction. They now add uniquely named headless render_review_range and MCP preview_review_range, scope canonical bounds to the new review resolver, and explicitly retain legacy render_preview_range required fields and historical dimension acceptance. This resolves the blocker and supports ADR 0002's additive classification. Implementation regression evidence must demonstrate above-bound legacy dimensions retain existing routing while new review dimensions fail before render side effects; a fake process/artifact adapter can avoid actual large renders.

## Reviewed decisions without further blockers

- Separate MCP preview_review_range preserves legacy required custom dimensions/fps and audio-off omission while providing coherent audio-enabled review defaults.
- Height-based aspect preservation, nearest-even width with upward ties, minimum width 2 and checked integer formula are deterministic and internally consistent.
- Range-only scope leaves frame/draft and export behavior unchanged; no persisted schema or migration is needed.
- Core ownership of settings resolution, immutable revision rendering, stable typed failures and early failure safety remain aligned with repository architecture.
- Planned aspect/boundary/default/audio/opt-out/parity/reopen/state, transport, capability, canonical contract and MCP workflow coverage is appropriate. Required final checks and designated human CODEOWNER review remain separate gates.

No remaining specification blockers. Implementation may proceed through the approved tasks. This acceptance does not claim that implementation checks or canonical parity have already passed.

## Architecture addendum acceptance

Accepted the concrete renderer-to-validation dependency for the new review-options resolver to call the existing `validate_project_settings` implementation. This points inward to the canonical domain owner, avoids a duplicate bounds/fps policy and introduces no cycle (validation does not depend on renderer). ADR 0003 explicitly permits reviewed new edges when the ADR and architecture test change together. Record this precise edge and its rationale in design decision 3, ADR 0003's dependency matrix and the architecture test in the same change; retain the existing renderer responsibility exclusions. Scope the call to the new review resolver so legacy numeric range behavior remains unchanged.

A renderer-owned serializable request selection/options DTO is acceptable as an ephemeral render API contract. It does not alter the persisted project model or schema; do not introduce persisted settings or backend-specific types into renderer-neutral evaluated-scene structures. Headless/MCP parity must still consume the canonical public request fixtures/catalogs.
