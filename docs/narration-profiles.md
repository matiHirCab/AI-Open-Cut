# Versioned narration profiles

Issue #207 implementation in progress; not release acceptance evidence.

Profiles are local to the configured projects root and shared between its projects. `narration_profile_create`, `narration_profile_get`, `narration_profile_list`, `narration_profile_update` and `narration_profile_apply` expose the core-owned library. Update requires `expectedVersion`; apply accepts `application` with `profileId`, `expectedVersion`, text and optional language/voiceId/speed/textOptions. Application checks provider readiness and available voices without synthesis or a project edit.

Speech request sources, insertion and regeneration accept `profile: {id, version}`. Explicit overrides replace matching defaults; textOptions replaces the whole options object, including an empty pronunciation list. Each generated request stores exact effective settings and `narrationProfile: {id, version, name}`. Profile updates affect future explicit applications only. Retained preview retry and ordinary regeneration do not refresh the library version.

The bounded version-1 JSON library uses the root lock and durable atomic publication. Maximum 100 profiles, 80 Unicode scalars in trimmed unique case-sensitive names, no name controls, 256 scalars in language/voice identifiers, 100 ordered pronunciation entries, 128/256 scalars in term/spoken and a 5000ms pause. Duplicate trimmed terms fail; overlapping literal terms execute in supplied order. Basic normalization retains existing NFC and whitespace behavior before replacements; profiles introduce no additional normalization or phoneme inference. Profile storage does not invoke providers; unsupported voices fail on explicit apply/generate with existing readiness/validation errors.

Schema46 adds optional attribution. Existing current/history snapshots migrate under project locks and retain exact old speech settings without invented metadata. Reopen, rendering and undo/redo are independent of library/provider availability. Do not downgrade schema46 with an older binary. No cloud sync, new model/provider, inference credits, merge or deployment.

Verification remains pending until all tasks and protected published-head checks pass.
