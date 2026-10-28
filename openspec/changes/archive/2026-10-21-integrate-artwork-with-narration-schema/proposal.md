## Why

PR #229 and current main independently allocated project schema47 to artwork provenance and required narration manifests. A textual merge would accept conflicting persistence and catalog meanings. The archived artwork approval anticipated a schema collision but its normative requirements still fix schema47, so this integration needs explicit reviewer approval before implementation.

## What Changes

- Reconcile #229 against main 8cb21a6bab2f5a0134ab6af3f91a018f0ae9bde8, retaining both artwork preparation and narration/channel-pack contracts.
- Allocate schema48 to artwork provenance, preserving main's schema47 required narrationManifests and schema1..46 migration semantics.
- **BREAKING persisted version boundary:** old builds reject schema48; migrate supported current/history generations atomically forward. Reject unpublished artwork-branch schema47 documents rather than guessing their identity or dropping fields. No downgrade is offered.
- Compose catalog inverses so the integrated tree can recover exact original artwork and narration predecessor hashes without recapture, relaxed assertions, or lost peer additions.
- Verify both features, existing package46 compatibility, and the full unchanged required gates before publication.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `channel-style-packs`: synchronize the already-approved ordinary capture-in-schema48 requirement with the existing explicit source-version inventory; retain portable component46.
- `project-persistence`: replace Artwork metadata schema migration with the integrated schema48 boundary and add combined feature conformance requirements.

## Impact

Core model/deserialization/migrations/store, native asset initializers and schema assertions; bridge/headless schemas and operations; canonical current schema markers, ownership/catalog unions, exact historical inverse layers and focused tests; operator schema documentation. The original artwork algorithms, review filesystem behavior, narration workflow and package v1 format remain governed by their existing approved specifications.

## Non-goals

No Windows preview-disposal investigation, diagnostics, production deletion changes or manual race reproduction; no PR233, #208 WIP, #228 edits, merge, deploy, issue closure, force push, workflow changes, new provider or paid service. Existing Windows CI failures remain reported blockers. Approval and implementation state are tracked separately in approval.md, tasks.md and verification.md.
