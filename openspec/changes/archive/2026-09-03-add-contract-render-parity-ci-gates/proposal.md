## Why

The canonical cross-language fixtures and renderer golden references now exist, but their checks are embedded in broader CI jobs whose status does not expose either parity boundary independently. Issue #16 requires stable, required CI gates that fail directly on contract drift or reviewed render-output regressions before later motion-graphics milestones proceed.

## What Changes

- Add a dedicated contract-parity CI job that runs the standalone Rust/TypeScript/Zod/MCP fixture gate from a pinned repository checkout.
- Add a dedicated Linux render-parity CI job that configures deterministic FFmpeg, FFprobe, and font dependencies, runs native golden conformance, validates the report-only baseline, and retains the validated observation artifact.
- Keep lifecycle, general correctness, integration, and packaged-smoke coverage while removing duplicate parity execution from broader jobs and making their dependency on the dedicated foundations explicit.
- Add automated workflow-policy evidence that fails if either named gate, its strict command, deterministic environment, report validation, or artifact publication is removed or weakened.
- Document the stable gate names and local reproduction commands for contributors and branch-protection configuration.
- Non-goals: change application behavior, public requests or responses, MCP registrations, capability/version reporting, persisted project data or history, renderer semantics, golden thresholds, benchmark budgets, or canonical fixture contents.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `contract-governance`: Require a dedicated, independently visible CI status for the standalone cross-language contract-parity command.
- `render-regression-fixtures`: Require a dedicated, independently visible Linux CI status for deterministic render conformance and validated report publication.
- `repository-validation`: Define stable foundation-gate workflow structure and automated policy evidence without changing application or contract behavior.

## Impact

- Affects `.github/workflows/bun-ci.yml`, repository workflow-policy tests, contributor documentation, and the three listed living specifications.
- Reuses `bun run contracts:check` and editor-core's existing native golden/report validators; it adds no production dependency and changes no runtime ownership boundary.
- The change is CI-only and non-breaking. Public contracts, protocol versions, stable errors, project schema version, revisions, undo/redo, migrations, aliases, reopen behavior, and preview/export output remain unchanged.
