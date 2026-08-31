## Context

PR #94 introduced an explicit owner-to-allowed-import matrix in `crates/editor-core/tests/architecture.rs`. The current check treats an owner as imported only when the production source contains `<owner>::`. Rust also permits direct, grouped, nested, and aliased module imports, so `use crate::{persistence as storage}; storage::read_json(...)` can introduce a forbidden edge without containing `persistence::`.

The correction is test-only enforcement. It must not change the approved matrix, production dependencies, public APIs, persisted formats, or runtime behavior.

## Goals / Non-Goals

**Goals:**

- Extract every editor-core owner named by a production `use crate::...` declaration, including aliases and grouped/nested imports.
- Keep the matrix centralized and produce failures that identify the importing owner and forbidden dependency.
- Unit-test the extractor with representative allowed and forbidden import syntax independently of the real source tree.

**Non-Goals:**

- Parsing arbitrary Rust semantics beyond crate-local import declarations.
- Changing the dependency graph or adding a production/parser dependency.
- Enforcing dependencies expressed only in tests; the existing production/test split remains.

## Decisions

### 1. Extract dependencies from import declarations before applying the matrix

Add a small test-local scanner that collects complete `use` statements beginning with `use crate::`, removes comments and whitespace that do not affect import structure, and matches every known owner as an import path segment. A segment counts whether it is followed by `::`, `,`, `}`, `;`, or ` as `, which covers direct, grouped, nested, and aliased imports without confusing owner names embedded in longer identifiers. Preserve the existing detection of fully qualified `crate::<owner>::...` references and combine both sources into the dependency set so closing the alias bypass does not open a non-import bypass.

The matrix test will compare the extracted owner set with the allowed set rather than searching the whole source body.

Alternative considered: add `syn` as a dev dependency. Rejected because this narrow regression can be handled deterministically without expanding dependencies or lockfiles.

Alternative considered: add more `contains` patterns at the call-site. Rejected because a growing list of spellings would repeat the same omission-prone design.

### 2. Test the extractor as data, then retain source-tree integration coverage

Focused tests will feed direct, grouped, nested, aliased, multiline, and similarly-prefixed non-owner identifiers to the extractor. The existing integration test will continue applying the result to every production owner and the ADR matrix.

Alternative considered: temporarily mutate source files during the test. Rejected because it is slower, risks concurrent test interference, and is unnecessary when extraction and policy checking can be tested independently.

## Risks / Trade-offs

- [A lightweight scanner could mis-handle unusual comments or strings] → Restrict scanning to `use crate::` statements, normalize comments, and cover multiline/alias cases explicitly.
- [Owner names could match prefixes of unrelated identifiers] → Require Rust import-segment boundaries around every matched owner.
- [Future import syntax could exceed the scanner] → Keep extraction isolated behind focused tests so extending syntax support does not change the matrix policy.

## Migration Plan

No runtime migration is needed. Add the failing regression, implement the extractor, run architecture and strict repository checks, verify and archive OpenSpec, then commit and push to PR #94. Rollback is the single follow-up commit.

## Open Questions

None.
