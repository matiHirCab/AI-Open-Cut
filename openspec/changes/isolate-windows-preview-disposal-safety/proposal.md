## Why

Windows native packaged acceptance on main `8cb21a6bab2f5a0134ab6af3f91a018f0ae9bde8` fails because an outside sentinel disappears during preview disposal. The fixture itself recursively removes a junction pointing at its temporary root; existing evidence cannot distinguish native deletion from fixture/runtime cleanup. PR #229 records an earlier intermittent occurrence, so attribution to PR #232 is unsupported.

## What Changes

- Add bounded diagnostic controls separating junction setup/teardown without native disposal, ordinary native disposal without replacement, and the actual source/package replacement race.
- Assert sentinel preservation before racing, after fixture link removal, after native invocations and after restoration; retain the original twelve invocations and all existing safety, close/debt and foreign-file assertions.
- Require evidence that a replacement actually occurred; distinguish Windows sharing refusal from an exercised replacement.
- If Windows evidence identifies fixture link cleanup as the cause, replace only that cleanup with a non-traversing link removal and cover target preservation. A native disposer defect requires an updated concrete design and approval before changing production behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `artifact-resources`: add explicit acceptance evidence requirements for preview-disposal race safety, without weakening existing L4–L10 invariants.

## Impact

Private agent-bridge tests and fixture helpers; native disposal code is an investigation target, not approved implementation scope. No public/persisted contracts, schema, catalog, revision semantics, package roles, dependencies or migrations change. Required native Windows evidence remains distinct from Linux verification.

## Non-goals

No skipped tests, increased timeouts, retry-until-green, runtime upgrades, CI/security/secret changes, production deletion refactor without evidence, #208 WIP, #228/#229 modifications, merges or deployments.

## Approval status

Explicit delegated reviewer approval granted on2026-10-21 for review diff SHA2562d46814469057528303e89cb32695e044f7e38f7f1d3614ae2e487db49a0747e, parent thread01a1583b-f8d9-70ae-8c32-75210bced0e5. Diagnostic publication may precede Windows verification after local checks; label investigation/pending and keep this active specification unarchived. No production disposer change approved.
