## Why

The required Linux golden job passes a workspace-relative report path to a Cargo test whose runtime working directory is the crate, so the report write fails before schema validation and artifact upload. CI must name one unambiguous workspace artifact path and ensure its parent exists.

## What Changes

- Resolve the Linux golden report path from the GitHub Actions workspace rather than the test process working directory.
- Create the report's workspace `target` directory before native conformance.
- Keep schema validation and artifact upload pointed at the same workspace file.
- Add verification that reproduces the workflow command and validates the schema-2 report.

Non-goals: changing the Rust report writer, environment-variable semantics outside CI, fixture media, `CURRENT`, generation digests, schemas, tolerances, renderer behavior, or public contracts.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: Require Linux CI to use a workspace-anchored report path shared by capture, validation, and artifact upload.

## Impact

The change affects only the Linux packaged-smoke workflow and the render-regression-fixtures specification. It adds no dependency, API, contract, migration, fixture, or runtime behavior change.
