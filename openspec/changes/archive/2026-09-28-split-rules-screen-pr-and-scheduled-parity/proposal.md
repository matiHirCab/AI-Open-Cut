## Why

The required 1920×1080 rules-screen shard performs 25 native renders and exceeded its 180-minute CI timeout on PR #127 after completing 23 renders without a comparison failure. Its cost makes the protected PR gate unable to finish within the repository's 120-minute default budget; the full evidence should remain automated while PRs receive a bounded, meaningful 1920×1080 check.

## What Changes

- **BREAKING CI evidence policy:** Keep all three resolutions in the required PR matrix, but run the full 25-render lifecycle at 960×540 and 1280×720 and a six-render 1920×1080 sample covering original and edited states, frame preview at 500 ms, audiovisual range preview, and export. Check semantic lifecycle state for undo, redo, and reopen without their expensive 1920×1080 renders.
- Run the complete existing 1920×1080, five-state, 25-render conformance weekly on the default branch and on manual dispatch, with a bounded timeout, required native dependencies, unchanged references and tolerances, and a visible failing Actions status on any failure or omitted test.
- Guard both workflows and their exact test selection in repository policy tests. Return the protected PR workflow to the default 120-minute duration budget and remove its temporary 180-minute exception.
- Update the CI guidance and OpenSpec requirements to distinguish the required PR evidence from the scheduled full evidence.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: Split required PR and scheduled 1920×1080 coverage while preserving the existing fixture, full lifecycle assertions, immutable references, and tolerances.
- `repository-validation`: Require and validate the bounded PR matrix and the separate weekly full-resolution workflow, including duration limits and failure visibility.

## Impact

Affected files are the editor-core rules-screen conformance test, the protected and new scheduled GitHub Actions workflows, CI policy validator and tests, and CI documentation. The six-render PR check reduces immediate 1920×1080 evidence; the scheduled workflow retains the complete test with an expected detection delay of up to one week. Scheduled GitHub workflows run on the default branch, so the full suite will first run after this change is merged. There are no public or persisted contract, schema, migration, rendering-semantics, golden-reference, or dependency changes. This does not remove 1920×1080 PR coverage or relax its comparisons.
