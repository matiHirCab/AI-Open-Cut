# OpenCut contributor architecture

These instructions apply to the entire repository. Keep changes inside the owning layer and preserve the dependency direction described below.

## Ownership boundaries

- `crates/editor-core` owns domain models, validation, persistence, revisions, undo/redo, atomic mutations, schema migration, media ownership, integrity checks, garbage collection, and rendering rules. It must not depend on MCP, provider workers, environment configuration, or presentation concerns.
- `apps/headless` is the typed JSON-lines process transport over `editor-core`. It deserializes requests, emits events and responses, and translates core errors without duplicating domain or persistence rules.
- `apps/agent-bridge` owns application workflows, MCP adapters, process-local jobs, immutable runtime configuration, diagnostics, structured logging, and provider orchestration. Keep transport handlers thin and inject services rather than importing concrete providers into capability registrars.
- `apps/kokoro-tts` is a replaceable, CPU-only speech provider worker. It owns inference, provider metadata, and WAV production, and has no knowledge of projects, assets, or timelines.
- `apps/desktop` owns presentation and interaction. Reuse `editor-core` behavior instead of recreating domain validation or mutations in UI code.
- `contracts` contains canonical cross-language catalogs and fixtures. Rust, TypeScript, and Python contract tests must agree with changes made there.

Dependencies point inward: presentation, transports, and provider adapters may call application or core behavior, but `editor-core` must never depend on those outer layers. Provider-specific concepts stay behind provider-neutral bridge contracts.

## Compatibility and migrations

- Treat public request, response, project, and error contracts as compatibility surfaces. Prefer additive changes, preserve documented aliases, and keep stable error codes and retryability aligned with the canonical catalog.
- Persisted schema changes require deterministic migration tests for current state and retained undo/redo history. Perform migrations under the project lock, preserve provenance and media integrity, and update related snapshots atomically.
- Reject unknown future schema versions rather than guessing how to downgrade them.

## Testing and maintenance

- Before marking an improvement complete, run Rust formatting, workspace-wide strict Clippy, workspace tests, TypeScript typecheck/lint/unit tests, MCP integration, packaged smoke, and the hermetic Python worker tests relevant to the change.
- The dormant desktop prototypes are the sole warning exception: `apps/desktop/src/components/mod.rs` may locally allow `dead_code` and `unused_imports`. Remove the allowance when components become used, or delete abandoned prototypes. Do not add crate-wide or workspace-wide warning suppression.
- Preserve unrelated working-tree changes. Keep edits scoped, do not overwrite user work, and do not commit secrets, local data, dependency caches, or generated build output.
