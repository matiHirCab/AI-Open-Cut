## Context

Windows run38021229087/job114122548321 at d64e539e passed Rust/type/lint but failed target_measurement/deadline PID readiness:843 unit passes,1 failure,15 configured skips. Full original log /tmp/mg79-ci-d64-windows-correctness-failure.log SHA2566c01239eeb4bf688b7b71aeddf638ddf83a327eb94df2cac91089de5c53b7a98. Earlier main failure was target_measurement/shutdown; original fixture phase/outcome files are not in the log. Precise cause remains unproved.

## Goals / Non-Goals

Stabilize the private fixture process launch path while retaining actual ownership proof and bounded failure behavior. Preserve all54 cases, phase selection, metrics/finite PCM/fault cases, completed outputs, overlapping request isolation/reuse and authoritative bytes. No production, public, migration, timeout, CI workflow, audit branch or #80 change.

## Decisions

Compile a std-only Rust fixture once to an isolated owned temporary directory with the existing pinned compiler. Compilation is a separate20s setup command inside a30s beforeAll hook, never a change to the original controlled/cold limits. Each fresh test project uses direct copied native ffmpeg/ffprobe names. The private backend emits the existing stub metrics and PCM and starts the same native binary in descendant mode, avoiding repeated cmd/Bun interpreter dispatch. This is transport/lifetime evidence, not real FFmpeg/media correctness and must not forge production cache identity. Actual native oracles remain #77/#79's distinct existing gates.

Retain original per-case cold5000ms preparation and controlled test5000ms/readiness1500ms/deadline2000ms. Assert both PIDs live before trigger and absent after completion. Readiness failure aborts and settles only its pending request for safe diagnostics, records actual phase events and outcome code, then rethrows the original failed assertion. No retry or accepted missing PID. Build executable reuse shares no project, control, history or completed outputs.

Alternatives: increasing waits or retries would weaken required limits; replacing PID assertions with counters would lose actual process proof; production cancellation edits have no evidentiary basis. Keeping interpreted backends retains repeated startup overhead. Compiled Bun bundles retain interpreter initialization and a much larger executable; std-only native fixture is simpler for this limited controlled protocol.

## Risks / Trade-offs

Fixture protocol drift -> all original54 actual success/fault/lifecycle scenarios remain mandatory, plus direct setup/probe/unknown-argument controls. Rust compiler unavailable -> setup fails, no skip/fallback. Windows startup cause may differ -> exact-head all-platform required CI remains the acceptance gate and failures retain phase/outcome diagnostics. No inferred physical media quality from stub output. Temporary executable locks -> close clients before owned fixture cleanup with existing bounded filesystem retries.

## Migration Plan

No migration or rollout. Commit only source fixtures/tests, verified archived requirements and evidence. Preserve failed-head evidence; require exact new-head CI before #80. Revert this bounded fixture change independently if needed.

## Open Questions

The old Windows readiness failure's precise startup/phase cause remains unresolved without its retained private files; do not claim a production defect or root-cause proof.
