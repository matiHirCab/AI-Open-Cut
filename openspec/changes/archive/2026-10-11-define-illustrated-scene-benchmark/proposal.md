## Why

Issue #181 needs a small reproducible starting point for illustrated compositions before #182 adds a reusable hero card. Current native shapes, text, components and managed artwork already exist, but their illustrated workload and measured reference envelope are unspecified.

## What Changes

- Define one original/synthetic hero-card benchmark with named native layers, supported inputs, explicit complexity caps and independent layout/pixel expectations.
- Describe three bounded final composition targets without implementing the pack or future feature APIs.
- Exercise an actual native frame and short range at 1920×1080, record exact source/runtime identity and fresh/repeated request observations, then set reference-environment performance targets from those observations.
- Add deterministic fixture coverage to the existing native golden conformance entry point without replacing existing evidence or modifying protected workflows.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: bounded illustrated benchmark inventory, independent evidence and attributable measurements.

## Impact

Test-only editor-core fixture/verification code and illustrated benchmark documentation. Existing public/persisted contracts, schema, error codes, renderer behavior and dependency graph remain unchanged; no breaking changes or migration. No dependency on unmerged #85. #23, #28–#31, #36 and #46 are closed on main `431a5e38bba964d780cfdbe0fe4ddf287ea21c58`.

## Non-goals

No built-in pack, graphic palette slot API, desktop controls, renderer optimization, raised resource limits, precomposition, external art services, private projects or progress/startup diagnosis. #182–#191 own those later slices; #177–#179 and #85 stay separate.
