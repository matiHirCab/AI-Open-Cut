## Why
Review of issue #32 reproduced frozen position, scale and opacity for ungrouped styled root text. The measurement-only identity ancestor never reaches rendering.
## What Changes
Persist identity ancestry in evaluated layers, remove the measurement fallback, and prove actual animation with independent static snapshots and native render parity.
## Capabilities
### Modified Capabilities
- rendering-export: preserve styled root text animation across all render intents.
## Impact
Core evaluator and regression tests only. No public contract, schema, migration or dependency edge changes. Preserve the previous archived change.
## Approval
The user explicitly requested implementation of this exact plan on 2026-09-09. These artifacts transcribe that approved plan without expanding scope.
