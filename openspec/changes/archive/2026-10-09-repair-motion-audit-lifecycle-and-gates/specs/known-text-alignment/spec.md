## ADDED Requirements

### Requirement: Isolated inference worker recovery
The provider MUST retire cancelled or timed-out workers before dispatching queued or immediate successor inference, preserving concurrency-one FIFO and existing JOB_CANCELLED or TRANSCRIPTION_TIMEOUT codes/retryability. Output and lifecycle callbacks from a retired worker MUST NOT reject or corrupt another worker's requests. Alignment and ordinary transcription MUST retain their existing compatible output contracts.

#### Scenario: Cancel and reuse inference
- **WHEN** active alignment or transcription is cancelled with a valid successor already queued or immediately submitted after rejection
- **THEN** cancellation remains typed and the successor completes on a healthy worker without a caller delay or overlapping inference

#### Scenario: Time out and reuse inference
- **WHEN** active inference exceeds its deadline with a valid queued or immediate successor
- **THEN** only expired work fails with TRANSCRIPTION_TIMEOUT and the successor succeeds after terminated-worker cleanup

### Requirement: Graceful direct provider disposal
Direct provider close SHALL reject new work and gracefully drain active inference before terminating/reaping its worker. Bridge signal shutdown MUST cancel cancellable jobs before disposing the provider, retaining existing application shutdown order and typed cancellation.

#### Scenario: Close an active provider directly
- **WHEN** direct close begins during finite active inference
- **THEN** the active operation completes, new work is unavailable, and close waits for worker cleanup

#### Scenario: Shut down the bridge during inference
- **WHEN** the bridge receives its supported shutdown signal during cancellable inference
- **THEN** jobs are aborted before graceful provider disposal and the bridge and provider worker terminate without waiting for the inference deadline
