## ADDED Requirements

### Requirement: Actionable safe visual decoder failures
The visual decoder MUST preserve `FFMPEG_FAILED`, nonretryability and the `visual_decode` stage while distinguishing process spawn, unavailable pipes, pipe I/O, incomplete RGBA frames, empty successful output, wait failure and unsuccessful child exit with fixed safe diagnostic messages. Every failure after child creation MUST reap the child and include available exit status and bounded path-redacted stderr through the existing fields. Raw command lines, source paths and operating-system error text MUST NOT be added to messages. Existing revision checks, render publication/rollback and successful frame selection MUST remain unchanged.

#### Scenario: Empty and incomplete decoder output
- **WHEN** a decoder closes stdout with no complete frame or a trailing partial RGBA frame, including after an earlier complete frame
- **THEN** decoding fails with the distinct empty-output or incomplete-frame explanation, collected child exit status and bounded sanitized stderr; no decoded frame is returned as success

#### Scenario: Pipe I/O and process failures
- **WHEN** spawning fails, a required pipe is absent, a non-interrupted read fails, waiting fails, or a child exits unsuccessfully after complete output
- **THEN** the matching fixed explanation is returned, available diagnostics are preserved, every created child is reaped and no successful artifact is published

#### Scenario: Complete frames and interrupted reads
- **WHEN** a successful decoder emits one or more complete frames with short reads or interrupted reads
- **THEN** the last complete RGBA frame is returned byte-for-byte with the existing two-frame memory bound and no change to timestamp selection

#### Scenario: Native boundary control and causal limits
- **WHEN** a synthetic 1280x720 30fps source is decoded before and after a cut boundary
- **THEN** successful output has the expected complete-frame bytes; the recorded evidence does not claim reproduction or resolution of the private Windows #177 failure

#### Scenario: Interrupted or failed stderr collection
- **WHEN** stderr collection is interrupted or fails after collecting a bounded diagnostic fragment
- **THEN** interruptions are retried and a later collection failure preserves the already captured bounded fragment for the visual decoder error without exposing raw operating-system error text

#### Scenario: Failed-output producer remains alive after EOF
- **WHEN** a decoder closes stdout empty or incomplete but remains alive
- **THEN** the adapter gives it at most a 100ms exit-status collection grace before termination/reaping rather than waiting indefinitely, retains diagnostics, and publishes no frame
