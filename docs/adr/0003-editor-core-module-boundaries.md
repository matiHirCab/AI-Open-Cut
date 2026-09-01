# ADR 0003: Editor-core module boundaries

- Status: Accepted
- Date: 2026-08-31
- Issue: [#83](https://github.com/matiHirCab/AI-Open-Cut/issues/83)
- Related roadmap: [#12](https://github.com/matiHirCab/AI-Open-Cut/issues/12), [#13](https://github.com/matiHirCab/AI-Open-Cut/issues/13)

## Context

The editor core is the canonical owner of persisted editor semantics, but its store and renderer implementations accumulated unrelated responsibilities. Continuing to add scene-graph, animation, compositing, and audio behavior to those modules would make domain rules, infrastructure failures, and renderer details difficult to test or change independently.

This decision establishes one owner for each responsibility while preserving the existing `EditorCore` and `Renderer` facades, serialized documents, stable errors and warnings, revision semantics, and render behavior.

## Decision

Dependencies point inward through the following ownership graph:

```text
apps/desktop ─┐
apps/agent-bridge ─> apps/headless ─> editor-core facades
              │
              └────────────── no domain-rule ownership

store facade ─> assets ─> persistence
     ├─────────> drafts ─> persistence
     ├─────────> migrations
     ├─────────> persistence
     ├─────────> timeline ─> animation/validation
     └─────────> validation

renderer facade ─> render_artifact ─> render_plan ─> animation
        ├────────> render_plan
        └────────> render_process ─> render_plan
```

Root facade re-exports provide model and error types without adding an outward owner edge. The complete private-owner matrix is:

| Owner | Allowed private-owner imports |
| --- | --- |
| `animation` | none |
| `assets` | `persistence` |
| `drafts` | `persistence` |
| `error` | none |
| `migrations` | none |
| `model` | `error` |
| `path_policy` | none |
| `persistence` | none |
| `render_artifact` | `render_plan` |
| `render_plan` | `animation` |
| `render_process` | `render_plan` |
| `renderer` | `render_artifact`, `render_plan`, `render_process` |
| `store` | `assets`, `drafts`, `migrations`, `persistence`, `timeline`, `validation` |
| `timeline` | `animation`, `validation` |
| `validation` | none |

### Canonical owners

| Concern | Canonical owner | Must not own |
| --- | --- | --- |
| Serialized editor model | `model` | I/O, process execution, transport schemas |
| Domain validation | `validation` | Persistence, FFmpeg, presentation validation copies |
| Timeline operations and history transitions | `timeline` | Filesystem and transport behavior |
| Asset references, integrity, and managed-content policy | `assets` | Transport/provider behavior |
| Schema upgrades | `migrations` | Call orchestration and presentation fallback |
| Locking, durable transactions, recovery, and document I/O | `persistence` | Timeline/domain decisions |
| Durable draft lifecycle | `drafts` | Independent validation rules |
| Public project orchestration | `store` | Duplicate implementations of the inward owners |
| Scene evaluation and declarative render planning | `render_plan` | Process spawning, environment lookup, artifact publication |
| FFmpeg/FFprobe execution and diagnostics | `render_process` | Scene/domain rules and output publication policy |
| Workspaces, prepared resources, temporary output, and publication | `render_artifact` | Scene evaluation and process semantics; it may consume resource requests from `render_plan` |
| Stable renderer API orchestration | `renderer` | Duplicate implementations of the three render owners |

### Persistence port

Persistence logic depends on a crate-private, I/O-shaped port. It exposes only the operations required for locked document access, synchronized atomic replacement, durable removal, directory and metadata access, and managed-file copying. The production adapter uses the filesystem. Deterministic tests inject named failure phases through the same boundary. Transaction state, commit points, recovery decisions, migration decisions, and public warning/error mapping remain outside the adapter.

### Renderer ports

Render planning emits declarative, comparable planning data and never starts a process or publishes output. A process executor accepts structured program arguments and reports bounded outcomes. Artifact I/O owns workspace lifetime, temporary paths, overwrite/collision policy, publication, cleanup, and metadata. The public renderer coordinates these owners and maps failures to the existing `CoreError` behavior.

The scene-to-plan input is the only handoff seam reserved for `EvaluatedScene`. Issue #12 owns that future representation; issue #13 will route all render entry points through it. This ADR deliberately does not define a placeholder public scene model.

## Compatibility constraints

- `EditorCore`, `Renderer`, and their existing public DTO names and signatures remain available through `lib.rs`.
- Project, history, and draft Serde shapes and schema versions do not change.
- Stable error codes, retryability, warning strings, revisions, undo/redo, reopen, and recovery behavior do not change.
- Preview and export visual/audio behavior, path safety, overwrite rules, and artifact metadata do not change.
- Headless, bridge, and desktop code delegate canonical validation to editor-core.

## Enforcement and review

Rust module privacy is the primary enforcement mechanism. `crates/editor-core/tests/architecture.rs` checks every private owner against the explicit import matrix, the stable public facade, and reviewed responsibility exclusions such as process creation in planning, project traversal or command construction in the renderer facade, and managed-asset collection in the store. Semantic correctness remains covered by owner-focused and facade integration tests.

A change that needs a new dependency edge must update this ADR and the architecture test in the same pull request. Reviewers must verify that the edge still points inward and does not create a second owner. Public or persisted contract changes additionally follow ADR 0002 and the contract-governance workflow.

## Consequences

The editor core gains more private modules and explicit adapter types, but callers keep one stable facade. Pure domain and planning tests no longer require filesystem or FFmpeg setup. Future `EvaluatedScene` work can replace the inward scene input without moving process or artifact concerns into the scene model.
