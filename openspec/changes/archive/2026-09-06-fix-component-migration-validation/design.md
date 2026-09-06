## Context

The migration owner clones current/history, upgrades each version, then publishes the clones. Store loading validates the upgraded documents before transactional persistence. Schema 13 relaxed nested legacy transform validation, so validating only after relabeling can accept content forbidden by schemas 11–12. Source deserialization already enforces other version-specific shape constraints.

## Goals / Non-Goals

**Goals:** Restore source-version rejection before relabeling, preserve atomicity and supported migrations, and demonstrate conformance with regression tests.

**Non-Goals:** No public interfaces, schema bump, rendering changes, broad migration refactor or automatic repair of previously invalid files.

## Decisions

- Add a source-schema constraint guard in the core migration owner and invoke it before changing a supported snapshot's version. For schemas 11–12, inspect every definition-local component instance and reject any legacy transform unequal to Transform::default() with non-retryable INVALID_ARGUMENT. Hidden flags and reachability do not bypass it. Preserve the existing older-schema root-instance guard and unsupported-version errors.
- Use the existing clone-before-mutation traversal for current, undo and redo. A later invalid snapshot must not update the caller's originals, write a migration journal or replace project/history. Retain post-migration validation and asset/reference checks under the existing project lock and recoverable transaction.
- Keep schema 13 exempt from the historical transform restriction. Do not reject valid transform2d with a default legacy transform. Preserve normalization for schemas 1–8; moving all current-schema validation ahead of that normalization could incorrectly reject supported legacy input.
- Prefer a core guard over transport validation or JSON-only checks: it covers typed migration inputs as well as persisted projects without duplicating domain rules in adapters. Do not clamp/reset invalid transforms, because that silently changes source content.

## Risks / Trade-offs

- Over-restricting schema 13 or transform2d -> positive regressions paired with the rejection matrix.
- Rewriting current state before discovering invalid history -> byte-for-byte current/history assertions plus in-memory clone assertions.
- Breaking old normalization -> rerun existing legacy migration and publication-fault tests.
- Files already rewritten to schema 13 cannot be distinguished from legitimately authored schema-13 files -> do not infer provenance or retroactively reject them.

## Migration Plan

No new format or migration step. Ship the guard with schema 13 unchanged. Valid schemas 1–12 continue to migrate; invalid source transforms fail without publication. A binary rollback does not require data conversion but restores the defect. No privacy, external-resource or dependency changes occur. ADR 0003 forbids a migrations-to-validation dependency, so the historical guard stays private to migrations, alongside the existing historical root-instance check.

## Verification

Parameterize schemas 11–12, current/undo/redo placement and non-default x/y position, scale and opacity, including unused/hidden definitions. Verify INVALID_ARGUMENT and unchanged project/history bytes, plus unchanged typed inputs on failure. Positive cases cover default legacy transforms, supported transform2d, schema-13 non-default legacy transforms, mixed history and deterministic reopen. Retain existing root rejection, unknown-version, normalization and crash recovery coverage. Run native FFmpeg tests and all repository-required checks, record results, verify/archive, then run the archive-only Moon gate.

## Open Questions

None. The user approved the concrete artifacts on 2026-09-06.
