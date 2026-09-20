## Why

Independent review of issue #35 reproduced three conformance defects: paint margins displace logical layout boxes, retained drafts bypass layout validation on reopen/migration, and fractional tracking changes exact-width wrapping and fitting. Existing work-limit tests also do not establish inclusive accounting or shared expanded-scene preflight behavior.

## What Changes

- Preserve fractional logical box geometry and compose raster offsets consistently through anchors, transforms, animation, components and repeaters.
- Validate retained draft text styles through the canonical validator before authoritative publication, including nested component text, without replaying stale drafts.
- Use consistent advanced-layout width accumulation for wrapping, diagnostics and exact integer fitting.
- Add regression and boundary coverage, including private test-budget injection while retaining the production candidate-glyph ceiling of 16,777,216.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `advanced-text-layout`: Explicit logical-box placement, fractional-width consistency, retained-draft validation and inclusive shared work-accounting conformance.
- `rendering-export`: Shared corrected geometry and preflight behavior across frame, range, draft and export.

## Impact

Changes remain inside editor-core geometry, shaping, validation and render preflight, with Rust and bridge regression tests and supporting documentation. Existing public fields, operations, capabilities, schema version 21 and encoding settings remain unchanged. These are corrective compatibility changes: valid legacy requests and persisted documents retain their behavior; invalid retained layouts fail with the existing INVALID_ARGUMENT contract before publication. No new dependency edge or public contract version is introduced.

## Non-goals

No new layout controls, epsilon comparisons, numerical limits, newline-work rejection rule, runtime budget configuration, encoding changes or stale-draft replay. Preserve layout-absent arithmetic and rendering exactly. Preserve the original `2026-09-19-advanced-text-layout-auto-fit` archive and unrelated working-tree changes.

## Approval

The user explicitly approved this proposal, design, delta specifications and tasks with "Yes" on 2026-09-19. Implementation is authorized through this task list.
