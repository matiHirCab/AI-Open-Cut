## Why

Issue #63 in epic #8 needs a canonical named sound library before #64 can place deterministic semantic audio events. Dependencies #11 and #17 are closed. Issue65 is verified at321dc3583df2d95380efb613153b0135bb32268d with all11 checks passing in CI37721029887, and this branch starts exactly there.

## What Changes

- Introduce schema40 with a required project-level soundDefinitions collection of bounded closed named records: ordered existing content-addressed variant asset IDs, default finite dB gain, a built-in routing bus and a saved JavaScript-safe deterministic variant seed. New/migrated projects receive an empty registry.
- Add typed sound_event_register as a protocol1/MCP standalone and batch/draft operation. Registration creates or atomically replaces one named definition in stable order, returns its named identity for existing resultAlias resolution, and preserves revision, rollback, history and resource semantics.
- Resolve variants through existing canonical managed-asset metadata/integrity, and retain registered/current/history/draft asset references through the existing ownership and deletion/collection policy. No arbitrary path, URL or raw backend expression is accepted.
- Add a pure deterministic selector using saved/explicit safe seeds and ordered variants. Registration/selection metadata alone leaves evaluated plans, pixels, decoded audio, roles and ducking unchanged; timeline placement/render activation belongs to #64.
- Atomically migrate all supported current and retained generations, reject premature/nullable/malformed registry envelopes and unknown future schemas, and prevent failed registrations from publishing incidental legacy adoption.
- Manually synchronize canonical fixtures/ownership/CODEOWNERS, capability/schema reporting and complete Rust/TypeScript/MCP/native consumers with precise independent issue65 predecessor proofs. The seven existing active current-schema catalog headers advance exactly39→40; frozen feature/historical catalogs, old assertions, commands and unrelated drift negatives remain preserved.

## Capabilities

### New Capabilities
- `semantic-sound-event-definitions`: Bounded named project library, deterministic selection, transactional registration and governed additive contracts without render activation.

### Modified Capabilities
- `project-persistence`: Atomic schema40 empty-registry adoption and structural/current/history guards.
- `media-assets`: Sound-definition and pending-registration asset roots, deletion guards and integrity classifications under the existing asset owner.

## Impact

Existing editor-core model, validation, timeline, assets, migrations and store owners; thin headless/MCP transports; versioned manual catalogs/parity/policy/native/integration fixtures; documentation. The public operation/capability are additive protocol1; schema40 is an explicit persisted-version migration. No new dependency edge, transport domain validator, provider, DSP, timeline event item, rendering behavior, custom bus lifecycle, deletion API, unrelated refactor or weakened check is introduced.

The future draft targets main and includes issues62+65+63 while PR157/158 remain unmerged, with required merge order157→158→successor. No merge, deployment, GitHub issue closure or PR archival is authorized. Implementation conformance, required standard acceptance, specification synchronization/archive, protected validation and all11 exact-head CI are hard delivery gates before the next issue.
