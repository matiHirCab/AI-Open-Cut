---
name: opencut-render-regression
description: Investigate or change OpenCut rendering with canonical scene rules and deterministic regression evidence.
---

# OpenCut render regression

Use for preview, export, scene-evaluation, render-plan, or visual/audio parity work. Locate the affected requirements in `openspec/specs/rendering-export/`, `openspec/specs/render-regression-fixtures/`, and any feature-specific spec before selecting tests. Check `docs/adr/0003-editor-core-module-boundaries.md` and `docs/adr/0004-motion-graphics-architecture.md` for the owning editor-core module and scene handoff.

Before changing implementation, use an approved OpenSpec change whose proposal, delta spec, design, and tasks cover the desired output, failure behavior, compatibility, fixture impact, and verification. If coverage is missing, update the artifacts and obtain explicit approval first. Keep rendering semantics in `editor-core`; do not add parallel rules in transports or UI.

Use the reviewed fixture and commands described in `docs/ci-parity-gates.md`. Distinguish a fixture baseline update from a renderer fix: baseline changes need deliberate review and provenance. Run affected core tests and the required render-parity, formatting, lint, workspace, integration, and smoke checks from `AGENTS.md` and the approved tasks. Preserve exact failure output in an uncommitted log and use `$openspec-verify-change` before syncing and archiving.
