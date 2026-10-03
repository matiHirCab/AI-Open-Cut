## MODIFIED Requirements

### Requirement: Deterministic item-local loop evaluation
Editor-core MUST anchor loop phase to absolute source time derived from the same composition time in every render intent: item-local time plus the optional retained source-clock offset exactly once, independent of the requested preview range start. Omitted clocks MUST retain absolute item-local phase. A retained clock MUST preserve the original first/last source keys, phase and finite iteration budget through editing; negative source time MUST hold the first source value. The loop interval MUST start at the first keyframe and use the strictly positive first-to-last timestamp span. Before the first keyframe, sampling MUST hold its value. Forward repeat MUST sample each half-open `[first,last)` cycle from the start value at exact seams; a finite count denotes that many forward cycles and then holds the terminal value. Ping-pong MUST use the existing forward curve sampler during the outward traversal and reflect source time through the same curve during the return; one finite iteration denotes one complete outward-and-return round trip. At the turn it MUST sample the last value, at a round-trip seam the first value, and after finite exhaustion it MUST hold the first value. Infinite variants MUST continue until the item's half-open end. Persisted keyframe offsets MUST remain integer milliseconds. Visual time derived through component, stagger or repeater affine clocks MUST retain fractional milliseconds through phase mapping and interpolation without nearest-millisecond rounding. Integer sampling MUST retain existing results. Phase arithmetic MUST be checked and finite, constant-work, and independent of the number of elapsed cycles; evaluated scalar values MUST remain finite and within existing canonical property bounds.

#### Scenario: Sample exact forward seams and exhaustion
- **WHEN** finite or infinite repeat is sampled at the first/last keyframe, immediately around a seam, at a later exact seam, or after finite exhaustion
- **THEN** the documented half-open phase selects the same value at every equivalent cycle point, with no value jump and no preview-range reset

#### Scenario: Sample ping-pong turn and return
- **WHEN** a ping-pong Bézier or spring channel is sampled on both sides of a turn, at the turn, at a round-trip seam, or after finite exhaustion
- **THEN** the reverse traversal reflects time through the canonical forward curve, the exact endpoint values are selected, and output remains finite and within property bounds

#### Scenario: Preserve independent properties
- **WHEN** visual channels or audio gain use different loop records or one channel is unlooped
- **THEN** each property samples only its own phase and absent channels retain their static fallback

#### Scenario: Preserve fractional repeat phase
- **WHEN** a visual repeat channel with linear keys (0 ms, 0), (99 ms, 100), (100 ms, 0) is sampled through timeScale 0.995 at root 100 ms
- **THEN** mapped local time 99.5 ms samples value 50 rather than wrapping early to zero, with identical expected output across render intents

#### Scenario: Preserve fractional turns and finite completion
- **WHEN** fractional mapped visual times fall immediately before, at or after ping-pong turns, repeat seams or finite exhaustion, including nonzero first-keyframe offsets
- **THEN** phase reflects or completes only at the exact boundary and intermediate samples use their fractional time with the existing held endpoints and curves

#### Scenario: Preserve existing integer and independent audio sampling
- **WHEN** existing integer-clock visual samples or independent audio loop samples are evaluated
- **THEN** their established values, curves, bounds, audio behavior and error contracts remain unchanged

#### Scenario: Retain midcycle phase and finite exhaustion through edits
- **WHEN** split-right or left-trimmed repeat or ping-pong channels retain a source offset within a cycle and are evaluated at integer or fractional interiors, seams, turns or finite exhaustion
- **THEN** they sample the original source phase and original finite budget, without restarting cycles or depending on the requested output window
