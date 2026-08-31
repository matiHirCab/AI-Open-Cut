## Context

`crates/editor-core/src/store.rs` currently combines the public `EditorCore` facade, public request/result DTOs, project orchestration, canonical validation, timeline mutation, asset reference discovery and managed-file lifecycle, drafts, migrations, history, locking, crash-consistent persistence, and tests in one module. `renderer.rs` similarly combines public renderer DTOs, project-to-filter-graph planning, text preparation, FFmpeg/FFprobe spawning, progress and stderr handling, temporary outputs, artifact publication, and tests.

Those implementations already satisfy substantial compatibility and recovery requirements. The change is therefore a behavior-preserving extraction, not a rewrite. The public Rust facade is consumed by headless and transitively by the TypeScript bridge; persisted projects, history, drafts, stable errors/warnings, previews, and exports are compatibility surfaces. The untracked `docs/motion-graphics-implementation-plan.md` is unrelated user work and will not be modified.

Issues #12 and #13 will introduce a renderer-neutral `EvaluatedScene` and route all render entry points through it. This change must establish a plan/execution seam that those issues can use, but must not invent their model or observable semantics prematurely.

## Goals / Non-Goals

**Goals:**

- Give every editor-core responsibility one documented owner and an enforceable inward dependency direction.
- Split store responsibilities into cohesive modules with focused tests while retaining `EditorCore` as the stable orchestration facade.
- Keep canonical validation and mutation semantics in one inward domain layer.
- Put filesystem persistence and FFmpeg/process/artifact details behind narrow interfaces that support deterministic fault tests.
- Separate render planning from execution and publication, leaving one seam for future `EvaluatedScene` input.
- Preserve all serialized and public behavior and verify existing reopen, preview, and export paths.

**Non-Goals:**

- Changing persisted schemas, public protocols, stable error catalogs, capabilities, or MCP operations.
- Implementing `EvaluatedScene`, scene graphs, new animation/compositing/audio behavior, or renderer output changes.
- Generalizing editor-core into multiple crates or adding a dependency-injection framework.
- Moving domain validation into headless, bridge, desktop, or renderer infrastructure.

## Decisions

### 1. Use one public facade over private cohesive modules

`EditorCore`, its public request/result DTOs, and the existing renderer entry points remain re-exported from `lib.rs` with their current names. The store facade coordinates the following private owners:

| Owner | Responsibility | Allowed inward dependencies |
| --- | --- | --- |
| `model` | Serialized project/history/domain types | none beyond serialization/value helpers |
| `validation` | Canonical settings, timing, transform, style, audio, keyframe, track/item, batch, and draft validation | `model`, `error`, pure animation helpers |
| `timeline` | Pure edit application, aliases, revision/history transitions | `model`, `error`, `validation`, pure animation helpers |
| `assets` | Asset-reference inventory, integrity rules, content addressing, retained managed-path policy, and GC decisions | `model`, `error`, `path_policy`; storage only through the persistence seam |
| `migrations` | Deterministic current-project and retained-history upgrades | `model`, `error`, `assets` integrity helpers |
| `persistence` | Project locking, JSON documents, transaction journal/recovery, atomic replacement, draft documents, and the storage interface/adapters | `model`, `error`, `migrations`; no timeline or renderer dependency |
| `drafts` | Draft lifecycle and candidate materialization | `model`, `error`, `validation`, `timeline`, `assets`, persistence contracts |
| `store` | Stable `EditorCore` orchestration facade and public DTOs | all preceding owners, with no reverse dependency |

Focused tests move with their owner. Cross-owner compatibility and end-to-end persistence tests stay at the facade level.

Alternative considered: publish each extracted module and expose new service objects. Rejected because it expands the public API, lets callers bypass orchestration invariants, and makes compatibility harder.

Alternative considered: keep helpers in `store.rs` and only split tests. Rejected because it does not establish dependency direction or cohesive ownership.

### 2. Introduce a small persistence port with a production filesystem adapter

Persistence orchestration will depend on a crate-private storage port covering only the operations it needs: locked project access, bounded document reads, synchronized temporary writes and atomic replacement, durable removal, directory enumeration, metadata, and managed-file copy/hash operations. Production uses the existing filesystem behavior. Tests use a deterministic faulting adapter or fault policy at named transaction phases.

The transaction state machine, commit point, recovery validation, warning mapping, and schema migration decisions remain canonical persistence logic; adapters only perform requested I/O and report failures. `PathPolicy` remains the authority for allowed external/project paths.

Alternative considered: mock `std::fs` globally or retain thread-local faults inside individual helpers. Rejected because it leaves filesystem details spread across domain owners and makes failures difficult to target without production-only branches.

Alternative considered: make `EditorCore` generic over storage. Rejected because generic parameters would leak through the public facade. The injected port stays crate-private behind a concrete facade constructor used by production, with a test-only/internal construction path.

### 3. Make renderer planning declarative and execution replaceable

The renderer becomes a facade over three owners:

