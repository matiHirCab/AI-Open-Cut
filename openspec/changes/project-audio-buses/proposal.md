## Why

Issue #65 in epic #8 needs canonical voiceover, music, sound-effects and master routing before designed events and bus DSP can share one persisted model. Dependency #17 is closed; issue #62 is verified at d8168ad6631d834891df9303485e1d8c1b9b1116 with all eleven checks passing in CI37706235074.

## What Changes

- Introduce schema39 with exactly four built-in project buses, stable IDs voiceover/music/sfx/master, and bounded acyclic output routes terminating at master. Defaults route each stem directly to master; master is terminal.
- Add optional explicit audioBusId on root/component audio-bearing tracks; omission maps existing audioRole to its matching bus, with unassigned mapping to master. Explicit routing does not replace role-based ducking.
- Add typed standalone/batch audio_bus_set_route and scoped audio_track_route edits with existing alias, revision, lock, rollback and history semantics. Existing project reads expose the model; no additional inspection API is required here.
- Atomically migrate current and every retained undo/redo snapshot, preserving absence of explicit track routing and every existing value/resource. Reject premature introduced fields, malformed current records and unknown future schemas before publication.
- Add a manually governed audio-buses-v1 catalog and precise schema39/additive-public-surface predecessor proofs. Protocol1 and existing operations remain compatible; the persisted schema advances with explicit deterministic migration.

## Capabilities

### New Capabilities
- `project-audio-buses`: Bounded built-in bus routing, role fallback, typed transactional routing edits and unchanged rendered behavior.

### Modified Capabilities
- `project-persistence`: Atomic schema39 adoption across current/retained generations, strict introduced-field guards and recovery.

## Impact

Canonical editor-core model, validation, migrations and timeline mutation owners; thin headless/MCP transports; canonical contract/ownership/CODEOWNERS, parity and native/migration/integration consumers; documentation. No provider behavior, dependencies, global settings or unrelated work changes. New project fields are a versioned persisted addition, not an unversioned narrowing of existing requests. Frozen predecessor fixtures and negative controls remain intact; only exact approved deltas are projected away for historical evidence.

## Non-goals

Custom bus creation/deletion, arbitrary graphs or IDs, gain/pan/EQ/compression (#66), explicit side-chain ducking (#67), event definitions/placement (#63/#64), metering/normalization (#68/#69), full inspection (#70), raw FFmpeg expressions and renderer output changes are outside this issue. No merge, deployment or issue closure; the draft PR targets main and remains cumulative with #62 until PR157 is merged.
