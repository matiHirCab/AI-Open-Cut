## Why

Issue211 needs reusable channel styling without repeatedly reconstructing native scenes, fonts, palette/caption defaults and optional narration settings. Its sole hard dependency207 is closed; existing ordinary components, typed slots and content-addressed fonts can provide a bounded independent slice without touching208 spoken-text audition or its unpublished UTF-16 work.

## What Changes

- Add a bounded version1 core-owned library of captured channel packs. Each versioned manifest contains ordinary native component recipes, exact managed font faces, explicit managed-artwork dependency declarations, existing typed slot defaults, an optional caption-recipe role and optional narration-profile reference.
- Capture selected existing source components and their bounded dependency closure without changing source project/history/media. Export and import portable data packages; create/update use the existing root lock, atomic publication and optimistic versions.
- Explicitly instantiate into independent project-owned ordinary definitions/instances with fresh identities, managed pinned fonts and caller-bound matching artwork. Explicit reapply uses the current project revision, preserves instance overrides and reports concrete definition changes; later library edits never propagate to projects.
- Make missing/incorrect fonts, artwork and optional profiles actionable. Font bytes must match pinned identities; artwork binds only explicit existing managed assets; profile choices use existing207 data. No network lookup, ambient replacement or inference occurs in pack operations.
- Expose six additive headless operations and a source CLI with capture/update/list/get/import/export/apply commands. Existing MCP registrations, provider/text-audition files and installed GUI flows remain unchanged.
- Reject executable/unknown package fields, arbitrary filesystem references, future formats, malformed native content and excess resources. Reuse existing core validation, font ingestion, storage/resource rollback, revisions/history, aliases and native rendering rather than introducing a template engine.
- Govern the new fixture and all consumers, preserve existing raw predecessors/pins/negative controls, verify real two-project source/packaged native workflows and complete the OpenSpec/required-check/exact-head CI lifecycle before delivery. MontiPKM is an example workflow name with configurable original data, not an approved brand design.

## Capabilities

### New Capabilities

- `channel-style-packs`: bounded portable captured styling, explicit independent instantiation/reapply, managed dependencies and immutable existing project behavior.

### Modified Capabilities

None. Existing native component, slot, font, narration, rendering and persistence semantics remain authoritative; the new capability composes them.

## Impact

New core model/store modules and a narrowly shared existing staged-transaction helper; thin headless dispatch, typed native request union and isolated bridge CLI modules; canonical channel-pack fixture, additive headless catalog entries and exact test-only projections/consumers; documentation and independent verification. The already-locked base64 dependency may be made direct for core font-data encoding. No project schema/version, renderer, provider protocol, error catalog, security configuration, credentials, deployment, merge or208 device/WIP access. Preserve191 illustrated-release ownership and213/214 readiness limits. Main/PRs/dependencies/comments must be refreshed before implementation; independently merged215 requires an approved compatible governance union rather than historical repinning.
