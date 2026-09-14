## Why

Draft replacement still assigns retained fonts by array order when one old action matches several replacements, and changing a component child's selector resets untouched siblings. Both violate stable font intent.

## What Changes

- Analyze complete one-to-one matching alternatives, including retained versus newly unresolved outcomes.
- Retain component fonts by scoped local text identity and unchanged selectors.
- Reject draft-v2 selector collisions requiring different complete bindings before publication.
- Add regression and lifecycle evidence for these scenarios.
- Explicitly resolve changed component selectors and reintroduced local IDs, rejecting reversions when inherited bindings make draft-v2 replay disagree with current resolution.

## Capabilities

The approved preceding-bindings extension replaces unresolved-only simulation with chronological preparation and globally optimal matching alternatives. It fixes false ambiguity for equivalent inherited fonts and missed ambiguity between explicit resolution and inheritance after an earlier draft action supplies a font.

### New Capabilities

None.

### Modified Capabilities

- `font-resolution`: complete ambiguity detection, local component retention and representable draft steps.

## Impact

Editor-core assets/store preparation, tests and text-layout documentation. No public fields, dependencies, migrations or format changes: schema 19, draft 2 and opencut-text-v2 remain. Non-goals: Unicode changes and unrelated repository work. The user's explicit implementation request approves the supplied plan, including atomic rejection of unrepresentable steps.
