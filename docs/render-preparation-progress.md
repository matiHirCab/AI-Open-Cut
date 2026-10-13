# Native render preparation progress

Range previews and video exports report sampled visual preparation through the existing numeric headless progress event and MCP job status. Each successfully composed frame advances the preparation share from zero to 0.5. Final encoding maps its reported fraction onto 0.5 through 0.99. Renders without sampled preparation map encoding onto zero through 0.99 and omit preparation updates.

Values are bounded and monotonic. They measure allocated work, not elapsed time or estimated time remaining. Frames can have different costs; a single expensive frame produces an update only when composition finishes. There is no timer heartbeat or ETA. Audio normalization retains its existing reporting behavior.

Progress reaches one only after output publication and artifact validation succeed, including a reused range preview. Preparation, encoding, publication and validation failures retain their last reported fraction below one. Cancellation freezes the process-local job status; late callbacks cannot complete or advance that cancelled job. Revision and validation checks retain their existing precedence and do not manufacture preparation updates.

The JSON event remains `{ "type": "progress", "progress": 0.25 }`; worker request envelopes, protocol version, public operations, schemas and error contracts remain unchanged. The canonical behavior fixture is [render-preparation-progress-v1.json](../contracts/render-preparation-progress-v1.json). Jobs remain process-local as described in [job-restart.md](job-restart.md).

Slow core adapters exercise actual sampled frame composition and publication faults. Source and packaged MCP tests use an owned transport barrier to prove live polling and cancellation; those transport fixtures do not prove successful native media rendering. Native media and regression suites provide the separate rendering evidence.
