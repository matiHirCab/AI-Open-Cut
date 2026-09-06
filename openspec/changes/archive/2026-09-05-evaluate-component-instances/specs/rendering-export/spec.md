## ADDED Requirements

### Requirement: Shared component audiovisual rendering
Frame, range, draft preview and export MUST consume one canonical expanded scene, including mapped media source time, local animation, fades, internal transitions and audio. Audio source playback MUST use the product of instance rates with pitch-preserving tempo adjustment and existing track mute/role/gain and global mixing/ducking semantics. Voiceover activity MUST be mapped before ducking. Instance item/track hidden MUST suppress its entire occurrence including audio; visual group parenting MUST retain its existing audio independence. Supported cumulative media rates SHALL be [2^-32,2^32] and tempo preparation MUST use at most 32 factors in [0.5,2]; unsupported or non-finite rates MUST fail with INVALID_ARGUMENT before writes, without fallback to another speed.

#### Scenario: Compare preview and export
- **WHEN** a nested audiovisual fixture with trim, non-unit rate, local animation, transitions, slot overrides and repeated assets is rendered through every intent
- **THEN** corresponding frames and sound agree within the existing canonical visual/audio tolerance and repeated renders are deterministic

#### Scenario: Preserve publication and path safety
- **WHEN** a nested managed resource is missing or unsafe, timing is unsupported, or backend preparation/execution fails
- **THEN** existing typed errors and atomic publication rules apply, with no partial final artifact, persisted-state mutation, raw expression, arbitrary path or network fallback

#### Scenario: Apply occurrence visibility to sound
- **WHEN** an instance is hidden, a local audio track is muted, or only an ordinary visual group ancestor is hidden
- **THEN** instance-hidden content emits no sound, muted tracks retain their mute semantics, and ordinary visual group hiding does not independently mute media audio
