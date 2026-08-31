# OpenCut contributor architecture

These instructions apply to the entire repository. Keep changes inside the owning layer and preserve the dependency direction described below.

## Mandatory spec-driven development

All implementation work MUST follow the OpenSpec lifecycle documented in `docs/spec-driven-development.md`. This is a hard gate, not a recommendation. An implementation includes every change to executable behavior, contracts, schemas, migrations, configuration, user-visible behavior, defect handling, or architecture. Bug fixes and refactors are not exempt.

Before editing implementation files:

1. Read the applicable living requirements in `openspec/specs/` and inspect active changes in `openspec/changes/`.
2. Identify the approved OpenSpec change that authorizes the work. Its proposal, delta specs, design, and tasks MUST describe the requested outcome, affected behavior, constraints, failure cases, compatibility impact, and verification plan.
3. If that coverage is absent, incomplete, ambiguous, or inconsistent with the request, STOP. Create or update the OpenSpec artifacts first, then obtain explicit user or reviewer approval before implementation. Never infer missing requirements or silently expand scope in code.
4. Implement only through the approved change's `tasks.md`, keep task status accurate, and maintain traceability from every implementation edit and test to a requirement and scenario. Do not add behavior that has no corresponding requirement.

Implementation MUST demonstrate conformance, not merely compile:

- Add or update tests for every changed scenario before considering its implementation complete. Each normative requirement MUST have automated coverage, or a documented justification in the change when automation is technically impossible.
- Keep code, contracts, fixtures, migrations, documentation, design decisions, delta specs, and tasks synchronized throughout the change.
- Run `moon run openspec-validate` plus all affected formatting, linting, type, unit, integration, migration, and smoke checks. A failed or skipped required check blocks completion and MUST be reported explicitly.
- Use `$openspec-verify-change` after implementation. Resolve every mismatch among requirements, design, tasks, tests, and code before presenting the work as complete.
- Archive the verified change with `$openspec-archive-change` so accepted deltas are merged into `openspec/specs/`. Work is not complete while its specification is stale or its OpenSpec change remains unverified.

No agent, contributor, urgency label, or user request to "just code it" overrides these gates. If approval or required specification detail is unavailable, report the work as blocked rather than modifying implementation.

## Ownership boundaries

ADR 0003 (`docs/adr/0003-editor-core-module-boundaries.md`) is the canonical editor-core ownership map and allowed dependency graph. Any new dependency edge requires the ADR and the editor-core architecture test to change in the same review. Headless, bridge, and desktop code must not add parallel validation for project, timeline, asset, draft, migration, or rendering semantics; submit typed input to `editor-core` and translate its result instead.

- `crates/editor-core` owns domain models, validation, persistence, revisions, undo/redo, atomic mutations, schema migration, media ownership, integrity checks, garbage collection, and rendering rules. It must not depend on MCP, provider workers, environment configuration, or presentation concerns.
- `apps/headless` is the typed JSON-lines process transport over `editor-core`. It deserializes requests, emits events and responses, and translates core errors without duplicating domain or persistence rules.
- `apps/agent-bridge` owns application workflows, MCP adapters, process-local jobs, immutable runtime configuration, diagnostics, structured logging, and provider orchestration. Keep transport handlers thin and inject services rather than importing concrete providers into capability registrars.
- `apps/kokoro-tts` is a replaceable, CPU-only speech provider worker. It owns inference, provider metadata, and WAV production, and has no knowledge of projects, assets, or timelines.
- `apps/desktop` owns presentation and interaction. Reuse `editor-core` behavior instead of recreating domain validation or mutations in UI code.
- `contracts` contains canonical cross-language catalogs and fixtures. Rust, TypeScript, and Python contract tests must agree with changes made there.

Dependencies point inward: presentation, transports, and provider adapters may call application or core behavior, but `editor-core` must never depend on those outer layers. Provider-specific concepts stay behind provider-neutral bridge contracts.

## Compatibility and migrations

- Treat public request, response, project, and error contracts as compatibility surfaces. Prefer additive changes, preserve documented aliases, and keep stable error codes and retryability aligned with the canonical catalog.
- Follow ADR 0002 and `contracts/contract-ownership-v1.json` for every cross-language public contract change. Update the canonical versioned fixture or catalog, every listed consumer, and parity evidence in the same change; run `bun run contracts:check` from `apps/agent-bridge` and obtain review from the designated CODEOWNER.
- Classify optional request fields, ignorable response fields, and new uniquely named operations, capabilities, or resources as additive only when existing clients remain valid. Removing, renaming, narrowing, changing meaning or retryability, or reusing an identifier is breaking and requires a new major contract plus an explicit migration path.
- Persisted schema changes require deterministic migration tests for current state and retained undo/redo history. Perform migrations under the project lock, preserve provenance and media integrity, and update related snapshots atomically.
- Reject unknown future schema versions rather than guessing how to downgrade them.

## Testing and maintenance

- Before marking an improvement complete, run Rust formatting, workspace-wide strict Clippy, workspace tests, TypeScript typecheck/lint/unit tests, MCP integration, packaged smoke, and the hermetic Python worker tests relevant to the change.
- The dormant desktop prototypes are the sole warning exception: `apps/desktop/src/components/mod.rs` may locally allow `dead_code` and `unused_imports`. Remove the allowance when components become used, or delete abandoned prototypes. Do not add crate-wide or workspace-wide warning suppression.
- Preserve unrelated working-tree changes. Keep edits scoped, do not overwrite user work, and do not commit secrets, local data, dependency caches, or generated build output.
