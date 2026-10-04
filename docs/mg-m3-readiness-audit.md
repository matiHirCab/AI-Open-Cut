# MG-M3 readiness audit: pending

This is a pre-audit limitation record, not an assertion that the full audit has begun, passed or been accepted. The inspected base is main `ad5cd0ca0514c7ed92d4f3dae9cf9444ef3ff9de` (completed issues38–47) plus the current issue48 branch. Issue48 final checks and exact published-head CI must finish before the independent Sol-medium readiness audit begins; that audit is required before MG-M4 work.

## Known remaining timestamp limitations

The issue48 nonaligned requested-origin correction reuses canonical CPU sampling for supported animated Rectangle/Shape/Media and existing shaped/PAM-capable Text, plus static explicit Transform2D Rectangle activity. Caption and unshaped legacy Text remain excluded: their existing nonaligned expression-time limitation is unresolved. Their prior paths/error behavior are preserved, without a new unsupported-source or missing-binding error. This is not a universal timestamp-fidelity claim.

The [temporal fixture guide](temporal-animation-fixtures.md) describes the corrected scope, independent oracles and unchanged tolerances. Issue48's active conformance record identifies its actual focused/native/GUI evidence and pending final gates. Passing issue48 fixtures does not by itself establish full motion-graphics readiness.

## Required independent audit

After issue48 exact-head CI passes, an independent Sol-medium audit must inspect the actual issues38–48 stack: core/render/desktop behavior, public contracts and ownership, retained history, presets/provenance, schema migrations and the complete mandatory gate evidence. It must inspect actual integrated inputs and outputs rather than assume readiness from issue closure or prior branch results.

Verified gaps require bounded, explicitly approved fix specifications and reviewable PRs or an ordered stack, with relevant verification and exact-head CI. This record authorizes no merge/deployment, assumes no future issue closure, and does not replace the required independent audit or its eventual decision.

## Status

- Full audit: pending after issue48 exact-head verification.
- Caption/unshaped-Text nonaligned timestamp limitations: tracked, unresolved.
- MG-M4 readiness: not certified by this record.
