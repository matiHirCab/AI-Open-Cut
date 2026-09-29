# animation-loops Specification

## Purpose

Deterministic finite and infinite animation loop behavior for active typed channels.

## Requirements

### Requirement: Closed bounded channel loop input
An active typed animation channel MAY carry an optional closed `loop` record with `mode` equal to `repeat` or `ping_pong` and `iterations` equal to an integer in [1, 10000] or the literal `infinite`. Editor-core MUST reject loops with fewer than two keyframes, a nonpositive first-to-last keyframe span, unknown fields or variants, noninteger or out-of-range iterations, unsupported channels, or values that violate the channel's existing bounds. `repeat` MUST require exactly equal typed first and last values so each seam is value-continuous. Loop input MUST not admit arbitrary expressions, executable SVG, paths, or network resources. Absence of `loop` MUST preserve the existing channel behavior and serialized compatibility.

#### Scenario: Accept bounded active loops
- **WHEN** a compatible visual or audio channel has two or more valid keyframes and a valid finite or infinite repeat or ping-pong record
- **THEN** core stores the exact typed record and existing curves and property bounds remain in force

#### Scenario: Reject invalid loops without a commit
- **WHEN** a loop has one keyframe, invalid span/count/shape, an unavailable property, or mismatched repeat endpoints
- **THEN** core returns non-retryable `INVALID_ARGUMENT` and publishes no state, revision, history, alias, or artifact change

#### Scenario: Preserve unlooped clients
- **WHEN** a caller omits `loop` or uses legacy `set_keyframes`
- **THEN** wire representation, sampling, and established error behavior remain unchanged

### Requirement: Deterministic item-local loop evaluation
Editor-core MUST anchor loop phase to absolute item-local integer milliseconds derived from the same composition time in every render intent, independent of the requested preview range start. The loop interval MUST start at the first keyframe and use the strictly positive first-to-last timestamp span. Before the first keyframe, sampling MUST hold its value. Forward repeat MUST sample each half-open `[first,last)` cycle from the start value at exact seams; a finite count denotes that many forward cycles and then holds the terminal value. Ping-pong MUST use the existing forward curve sampler during the outward traversal and reflect item-local time through the same curve during the return; one finite iteration denotes one complete outward-and-return round trip. At the turn it MUST sample the last value, at a round-trip seam the first value, and after finite exhaustion it MUST hold the first value. Infinite variants MUST continue until the item's half-open end. Integer phase arithmetic MUST be checked, constant-work, and independent of the number of elapsed cycles; evaluated scalar values MUST remain finite and within existing canonical property bounds.

#### Scenario: Sample exact forward seams and exhaustion
- **WHEN** finite or infinite repeat is sampled at the first/last keyframe, immediately around a seam, at a later exact seam, or after finite exhaustion
- **THEN** the documented half-open phase selects the same value at every equivalent cycle point, with no value jump and no preview-range reset

#### Scenario: Sample ping-pong turn and return
- **WHEN** a ping-pong Bézier or spring channel is sampled on both sides of a turn, at the turn, at a round-trip seam, or after finite exhaustion
- **THEN** the reverse traversal reflects time through the canonical forward curve, the exact endpoint values are selected, and output remains finite and within property bounds

#### Scenario: Preserve independent properties
- **WHEN** visual channels or audio gain use different loop records or one channel is unlooped
- **THEN** each property samples only its own phase and absent channels retain their static fallback

### Requirement: Transactional loop edits and durable history
The existing `set_animation_channels` standalone and `timeline_batch_edit` operations MUST accept loop records through the same typed replace-channels input. They MUST retain expected-revision, locked-track, missing-item, ordered batch alias, changed-ID, atomic rollback, undo/redo, and deterministic reopen behavior. Invalid loops MUST return `INVALID_ARGUMENT`; missing references and stale revisions MUST retain their established stable code and retryability. Successful edits MUST commit one revision and history step per standalone edit or batch.

#### Scenario: Loop a newly created item by alias
- **WHEN** a batch creates a compatible item and a later edit addresses its alias with valid looped channels
- **THEN** one atomic commit stores the resolved item ID and undo/redo and reopen restore the exact loop record

#### Scenario: Roll back failed loop work
- **WHEN** a loop edit fails after earlier valid batch work, or an edit targets a missing/locked item or stale revision
- **THEN** the established typed error is returned and project, history, revision, aliases, and artifacts remain unchanged
