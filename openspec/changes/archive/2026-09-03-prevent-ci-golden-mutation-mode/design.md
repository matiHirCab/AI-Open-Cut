## Context

The required render-parity job is configured correctly, but its structural validator checks only the presence and values of approved step-level environment keys. GitHub Actions also supports workflow- and job-level environment inheritance, and the native harness treats `OPENCUT_UPDATE_GOLDENS=1` as a request to replace the reviewed generation rather than compare against it. `OPENCUT_CAPTURE_GOLDENS_TO` similarly changes the verification workflow by producing an alternate capture.

This follow-up hardens policy validation without changing the valid workflow, native harness, fixtures, renderer, public contracts, or persisted state.

## Goals / Non-Goals

**Goals:**

- Guarantee that required CI invokes native golden conformance in verification-only mode.
- Reject mutation or alternate-capture flags from every effective GitHub Actions environment scope.
- Require exact environment maps for the native conformance and strict report-validation steps.
- Permit unrelated global CI variables that do not affect the golden harness.

**Non-Goals:**

- Remove or rename deliberate local golden update/capture workflows.
- Change workflow commands, job identities, renderer behavior, fixtures, golden thresholds, or repository settings.
- Generalize the policy checker into a complete environment or GitHub Actions interpreter.

## Decisions

### Combine an exact critical-step allowlist with inherited-mode rejection

Require the native step environment to contain exactly the five reviewed keys and the report-validation step to contain exactly its report path. Separately inspect `workflow.env` and `render-parity.env` for `OPENCUT_UPDATE_GOLDENS` and `OPENCUT_CAPTURE_GOLDENS_TO`, because those scopes are inherited even when the step map is exact.

Rejecting every workflow- or job-level variable was rejected because unrelated CI configuration does not weaken golden verification. Checking only the two forbidden names at the step level was rejected because inheritance would bypass that boundary.

### Treat both public mode-changing variables as forbidden in required CI

Forbid both update and alternate-capture variables regardless of value at inherited scopes. Their mere declaration creates ambiguity through expressions or string conversion; required CI has no valid reason to expose them. Local maintainer commands remain unchanged.

Allowing explicit zero values was rejected because later expression or parsing changes could turn a seemingly disabled declaration into active behavior, while absence is unambiguous.

## Risks / Trade-offs

- **New public golden mode variables could be added later.** → Update the validator and specs in the same change that introduces any new harness mode.
- **Exact step maps make intentional CI observation changes require policy updates.** → Keep those changes reviewed and synchronized with focused negative tests.
- **Static validation does not execute GitHub inheritance.** → Test mutations at workflow, job, and step scopes against the same parsed hierarchy GitHub uses.

## Migration Plan

Land the policy checker, tests, documentation, and specification updates together. The checked-in workflow requires no migration. Rollback reverts those policy artifacts without touching application data or golden fixtures.

## Open Questions

None.