- `render_plan`: deterministic evaluation of current project state into ordered inputs, filter/composition/audio instructions, timing, output intent, and required prepared resources. It contains no `Command`, process spawn, final artifact mutation, environment lookup, or output publication.
- `render_process`: FFmpeg/FFprobe readiness, probing, process invocation, progress, bounded diagnostics, cancellation/exit mapping, and a narrow executor port. It consumes a declarative invocation produced from a plan.
- `render_artifact`: workspace lifetime, text/resource preparation that requires I/O, temporary output allocation, collision/overwrite policy, atomic publication, cleanup, and artifact metadata.

The public `Renderer` owns concrete production adapters and coordinates these stages. Tests can inspect plans without FFmpeg, inject process outcomes, and inject publication failures. Existing smoke tests still exercise the real adapters when FFmpeg is available.

Alternative considered: expose `std::process::Command` as the plan. Rejected because `Command` is not renderer-neutral, is awkward to compare deterministically, and couples evaluation to execution.

Alternative considered: wait for `EvaluatedScene` before extracting renderer modules. Rejected because #83 must establish the boundary that #12/#13 depend on; waiting would force those changes to edit the same concentration first.

### 4. Reserve one scene-to-plan input seam without defining the roadmap model

Today, a crate-private scene/planning input adapter derives the plan from `Project`. Every public render entry point passes through the same plan builder. Issue #12 may later add `EvaluatedScene` on the inward side of this seam, and #13 may switch frame, range, draft, and export entry points to that input. FFmpeg execution and artifact adapters will remain unchanged by that substitution.

Alternative considered: define a placeholder public `EvaluatedScene` now. Rejected because #12 owns its coordinate, timing, ordering, limits, and validation semantics; guessing them would create an unapproved public/domain contract.

### 5. Enforce boundaries with compiler privacy plus explicit source checks

Module privacy provides the primary enforcement. A focused architecture test additionally scans editor-core module imports/declarations and fails on documented forbidden edges, including domain/planning dependencies on app crates, transports, environment configuration, process execution, or artifact publication. The ADR records the same ownership matrix and requires an ADR plus test update for any new allowed edge. Root contributor guidance links to the ADR and states that bridge/desktop validation is prohibited.

Alternative considered: documentation-only review rules. Rejected because the issue explicitly requires prevention, and documentation alone cannot catch dependency inversion in CI.

Alternative considered: add a third-party architecture-lint dependency. Rejected because the required graph is small, Rust privacy already enforces much of it, and a targeted repository test avoids a new supply-chain/runtime dependency.

## Compatibility, Failure Modes, and Security

- No public DTO, Serde attribute, stable error/warning, persisted field, schema version, or headless/bridge contract changes are permitted.
- Extraction must preserve project locking, transaction commit/recovery semantics, path-policy checks, content hashing, GC reachability, bounded stderr, path redaction, output collision/overwrite behavior, and temporary cleanup.
- Storage and process adapters return internal failures that the existing facade maps to current `CoreError` codes and retryability. Adapters do not create new public errors.
- Render plans contain structured arguments and paths, never shell command strings or raw executable expressions supplied by callers.

## Migration Plan

1. Add the ADR/ownership map and architecture test before moving behavior.
2. Extract validation and pure timeline logic, preserving facade tests and public re-exports.
3. Extract asset ownership, migrations, persistence, and drafts incrementally; run focused and reopen/recovery tests after each move.
4. Add the persistence port and migrate production filesystem behavior and named fault tests to it.
5. Extract render planning, process execution, and artifact I/O one stage at a time; retain golden/structural plan tests plus existing FFmpeg smoke comparisons.
6. Run all required Rust, headless, bridge, OpenSpec, integration, and packaged smoke checks.

Rollback is file-level and behavior-neutral: because no persisted or public contract changes occur, any extraction stage can be reverted to the prior module placement without data migration. Partially extracted stages must not merge unless the facade suites and architecture checks pass.

## Risks / Trade-offs

- [Large mechanical moves can hide behavior changes] → Move one owner at a time, preserve tests, compare public signatures/Serde shapes, and run focused suites after every extraction.
- [Overly broad storage ports can become an alternate domain layer] → Keep methods I/O-shaped and keep transaction, migration, validation, and warning decisions outside adapters.
- [A declarative FFmpeg plan can accidentally encode future `EvaluatedScene` semantics] → Model only current rendering facts and keep the scene-to-plan input crate-private until #12 defines the canonical representation.
- [Architecture checks can become brittle text matching] → Enforce only high-value forbidden edges, rely on Rust privacy/compiler checks for symbol access, and document the exact reviewed graph in the ADR.
- [Module cycles may appear during extraction] → Keep shared DTOs in `model`/facade types, move pure rules inward, and route cross-cutting I/O through persistence contracts rather than adding reverse imports.

## Open Questions

None required before implementation. Exact private type and file names may be adjusted during extraction provided the ownership graph, public facade, and normative requirements remain unchanged.
