## Why
PR152 exact commit1bc6bd6f fails Ubuntu unit test masked-hero-reveal.test.ts:24 because complete independent scalar oracle verification exceeds the existing5000ms deadline. Local verification passed but CI exposes repeated Gaussian evaluation cost.

## What Changes
Add test-only memoization around pure gaussian/glow raster functions, keyed by the complete numeric input raster; cached results are immutable tuples and every caller receives a fresh mutable copy. Preserve all scalar function bodies, full16RGBA/all8counterfactual/SHA/witness/stereoPCM verification, refusal controls, default5s deadline and native120s deadlines. No production or contract/reference bytes change.

## Capabilities
### Modified Capabilities
- masked-hero-reveal: efficient complete independent reference verification.

## Impact
Only scripts/create-masked-hero-reveal-references.py executable source plus own specification artifacts. No dependencies, production surfaces, old archives or tests weakened.
