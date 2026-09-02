## Context

The Linux packaged-smoke job supplies `target/render-baseline-linux.json` to a Cargo unit test. Cargo runs that test with the crate as its working directory, while later workflow steps validate and upload `target/render-baseline-linux.json` from the repository workspace. The test therefore fails before producing the artifact.

## Goals / Non-Goals

**Goals:**

- Give capture, schema validation, and upload one unambiguous workspace artifact.
- Ensure the destination parent exists before the native test starts.
- Preserve the report writer's existing caller-relative path semantics outside CI.

**Non-Goals:**

- Changing Rust code, report schemas, fixtures, digests, rendering, tolerances, or public contracts.
- Recapturing reviewed references or changing artifact retention.

## Decisions

### Anchor the CI environment value to the GitHub workspace

Set `OPENCUT_GOLDEN_REPORT_PATH` to `${{ github.workspace }}/target/render-baseline-linux.json`. Create its parent with `mkdir -p "$(dirname "$OPENCUT_GOLDEN_REPORT_PATH")"` before invoking Cargo. Later validation and upload remain workspace-relative because GitHub Actions runs those steps from the repository workspace.

This is preferred over changing the Rust writer because the defect is a disagreement between workflow working directories, while relative paths are valid caller-controlled behavior elsewhere. It is preferred over changing the Cargo step's working directory because Cargo already receives the intended repository-root invocation and its test-process directory is not a stable artifact contract.

## Risks / Trade-offs

- [GitHub expression path contains spaces] → The shell command quotes the complete environment value and its `dirname` expansion.
- [Validation or upload drifts from capture] → Keep the existing workspace-relative consumers and verify they resolve to the same absolute file.

## Migration Plan

The next Linux CI run creates the report at the corrected workspace path. Rollback restores the former environment value and directory-preparation line; there is no data migration.

## Open Questions

None.
