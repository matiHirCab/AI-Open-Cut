## Context

Main `84da152f` has `CoreError` and thin headless `ErrorBody`/bridge `BridgeError` translations. Stable errors are governed by error-codes-v1, events by headless-protocol-v1, and MCP output schemas by mcp-surface-v1. ADR 0003 keeps domain accounting inward. Existing shape, extended certification, motion blur and linear composition accounting already computes the values needed for the selected guards. No active unrelated OpenSpec changes exist.

## Goals / Non-Goals

Implement the proposal's closed diagnostic coverage without changing guard thresholds/order or persisted semantics. No provider-origin details, runtime memory sampling, completeness claim for all guards, progress, decoder/spawn, desktop, cache redesign or quality fallback.

## Decisions

1. Core owns a typed optional `renderBudget`, omitted for ordinary errors. Budget identifiers: `shape_segments`, `scene_shape_segments`, `scene_shape_raster_bytes`, `candidate_analysis_nodes`, `effect_pixel_passes`, `motion_blur_pixel_work`, `composition_pixel_work`, `composition_bytes`. Units are `segments`, `bytes`, `nodes`, `pixel_passes`, `pixel_samples`. Measurement is `charged` for exact admission counters/prefixes or `conservative` for allocation/effect bounds. Work means attempted accounting at this guard, not complete scene cost or observed RSS. Shape allowance can be smaller than the per-shape canonical maximum because an aggregate owner passes its remaining allowance. Message parsing and an estimator in transports are rejected.
2. Limit and work use unsigned decimal strings bounded to 20 digits to preserve u64 values across JS/JSON. `work` is null exactly when `overflow` is true; otherwise it is the value already computed by the owner before comparing its limit. Checked arithmetic remains checked. Do not invent a numeric overflow estimate. Derived expanded composition memory uses the same conservative classification.
3. Contributors are anonymous `{kind, work}` charges, with closed kinds `shape`, `grid`, `svg`, `effect`, `motion_blur`. At most eight entries; shape aggregate accounting retains the largest eight observed charges in descending charge/kind order. Single-charge owners expose their current charge when available; otherwise the list is empty. The list is explicitly partial, not a decomposition of total scene cost, and considers a known rejected charge in selection; eight larger charges can exclude it. No names, IDs, paths, source text, commands, source documents or arbitrary labels. Error module types do not depend on evaluated_scene; owning accounting supplies enum values and numbers.
4. Headless emits the optional field in standalone, status and worker error paths. Bridge validates the closed bounded payload from core events and carries it through BridgeError, errorBody, job polling and MCP. Malformed fields fail existing contract validation. Speech/transcription provider event schemas do not acquire native diagnostics. Stable code/message/retryability and FFmpeg detail fields remain unchanged; admission is not FFMPEG_FAILED or a renderer defect.
5. Add a canonical render-budget-diagnostics-v1 catalog with valid/invalid fixtures and ownership consumers. Manually add only the optional public error schema property in MCP/headless fixtures. Pin the complete predecessor catalog and remove exactly this addition in compatibility projection tests before existing historical projections; retain historical digest assertions and negative controls. No generated catalog from registrations, version bump or persistence migration.

## Risks / Trade-offs

- Partial guard coverage → publish an explicit coverage table and retain ordinary errors for unsupported accounting; never imply all guard failures have this field.
- Contributor retention overhead → fixed maximum eight anonymous numeric charges; no geometry retention or second traversal.
- ABI/error size → box the optional diagnostic inside CoreError and reuse inward types.
- Optional schema additions affect historical parity → exact inverse projection and pinned unchanged predecessor, synchronized canonical consumers.
- Resource failure precedence/atomicity → actual headless/MCP rejection with owned fake execution marker and existing destination/project/history/draft byte snapshots; boundary/overflow owner tests.

## Migration Plan

Ignorable optional field under version 1; absent-field behavior is identical. Project schema remains current main's 44 unless upstream changes independently. Rollback removes only diagnostic reporting and its canonical field. No credentials/configuration/deployment change.

## Approved architecture refinement

The full workspace architecture gate identified the new direct private `evaluated_scene -> error` import for bounded numeric contribution retention and checked-charge error construction. This inward primitive dependency SHALL be explicitly registered in ADR 0003 and every corresponding architecture allowlist in the same review. The error owner remains dependency-free; evaluated_scene retains geometry, admission limits, estimation and scene ownership. No renderer, persistence, transport or provider edge is added. Re-exporting private bookkeeping through the root solely to hide this import is not the chosen design.

## Open Questions

None blocking. Designated reviewer is @matiHirCab; the authenticated author cannot request self-review (GitHub 422 already established). Human owner review remains separate from automated verification.

## Approved hermetic canonical verification refinement
The first published3761617d contract job114670390933 reported DEPENDENCY_UNAVAILABLE before the new budget assertion because that canonical environment intentionally has no FFmpeg. The actual-native jobs are separate and green. The headless admission test SHALL compile an owned readiness-only executable: only exact filter/version inspection is accepted; any attempted rendering records an owned marker and fails. Only this test overrides its tools, then asserts no execution marker or publication. No production readiness behavior, native dependency requirement, decoder/renderer simulation, budget or timeout is changed. This fixture establishes actual core accounting/serialization in a hermetic transport process, not rendered-video evidence.

## Approved rollback completeness refinement
The MCP addition remover detects both properties and the named RenderBudgetDiagnostic definition. If either exists, every existing error owner must carry the reviewed budget field; otherwise reject with budget-specific text. Validate existing exact schemas before removal. Add all-six-missing controls for direct removal and raw restoration, plus historical no-field/no-definition identity and pinned-byte controls. The disposable combined union must independently exercise the same negative case against the corrected peer head. No union imports enter the owning branch. Reuse unchanged production/native/Python checks while refreshing complete canonical, full bridge unit, type/lint, strict-spec and protected gates. Hosted exact-head CI remains mandatory.
