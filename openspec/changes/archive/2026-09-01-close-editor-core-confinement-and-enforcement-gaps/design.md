## Context

`existing_project_dir` currently probes the candidate's project and transaction files before canonicalizing the directory. A syntactically valid ID backed by a linked external directory with no project markers therefore returns `PROJECT_NOT_FOUND` after filesystem inspection outside the configured root instead of failing confinement first.

The `syn` architecture analyzer resolves aliases declared as module items, but its block pre-scan handles only `use`. Legal block-local `extern crate` and path-based type aliases can therefore hide forbidden owner dependencies or direct filesystem calls. The native `Path` inspection list also omits `is_symlink`.

## Goals / Non-Goals

**Goals:**

- Prove project-root confinement before any marker probe, lock, persisted read, recovery, or garbage collection.
- Apply the existing deterministic alias model equally to module and block item scopes.
- Reject every currently supported filesystem-observing `Path` method, including `is_symlink`, in owners that must delegate I/O.
- Preserve public and persisted contracts, normal project behavior, stable diagnostics, and the existing ownership graph.

**Non-Goals:**

- Implementing broader security/stress gates, SVG ingestion, `EvaluatedScene`, render features, job retention, cancellation, or durability.
- Changing transaction recovery, asset reachability, garbage-collection policy, adapter visibility, or ADR 0003.
- Adding compiler name resolution, macro expansion, or type inference to the architecture analyzer.

## Decisions

### 1. Canonical confinement precedes existence classification

`existing_project_dir` will canonicalize the syntactically validated candidate through `Storage` first. A canonical result outside the already-canonical project root returns `PATH_NOT_ALLOWED` immediately. Only after containment succeeds may store orchestration probe the canonical project and transaction markers.

An `io::ErrorKind::NotFound` from canonicalizing an ordinary absent directory maps to the existing `PROJECT_NOT_FOUND` result; other canonicalization failures retain the existing I/O diagnostic. Marker probes that find neither persisted project nor recoverable transaction also retain `PROJECT_NOT_FOUND`.

Alternative considered: classify the candidate with non-following metadata before canonicalization. Rejected because classification would not establish the target's canonical containment and would add another pre-confinement filesystem operation.

### 2. Module and block scopes share item-alias discovery

Alias discovery will consume production-relevant `Item` references regardless of whether they come from a module item list or `Stmt::Item` entries in a block. It will seed `extern crate` and path-based type aliases, then resolve `use` trees against inherited and newly seeded aliases using the existing bounded fixed-point algorithm.

The enclosing block receives one lexical alias scope for all item declarations because Rust item scope covers the complete block. Test-only items do not seed that scope. Existing exact owner matching, ancestry handling, visited-state cycle detection, and diagnostic formatting remain unchanged.

Alternative considered: add separate local handling only for `extern crate`. Rejected because local type aliases create the same canonicalization bypass for UFCS filesystem calls and duplicated logic would drift again.

### 3. Native Path inspection uses one explicit complete set

The existing structural rule will add `is_symlink` to the explicit filesystem-observing `Path` method set. The same set drives both method-name and canonical qualified-path checks, covering method syntax, UFCS, and aliases without Rust type inference. Adapter-specific method names remain outside that set.

Alternative considered: reject every method call on values named `path`. Rejected because the AST analyzer has no reliable receiver type information and that heuristic would create unrelated false positives.

## Risks / Trade-offs

- [Canonicalization failure must distinguish absence from other I/O errors] -> match only `io::ErrorKind::NotFound` to `PROJECT_NOT_FOUND` and preserve the current I/O mapping for every other error.
- [Block pre-scanning makes aliases visible before textual declaration] -> this matches Rust item scope and is covered with local alias and test-only fixtures.
- [Method-name enforcement remains syntactic] -> keep authorized adapter operations uniquely named and cover both rejected native calls and accepted adapter calls.
- [Security-related wording could absorb future issue #79] -> limit this change to the existing project-root and architecture invariants; do not add release gates, resource limits, or new ingestion behavior.

## Migration Plan

No data, public contract, or runtime migration is required. After approval, add regressions, implement the private changes, run the complete repository gates, verify and archive this OpenSpec change, review `main...HEAD`, and publish one additional commit to PR #94. Rollback is that single commit.

## Open Questions

None.
